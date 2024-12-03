use std::{collections::HashMap, path::Path};

use futures::{stream, StreamExt};
use tokio::{
    io::{AsyncRead, AsyncSeek, AsyncWrite},
    time::Instant,
};

use crate::{
    config::{crypt_config::CryptConfig, AesConfig},
    constants::{color, strings},
    crypto::{
        aes_msgpack,
        assetbundle::{self, AbCryptArgs},
    },
    enums::CryptOperation,
    error::{CommandError, Error},
    models::serde::ValueF32,
    utils::{
        fs::{deserialize_file, scan_path, write_file},
        progress::ProgressBar,
    },
};

// When deserializing suitemaster files, we have to be careful to deserialize floats as f32
// Otherwise the game will not be able to properly read the values and crash/error.
type DeserializedSuiteFile = (String, ValueF32);

/// A struct responsible for encryption.
#[derive(Default)]
pub struct Encrypter {
    config: CryptConfig,
}

impl Encrypter {
    /// Creates a new Encrypter that will use the provided configuration.
    pub fn new(config: CryptConfig) -> Self {
        Self { config }
    }

    /// Encrypts an assetbundle from a Reader, returning the encrypted bytes.
    pub async fn encrypt_ab(
        reader: &mut (impl AsyncWrite + AsyncSeek + AsyncRead + Unpin),
    ) -> Result<Vec<u8>, Error> {
        let encrypted_bytes = assetbundle::encrypt(reader).await?;
        Ok(encrypted_bytes)
    }

    /// Encrypts assetbundles at a path.
    /// The path can lead to either a file or directory.
    ///
    /// If out_path is not provided, files will be encrypted in-place.
    /// Truncates and overwrites the file(s) at out_path.
    ///
    /// Returns the number of files that were successfully encrypted.
    pub async fn encrypt_ab_path(
        &self,
        in_path: impl AsRef<Path>,
        out_path: Option<impl AsRef<Path>>,
    ) -> Result<usize, Error> {
        let crypt_config = AbCryptArgs {
            recursive: self.config.recursive,
            quiet: self.config.quiet,
            concurrent: self.config.concurrency,
            operation: CryptOperation::Encrypt,
            strings: assetbundle::AbCryptStrings {
                process: strings::crypto::encrypt::PROCESS,
                processed: strings::crypto::encrypt::PROCESSED,
            },
        };

        let files_changed =
            assetbundle::crypt_path(in_path.as_ref(), out_path.as_ref(), &crypt_config).await?;

        Ok(files_changed)
    }

    /// Encrypts suitemaster .json files located at ``in_path`` into AES encrypted msgpack files.
    ///
    /// ``split`` determines how many files this data will be encrypted into.
    ///
    /// For example, if you had 100 suitemaster files and split was 3,
    /// 3 files that contain the data for those suitemaster files will be saved to ``out_path``
    ///
    /// Returns the number of files that were successfully encrypted.
    pub async fn encrypt_suite_path(
        &self,
        in_path: impl AsRef<Path>,
        out_path: impl AsRef<Path>,
        split: usize,
    ) -> Result<usize, Error> {
        let show_progress = !self.config.quiet;
        let encrypt_start = Instant::now();

        // get the paths to files to encrypt
        let paths = scan_path(in_path.as_ref(), self.config.recursive).await?;

        // create decrypt progress bar
        let deserialize_progress = if show_progress {
            println!(
                "{}[1/2] {}{}",
                color::TEXT_VARIANT.render_fg(),
                color::TEXT.render_fg(),
                strings::command::SUITE_PROCESSING,
            );
            Some(ProgressBar::progress(paths.len() as u64))
        } else {
            None
        };

        // deserialize all paths to [`serde_json::Value`]s.
        let mut deserialized_files: Vec<DeserializedSuiteFile> = Vec::new();
        {
            let deserialize_results: Vec<Result<DeserializedSuiteFile, CommandError>> =
                stream::iter(&paths)
                    .map(|path| async {
                        match path.file_stem().and_then(|os_str| os_str.to_str()) {
                            Some(file_stem) => match deserialize_file(&path.clone()).await {
                                Ok(value) => {
                                    if let Some(progress) = &deserialize_progress {
                                        progress.inc(1);
                                    }
                                    Ok((file_stem.into(), value))
                                }
                                Err(err) => Err(err.into()),
                            },
                            None => Err(CommandError::FileStem(path.to_str().unwrap_or("").into())),
                        }
                    })
                    .buffer_unordered(self.config.concurrency)
                    .collect()
                    .await;

            // separate errors and DeserializedFiles
            let mut errors = Vec::new();

            for result in deserialize_results {
                match result {
                    Ok(de_file) => deserialized_files.push(de_file),
                    Err(err) => errors.push(err),
                }
            }

            if !errors.is_empty() {
                return Err(CommandError::from(errors).into());
            }
        }

        if let Some(progress) = deserialize_progress {
            progress.finish_and_clear();
        }

        // split into chunks and serialize
        if show_progress {
            println!(
                "{}[2/2] {}{}",
                color::TEXT_VARIANT.render_fg(),
                color::TEXT.render_fg(),
                strings::command::SUITE_SAVING,
            );
        }

        let deserialized_len = deserialized_files.len();
        let chunk_size = {
            let max_chunks = split.clamp(1, deserialized_len);
            (deserialized_len + max_chunks - 1) / max_chunks
        };

        let chunks: Vec<Result<Vec<u8>, rmp_serde::encode::Error>> =
            stream::iter(deserialized_files.chunks(chunk_size))
                .map(|chunk| async {
                    match serialize_values(chunk, &self.config.aes_config) {
                        Ok(bytes) => Ok(bytes),
                        Err(err) => Err(err),
                    }
                })
                .buffer_unordered(self.config.concurrency)
                .collect()
                .await;

        // write to out directory
        for (n, result) in chunks.into_iter().enumerate() {
            let bytes = result?;
            let out_path = out_path.as_ref().join(format!(
                "{:02}{}",
                n,
                strings::command::SUITE_ENCRYPTED_FILE_NAME
            ));
            write_file(&out_path, &bytes).await?;
        }

        if show_progress {
            println!(
                "{}Successfully {} {} files in {:?}.{}",
                color::SUCCESS.render_fg(),
                strings::crypto::encrypt::PROCESSED,
                deserialized_len,
                Instant::now().duration_since(encrypt_start),
                color::TEXT.render_fg(),
            );
        }

        Ok(deserialized_len)
    }
}

fn serialize_values(
    chunk: &[DeserializedSuiteFile],
    aes_config: &AesConfig,
) -> Result<Vec<u8>, rmp_serde::encode::Error> {
    let values_map: HashMap<String, ValueF32> = chunk
        .iter()
        .map(|file| (file.0.clone(), file.1.clone()))
        .collect();

    aes_msgpack::into_vec(&values_map, aes_config)
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;
    use tokio::fs::write;

    use super::*;

    #[tokio::test]
    async fn test_encrypter_encrypt_suite_path() -> Result<(), Error> {
        let in_dir = tempdir()?;
        let out_dir = tempdir()?;
        let split_count = 3;

        // create mock suite files
        write(
            &in_dir.path().join("suite1.json"),
            r#"{"test": true, "number": 52, "string": "hello world!"}"#,
        )
        .await?;
        write(&in_dir.path().join("suite2.json"), r#"{"test": false}"#).await?;
        write(
            &in_dir.path().join("suite3.json"),
            r#"{"test": false, "number": 52512131243125152, "nested": {"string": "hello world"}}"#,
        )
        .await?;

        // encrypt to out_dir
        let encrypter = Encrypter::new(CryptConfig::builder().quiet(true).build());

        encrypter
            .encrypt_suite_path(in_dir.path(), out_dir.path(), split_count)
            .await?;

        // check if the encrypter successfully output to out_dir
        let out_files = {
            let mut files = Vec::new();
            if let Ok(mut read_dir) = tokio::fs::read_dir(out_dir.path()).await {
                while let Ok(Some(path)) = read_dir.next_entry().await {
                    files.push(path.path())
                }
            }

            files
        };
        assert_eq!(out_files.len(), split_count);

        Ok(())
    }
}

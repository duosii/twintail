use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use futures::{stream, StreamExt};
use serde_json::Value;
use tokio::{
    fs::{read, File},
    io::{AsyncRead, AsyncReadExt, AsyncSeek, AsyncWrite},
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
    error::Error,
    utils::{
        fs::{extract_suitemaster_file, scan_path, write_file},
        progress::ProgressBar,
    },
};

/// A struct responsible for decryption.
#[derive(Default)]
pub struct Decrypter {
    config: CryptConfig,
}

impl Decrypter {
    /// Creates a new Decrypter that will use the provided configuration.
    pub fn new(config: CryptConfig) -> Self {
        Self { config }
    }

    /// Decrypts msgpack + AES encrypted bytes into a type that implements the trait [`serde::de::DeserializeOwned`].
    pub fn decrypt_aes_msgpack<S>(&self, bytes: &[u8]) -> Result<S, Error>
    where
        S: serde::de::DeserializeOwned,
    {
        let deserialized = aes_msgpack::from_slice(bytes, &self.config.aes_config)?;
        Ok(deserialized)
    }

    /// Decrypts an aes msgpack file at the provided ``in_path`` into a JSON value.
    ///
    /// The .json file at ``in_path`` will be deserialized as a [`crate::models::serde::ValueF32`] before being encrypted.
    ///
    /// The file will be AES encrypted according to this encryptor's AES config.
    pub async fn decrypt_file_aes_msgpack(
        &self,
        in_path: impl AsRef<Path>,
        out_path: impl AsRef<Path>,
    ) -> Result<(), Error> {
        let file_bytes = read(in_path).await?;

        let decrypted: Value = self.decrypt_aes_msgpack(&file_bytes)?;
        let json_bytes = serde_json::to_vec_pretty(&decrypted)?;
        write_file(out_path, &json_bytes).await?;
        Ok(())
    }

    /// Decrypts an assetbundle from a Reader, returning the decrypted bytes.
    pub async fn decrypt_ab(
        reader: &mut (impl AsyncWrite + AsyncRead + AsyncSeek + Unpin),
    ) -> Result<Vec<u8>, Error> {
        let decrypted_bytes = assetbundle::decrypt(reader).await?;
        Ok(decrypted_bytes)
    }

    /// Decrypts assetbundles at a path.
    /// The path can lead to either a file or directory.
    ///
    /// If out_path is not provided, files will be decrypted in-place.
    /// Truncates and overwrites the file(s) at out_path.
    ///
    /// Returns the number of files that were successfully decrypted.
    pub async fn decrypt_ab_path(
        &self,
        in_path: impl AsRef<Path>,
        out_path: Option<impl AsRef<Path>>,
    ) -> Result<usize, Error> {
        let crypt_config = AbCryptArgs {
            recursive: self.config.recursive,
            quiet: self.config.quiet,
            concurrent: self.config.concurrency,
            operation: CryptOperation::Decrypt,
            strings: assetbundle::AbCryptStrings {
                process: strings::crypto::decrypt::PROCESS,
                processed: strings::crypto::decrypt::PROCESSED,
            },
        };

        let files_changed =
            assetbundle::crypt_path(in_path.as_ref(), out_path.as_ref(), &crypt_config).await?;

        Ok(files_changed)
    }

    /// Decrypts suitemaster files located at ``in_path`` into .json files at ``out_path``.
    ///
    /// Returns the number of files that were successfully decrypted.
    pub async fn decrypt_suite_path(
        &self,
        in_path: impl AsRef<Path>,
        out_path: impl AsRef<Path>,
    ) -> Result<usize, Error> {
        // get paths that we need to decrypt
        let decrypt_start = Instant::now();
        let to_decrypt_paths = scan_path(in_path.as_ref(), self.config.recursive).await?;
        let out_path = out_path.as_ref();
        let show_progress = !self.config.quiet;

        // create decrypt progress bar
        if show_progress {
            println!(
                "{}[1/1] {}{}",
                color::TEXT_VARIANT.render_fg(),
                color::TEXT.render_fg(),
                strings::command::SUITE_DECRYPTING,
            );
        }
        let decrypt_progress = ProgressBar::progress(to_decrypt_paths.len() as u64);
        decrypt_progress.enable_steady_tick(Duration::from_millis(200));

        // begin decrypting
        let pretty_json = self.config.pretty_json;
        let decrypt_results: Vec<Result<(), Error>> = stream::iter(to_decrypt_paths)
            .map(|in_path| async {
                let decrypt_result = decrypt_suitemaster_file(
                    in_path,
                    out_path,
                    &self.config.aes_config,
                    pretty_json,
                )
                .await;
                if show_progress {
                    decrypt_progress.inc(1)
                }
                decrypt_result
            })
            .buffer_unordered(self.config.concurrency)
            .collect()
            .await;

        decrypt_progress.finish_and_clear();

        // count the number of successes
        let success_count = decrypt_results
            .iter()
            .filter(|&result| {
                if let Err(err) = result {
                    if show_progress {
                        println!("suite decrypt error: {:?}", err);
                    }
                    false
                } else {
                    true
                }
            })
            .count();

        // print the result
        if show_progress {
            println!(
                "{}Successfully {} {} files in {:?}.{}",
                color::SUCCESS.render_fg(),
                strings::crypto::decrypt::PROCESSED,
                success_count,
                Instant::now().duration_since(decrypt_start),
                color::TEXT.render_fg(),
            );
        }

        Ok(success_count)
    }
}

/// Reads the file at the input path as a [`serde_json::Value`]
/// and extracts its inner fields to out_path as .json files.
///
/// If pretty is true, then the extracted suitemaster json files will be prettified.
async fn decrypt_suitemaster_file(
    in_path: PathBuf,
    out_path: &Path,
    aes_config: &AesConfig,
    pretty: bool,
) -> Result<(), Error> {
    // read in file
    let mut file = File::open(in_path).await?;
    let mut file_buf = Vec::new();
    file.read_to_end(&mut file_buf).await?;

    // deserialize as a value
    let deserialized: Value = aes_msgpack::from_slice(&file_buf, aes_config)?;

    // write to out_path
    extract_suitemaster_file(deserialized, out_path, pretty).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::Server;
    use tempfile::tempdir;
    use tokio::fs::{read_to_string, write};

    #[tokio::test]
    async fn test_decrypter_decrypt_file_aes_msgpack() -> Result<(), Error> {
        let in_dir = tempdir()?;
        let aes_config = Server::Japan.get_aes_config();

        let file_json = r#"
            {
                "hatsune": "miku",
                "kasane": 39
            }
            "#;
        let file_json_value: Value = serde_json::from_str(&file_json)?;
        let file_json_aes_msgpack_bytes =
            aes_msgpack::into_vec(&file_json_value, &aes_config)?;

        let in_file_path = in_dir.path().join("file");
        write(&in_file_path, file_json_aes_msgpack_bytes).await?;

        let out_file_path = in_dir.path().join("file.json");
        let encrypter = Decrypter::new(
            CryptConfig::builder()
                .quiet(true)
                .aes(aes_config.clone())
                .build(),
        );
        encrypter
            .decrypt_file_aes_msgpack(&in_file_path, &out_file_path)
            .await?;

        let decrypted_file_string = read_to_string(&out_file_path).await?;
        let decrypted_file_value: Value = serde_json::from_str(&decrypted_file_string)?;
        assert_eq!(file_json_value, decrypted_file_value);
        Ok(())
    }
}

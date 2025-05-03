use std::{collections::HashMap, path::Path};

use rayon::iter::{ParallelBridge, ParallelIterator};
use tokio::io::{AsyncRead, AsyncSeek, AsyncWrite};
use twintail_common::{
    color,
    crypto::{aes::AesConfig, aes_msgpack},
    models::{enums::CryptOperation, serde::ValueF32},
    utils::progress::ProgressBar,
};

use crate::{
    config::crypt_config::CryptConfig,
    crypto::assetbundle::{self, AbCryptArgs},
    error::Error,
    fs::{deserialize_files, scan_path, write_file},
};

mod strings {
    pub const PROCESS: &str = "Encrypting";
    pub const PROCESSED: &str = "encrypted";
    pub const SUITE_PROCESSING: &str = "Processing suitemaster files...";
    pub const SUITE_SAVING: &str = "Saving encrypted suitemaster files...";
    pub const SUITE_ENCRYPTED_FILE_NAME: &str = "_suitemasterfile";
}

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
                process: strings::PROCESS,
                processed: strings::PROCESSED,
            },
        };

        let files_changed =
            assetbundle::crypt_path(in_path.as_ref(), out_path.as_ref(), &crypt_config).await?;

        Ok(files_changed)
    }

    pub async fn encrypt_suite_values(
        &self,
        values: &[(String, ValueF32)],
        out_path: impl AsRef<Path>,
        split: usize,
    ) -> Result<usize, Error> {
        let to_serialize_count = values.len();

        // split into chunks and serialize
        let serialize_progress = if !self.config.quiet {
            println!(
                "{}{}{}",
                color::TEXT_VARIANT.render_fg(),
                color::TEXT.render_fg(),
                strings::SUITE_SAVING,
            );
            Some(ProgressBar::progress(to_serialize_count as u64))
        } else {
            None
        };

        let deserialized_len = values.len();
        let chunk_size = {
            let max_chunks = split.clamp(1, deserialized_len);
            deserialized_len.div_ceil(max_chunks)
        };

        let chunks: Vec<Result<Vec<u8>, rmp_serde::encode::Error>> = values
            .chunks(chunk_size)
            .par_bridge()
            .map(|chunk| {
                if let Some(progress) = &serialize_progress {
                    progress.inc(1);
                }
                match serialize_values(chunk, &self.config.aes_config) {
                    Ok(bytes) => Ok(bytes),
                    Err(err) => Err(err),
                }
            })
            .collect();

        if let Some(progress) = &serialize_progress {
            progress.finish_and_clear();
        }

        // write to out directory
        for (n, result) in chunks.into_iter().enumerate() {
            let bytes = result?;
            let out_path =
                out_path
                    .as_ref()
                    .join(format!("{:02}{}", n, strings::SUITE_ENCRYPTED_FILE_NAME));
            write_file(&out_path, &bytes).await?;
        }

        Ok(deserialized_len)
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
        // create decrypt progress bar
        let deserialize_progress = if !self.config.quiet {
            println!(
                "{}{}{}",
                color::TEXT_VARIANT.render_fg(),
                color::TEXT.render_fg(),
                strings::SUITE_PROCESSING,
            );
            Some(ProgressBar::spinner())
        } else {
            None
        };

        // deserialize all paths to [`serde_json::Value`]s.
        let deserialized_files: Vec<(_, ValueF32)> = self.deserialize_suite_path(in_path).await?;

        if let Some(progress) = deserialize_progress {
            progress.finish_and_clear();
        }

        self.encrypt_suite_values(&deserialized_files, out_path, split)
            .await
    }

    /// Deserializes suite files located at a specific path into [crate::models::serde::ValueF32].
    /// This function returns a Vec of tuples where the first value is the name of the file (without an extension)
    /// and the second value is teh deserialized value of the file.
    pub async fn deserialize_suite_path(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<Vec<(String, ValueF32)>, Error> {
        // get the paths to files to encrypt
        let paths = scan_path(path.as_ref(), self.config.recursive).await?;

        let values = deserialize_files(&paths)?;
        Ok(values)
    }

    /// Encrypts any value that implements [`serde::Serialize`] into msgpack + AES encrypted bytes.
    ///
    /// The value will be AES encrypted according to this encryptor's AES config.
    ///
    /// This function will return a Vec of bytes containing the encrypted representation of the provided ``value``
    pub fn encrypt_aes_msgpack<S>(&self, value: &S) -> Result<Vec<u8>, Error>
    where
        S: serde::Serialize,
    {
        let encrypted_bytes = aes_msgpack::into_vec(&value, &self.config.aes_config)?;
        Ok(encrypted_bytes)
    }

    /// Encrypts bytes into msgpack + AES encrypted bytes.
    ///
    /// The bytes will be deserialized as a [`crate::models::serde::ValueF32`] before being encrypted.
    ///
    /// The file will be AES encrypted according to this encryptor's AES config.
    ///
    /// This function will return a Vec of bytes containing the encrypted representation of the provided ``json_bytes``
    pub fn encrypt_json_bytes_aes_msgpack(&self, json_bytes: &[u8]) -> Result<Vec<u8>, Error> {
        let bytes_deserialized: ValueF32 = serde_json::from_slice(json_bytes)?;
        self.encrypt_aes_msgpack(&bytes_deserialized)
    }

    /// Encrypts a .json file at the provided ``in_path`` into a msgpack + AES encrypted value.
    ///
    /// The .json file at ``in_path`` will be deserialized as a [`crate::models::serde::ValueF32`] before being encrypted.
    ///
    /// The file will be AES encrypted according to this encryptor's AES config.
    pub async fn encrypt_file_aes_msgpack(
        &self,
        in_path: impl AsRef<Path>,
        out_path: impl AsRef<Path>,
    ) -> Result<(), Error> {
        let file_bytes = tokio::fs::read(in_path.as_ref()).await?;
        let encrypted_bytes = self.encrypt_json_bytes_aes_msgpack(&file_bytes)?;
        write_file(out_path, &encrypted_bytes).await?;
        Ok(())
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
    use serde_json::Value;
    use tempfile::tempdir;
    use tokio::fs::{read, write};
    use twintail_common::models::enums::Server;

    use super::*;

    #[tokio::test]
    async fn test_encrypter_encrypt_json_bytes() -> Result<(), Error> {
        let json_bytes = r#"
            {
                "name": "inabakumori",
                "values": [
                    "value1",
                    "value2"
                ],
                "songs": 3
            }
        "#
        .as_bytes();

        let encrypter = Encrypter::new(CryptConfig::builder().quiet(true).build());
        encrypter.encrypt_json_bytes_aes_msgpack(json_bytes)?;

        Ok(())
    }

    #[tokio::test]
    async fn test_encrypter_encrypt_json_file() -> Result<(), Error> {
        let dir = tempdir()?;

        let in_path = &dir.path().join("suite1.json");
        let out_path = &dir.path().join("out.json");

        write(
            &in_path,
            r#"
                {
                    "name": "inabakumori",
                    "values": [
                        "value1",
                        "value2"
                    ],
                    "songs": 3
                }
            "#,
        )
        .await?;

        let encrypter = Encrypter::new(CryptConfig::builder().quiet(true).build());
        encrypter
            .encrypt_file_aes_msgpack(in_path, out_path)
            .await?;

        assert!(out_path.exists(), "file should have been created");

        Ok(())
    }

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

    #[tokio::test]
    async fn test_encrypter_encrypt_file_aes_msgpack() -> Result<(), Error> {
        let in_dir = tempdir()?;
        let aes_config = Server::Japan.get_aes_config();

        let in_file_path = in_dir.path().join("file.json");
        let in_file_json = r#"{"hatsune":"miku","kasane":39}"#;

        write(&in_file_path, in_file_json).await?;

        // generate expected value
        let in_file_json_value: Value = serde_json::from_str(&in_file_json)?;

        // encrypt in_file
        let out_file_path = in_dir.path().join("file");
        let encrypter = Encrypter::new(
            CryptConfig::builder()
                .quiet(true)
                .aes(aes_config.clone())
                .build(),
        );
        encrypter
            .encrypt_file_aes_msgpack(&in_file_path, &out_file_path)
            .await?;

        let out_file_bytes = read(out_file_path).await?;
        let out_file_value: Value = aes_msgpack::from_slice(&out_file_bytes, &aes_config)?;

        assert_eq!(in_file_json_value, out_file_value);
        Ok(())
    }
}

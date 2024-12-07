use std::{path::{Path, PathBuf}, time::Duration};

use futures::{stream, StreamExt};
use serde_json::Value;
use tokio::{
    fs::File,
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
        fs::{extract_suitemaster_file, scan_path},
        progress::ProgressBar,
    },
};

/// A struct responsible for encryption.
#[derive(Default)]
pub struct Decrypter {
    config: CryptConfig,
}

impl Decrypter {
    /// Creates a new Decrypter that will use the provided configuration.
    pub fn new(config: CryptConfig) -> Self {
        Self { config }
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
                let decrypt_result =
                    decrypt_suitemaster_file(in_path, out_path, &self.config.aes_config, pretty_json).await;
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
    pretty: bool
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

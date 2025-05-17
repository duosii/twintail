use crate::{Error, color, strings};
use clap::Args;
use tokio::{sync::watch::Receiver, time::Instant};
use twintail_common::{models::OptionalBuilder, utils::progress::ProgressBar};
use twintail_core::{
    config::crypt_config::CryptConfig,
    crypto::{CryptAssetbundlePathState, CryptState, encrypt::Encrypter},
};

#[derive(Debug, Args)]
pub struct EncryptAbArgs {
    /// If the input is a directory, whether to recursively encrypt valid files in that directory
    #[arg(long, short, default_value_t = false)]
    pub recursive: bool,

    /// The maximum number of files to encrypt simultaneously
    #[arg(long, short)]
    pub concurrent: Option<usize>,

    /// Whether to output status messages
    #[arg(short, long, default_value_t = false)]
    pub quiet: bool,

    /// Path to the file or directory to encrypt
    pub in_path: String,

    /// Path to a directory or file to output to. If not provided, files are encrypted in-place.
    pub out_path: Option<String>,
}

/// Watches a [`tokio::sync::watch::Receiver`] for state changes.
///
/// Prints information related to the progress of an assetbundle encrypt.
async fn watch_encrypt_ab_state(mut receiver: Receiver<CryptState>) {
    let mut progress_bar: Option<indicatif::ProgressBar> = None;
    while receiver.changed().await.is_ok() {
        match *receiver.borrow_and_update() {
            CryptState::AssetbundlePath(CryptAssetbundlePathState::Scan) => {
                println!(
                    "{}[1/2] {}Scanning files...",
                    color::TEXT_VARIANT.render_fg(),
                    color::TEXT.render_fg()
                );
                progress_bar = Some(ProgressBar::spinner())
            }
            CryptState::AssetbundlePath(CryptAssetbundlePathState::Crypt(file_count)) => {
                if let Some(spinner) = &progress_bar {
                    spinner.finish_and_clear();
                }

                println!(
                    "{}[2/2] {}{} files...",
                    color::TEXT_VARIANT.render_fg(),
                    color::TEXT.render_fg(),
                    strings::crypto::encrypt::PROCESS,
                );
                progress_bar = Some(ProgressBar::progress(file_count as u64))
            }
            CryptState::AssetbundlePath(CryptAssetbundlePathState::CryptFile) => {
                if let Some(progress) = &progress_bar {
                    progress.inc(1);
                }
            }
            CryptState::AssetbundlePath(CryptAssetbundlePathState::Finish) => {
                if let Some(progress) = &progress_bar {
                    progress.finish_and_clear();
                }
                break;
            }
            _ => {}
        }
    }
}

/// Encrypts a file/folder using the provided arguments.
pub async fn encrypt_ab(args: EncryptAbArgs) -> Result<(), Error> {
    let crypt_start = Instant::now();

    let config = CryptConfig::builder()
        .recursive(args.recursive)
        .map(args.concurrent, |config, concurrency| {
            config.concurrency(concurrency)
        })
        .build();

    let (encrypter, state_recv) = Encrypter::new(config);

    let state_watcher = if args.quiet {
        None
    } else {
        Some(tokio::spawn(watch_encrypt_ab_state(state_recv)))
    };

    let (encrypt_count, total_file_count) = encrypter
        .encrypt_ab_path(args.in_path, args.out_path)
        .await?;

    if let Some(watcher) = state_watcher {
        watcher.await?;
        println!(
            "{}Successfully {} {} / {} files in {:?}.{}",
            color::SUCCESS.render_fg(),
            strings::crypto::encrypt::PROCESSED,
            encrypt_count,
            total_file_count,
            Instant::now().duration_since(crypt_start),
            color::TEXT.render_fg(),
        );
    }

    Ok(())
}

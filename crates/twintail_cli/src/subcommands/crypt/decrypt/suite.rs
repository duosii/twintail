use std::time::Duration;

use crate::{Error, color, strings};
use clap::Args;
use tokio::{sync::watch::Receiver, time::Instant};
use twintail_common::{models::enums::Server, utils::progress::ProgressBar};
use twintail_core::{
    config::{OptionalBuilder, crypt_config::CryptConfig},
    crypto::{CryptState, DecryptSuitePathState, decrypt::Decrypter},
};

#[derive(Debug, Args)]
pub struct DecryptSuiteArgs {
    /// If the input is a directory, whether to recursively decrypt valid files in that directory
    #[arg(long, short, default_value_t = false)]
    pub recursive: bool,

    /// The maximum number of files to decrypt simultaneously
    #[arg(long, short)]
    pub concurrent: Option<usize>,

    /// The server to decrypt the suitemasterfiles for
    #[arg(short, long, value_enum, default_value_t = Server::Japan)]
    pub server: Server,

    /// Whether to output status messages
    #[arg(short, long, default_value_t = false)]
    pub quiet: bool,

    /// Whether to save suitemaster .json files in a more compact format, reducing their file size
    #[arg(long, default_value_t = false)]
    pub compact: bool,

    /// Path to the file or directory to decrypt
    pub in_path: String,

    /// Path to a directory or file to output to.
    pub out_path: String,
}

/// Watches a [`tokio::sync::watch::Receiver`] for state changes.
///
/// Prints information related to the progress of a suite decrypt.
async fn watch_decrypt_suite_state(mut receiver: Receiver<CryptState>) {
    let mut progress_bar: Option<indicatif::ProgressBar> = None;
    while receiver.changed().await.is_ok() {
        match *receiver.borrow_and_update() {
            CryptState::DecryptSuitePath(DecryptSuitePathState::Start(file_count)) => {
                println!(
                    "{}[1/1] {}{}",
                    color::TEXT_VARIANT.render_fg(),
                    color::TEXT.render_fg(),
                    strings::command::SUITE_DECRYPTING,
                );
                let decrypt_progress = ProgressBar::progress(file_count as u64);
                decrypt_progress.enable_steady_tick(Duration::from_millis(200));
                progress_bar = Some(decrypt_progress)
            }
            CryptState::DecryptSuitePath(DecryptSuitePathState::Decrypt) => {
                if let Some(progress) = &progress_bar {
                    progress.inc(1);
                }
            }
            CryptState::DecryptSuitePath(DecryptSuitePathState::Finish) => {
                if let Some(progress) = &progress_bar {
                    progress.finish_and_clear();
                }
                break;
            }
            _ => {}
        }
    }
}

/// Decrypts encrypted suitemaster files into individual .json files.
pub async fn decrypt_suite(args: DecryptSuiteArgs) -> Result<(), Error> {
    let decrypt_start_instant = Instant::now();

    let config = CryptConfig::builder()
        .recursive(args.recursive)
        .server(args.server)
        .pretty_json(!args.compact)
        .map(args.concurrent, |config, concurrency| {
            config.concurrency(concurrency)
        })
        .build();

    let (decrypter, state_recv) = Decrypter::new(config);

    let state_watcher = if args.quiet {
        None
    } else {
        Some(tokio::spawn(watch_decrypt_suite_state(state_recv)))
    };

    let success_count = decrypter
        .decrypt_suite_path(args.in_path, args.out_path)
        .await?;

    if let Some(watcher) = state_watcher {
        watcher.await?;
        println!(
            "{}Successfully {} {} files in {:?}.{}",
            color::SUCCESS.render_fg(),
            strings::crypto::decrypt::PROCESSED,
            success_count,
            Instant::now().duration_since(decrypt_start_instant),
            color::TEXT.render_fg(),
        );
    }

    Ok(())
}

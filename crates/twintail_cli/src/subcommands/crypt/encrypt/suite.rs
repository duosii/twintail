use clap::Args;
use tokio::{sync::watch::Receiver, time::Instant};
use twintail_common::models::{OptionalBuilder, enums::Server};
use twintail_core::{
    config::crypt_config::CryptConfig,
    crypto::{CryptState, EncryptSuitePathState, EncryptSuiteValuesState, encrypt::Encrypter},
};

use crate::{Error, color, progress::ProgressBar, strings};

#[derive(Debug, Args)]
pub struct EncryptSuiteArgs {
    /// If the input is a directory, whether to recursively encrypt valid files in that directory
    #[arg(long, short, default_value_t = false)]
    pub recursive: bool,

    /// The maximum number of files to encrypt simultaneously
    #[arg(long, short)]
    pub concurrent: Option<usize>,

    /// The server to encrypt the suitemasterfiles for
    #[arg(short, long, value_enum, default_value_t = Server::Japan)]
    pub server: Server,

    /// The number of files to split the encrypted suitemaster files into
    #[arg(long, default_value_t = 7)]
    pub split: usize,

    /// Whether to output status messages
    #[arg(short, long, default_value_t = false)]
    pub quiet: bool,

    /// Path to the file or directory to encrypt
    pub in_path: String,

    /// Path to a directory or file to output to.
    pub out_path: String,
}

/// Watches a [`tokio::sync::watch::Receiver`] for state changes.
///
/// Prints information related to the progress of a suite encrypt.
async fn watch_encrypt_suite_state(mut receiver: Receiver<CryptState>) {
    let mut progress_bar: Option<indicatif::ProgressBar> = None;
    while receiver.changed().await.is_ok() {
        match *receiver.borrow_and_update() {
            CryptState::EncryptSuitePath(EncryptSuitePathState::Process) => {
                println!(
                    "{}{}{}",
                    color::TEXT_VARIANT.render_fg(),
                    color::TEXT.render_fg(),
                    strings::command::SUITE_PROCESSING,
                );
                progress_bar = Some(ProgressBar::spinner())
            }
            CryptState::EncryptSuiteValues(EncryptSuiteValuesState::SerializeStart(count)) => {
                if let Some(spinner) = &progress_bar {
                    spinner.finish_and_clear();
                }

                println!(
                    "{}{}{}",
                    color::TEXT_VARIANT.render_fg(),
                    color::TEXT.render_fg(),
                    strings::command::SUITE_SAVING,
                );
                progress_bar = Some(ProgressBar::progress(count as u64))
            }
            CryptState::EncryptSuiteValues(EncryptSuiteValuesState::Serialize(delta)) => {
                if let Some(progress) = &progress_bar {
                    progress.inc(delta as u64);
                }
            }
            CryptState::EncryptSuiteValues(EncryptSuiteValuesState::Finish) => {
                if let Some(progress) = &progress_bar {
                    progress.finish_and_clear();
                }
                break;
            }
            _ => {}
        }
    }
}

pub async fn encrypt_suite(args: EncryptSuiteArgs) -> Result<(), Error> {
    let encrypt_start = Instant::now();

    let config = CryptConfig::builder()
        .recursive(args.recursive)
        .server(args.server)
        .map(args.concurrent, |config, concurrency| {
            config.concurrency(concurrency)
        })
        .build();

    let (encrypter, state_recv) = Encrypter::new(config);

    let state_watcher = if args.quiet {
        None
    } else {
        Some(tokio::spawn(watch_encrypt_suite_state(state_recv)))
    };

    encrypter
        .encrypt_suite_path(args.in_path, args.out_path, args.split)
        .await?;

    if let Some(watcher) = state_watcher {
        watcher.await?;
        println!(
            "{}Successfully {} suite master files in {:?}.{}",
            color::SUCCESS.render_fg(),
            strings::crypto::encrypt::PROCESSED,
            Instant::now().duration_since(encrypt_start),
            color::TEXT.render_fg(),
        );
    }

    Ok(())
}

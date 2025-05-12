use std::time::Duration;

use clap::Args;
use tokio::{sync::watch::Receiver, time::Instant};
use twintail_common::{
    color,
    models::enums::{Platform, Server},
    utils::progress::ProgressBar,
};
use twintail_core::{
    config::{OptionalBuilder, fetch_config::FetchConfig},
    fetch::{DownloadSuiteState, FetchState, Fetcher},
};

use crate::{Error, strings};

#[derive(Debug, Args)]
pub struct SuiteArgs {
    /// The version of the game app get the suitemaster files for
    #[arg(short, long)]
    pub version: String,

    /// The app hash to get the suitemaster files for
    #[arg(long)]
    pub hash: String,

    /// The device platform to get the suitemaster files for
    #[arg(short, long, value_enum, default_value_t = Platform::Android)]
    pub platform: Platform,

    /// The server to get the suitemaster files from
    #[arg(short, long, value_enum, default_value_t = Server::Japan)]
    pub server: Server,

    /// The maximum number of files to download simultaneously
    #[arg(long, short)]
    pub concurrent: Option<usize>,

    /// The maximum number of times to retry a download if it fails
    #[arg(long, short, default_value_t = 3)]
    pub retry: usize,

    /// If set, the downloaded suitemaster files will not be decrypted.
    #[arg(long, short, default_value_t = false)]
    pub encrypt: bool,

    /// Whether to output status messages
    #[arg(short, long, default_value_t = false)]
    pub quiet: bool,

    /// Whether to save suitemaster .json files in a more compact format, reducing their file size
    #[arg(long, default_value_t = false)]
    pub compact: bool,

    /// The directory to output the suitemaster files to
    pub out_path: String,
}

/// Watches a [`tokio::sync::watch::Receiver`] for DownloadSuite state changes.
///
/// Prints information related to the progress of a suite download.
async fn watch_fetch_suite_state(mut receiver: Receiver<FetchState>) {
    let mut progress_bar: Option<indicatif::ProgressBar> = None;
    while receiver.changed().await.is_ok() {
        let fetch_state = receiver.borrow_and_update().clone();
        if let FetchState::DownloadSuite(download_suite_state) = fetch_state {
            match download_suite_state {
                DownloadSuiteState::Communicate => {
                    println!(
                        "{}[1/2] {}{}",
                        color::TEXT_VARIANT.render_fg(),
                        color::TEXT.render_fg(),
                        strings::command::COMMUNICATING,
                    );
                    progress_bar = Some(ProgressBar::spinner());
                }
                DownloadSuiteState::DownloadStart(file_count) => {
                    if let Some(spinner) = &progress_bar {
                        spinner.finish_and_clear();
                    }

                    println!(
                        "{}[2/2] {}{}",
                        color::TEXT_VARIANT.render_fg(),
                        color::TEXT.render_fg(),
                        strings::command::DOWNLOADING,
                    );

                    let progress = ProgressBar::progress(file_count as u64);
                    progress.enable_steady_tick(Duration::from_millis(100));
                    progress_bar = Some(progress);
                }
                DownloadSuiteState::FileDownload => {
                    if let Some(progress) = &progress_bar {
                        progress.inc(1);
                    }
                }
                DownloadSuiteState::Finish => {
                    if let Some(progress) = &progress_bar {
                        progress.finish_and_clear();
                    }
                    break;
                }
            }
        }
    }
}

pub async fn fetch_suite(args: SuiteArgs) -> Result<(), Error> {
    let quiet = args.quiet;
    // create fetcher
    let fetch_config = FetchConfig::builder(args.version, args.hash)
        .platform(args.platform)
        .server(args.server)
        .retry(args.retry)
        .decrypt(!args.encrypt)
        .quiet(quiet)
        .pretty_json(!args.compact)
        .map(args.concurrent, |config, concurrency| {
            config.concurrency(concurrency)
        })
        .build();
    let (mut fetcher, state_recv) = Fetcher::new(fetch_config).await?;

    let state_watcher = if quiet {
        None
    } else {
        Some(tokio::spawn(watch_fetch_suite_state(state_recv)))
    };

    // download suitemaster files
    let download_start = Instant::now();
    let (downloaded_count, file_count, suite_version) =
        fetcher.download_suite(args.out_path).await?;

    if let Some(watcher) = state_watcher {
        watcher.await?;
        println!(
            "{}{} {}{}",
            color::TEXT_VARIANT.render_fg(),
            strings::command::SUITE_VERSION,
            color::TEXT.render_fg(),
            suite_version
        );
        println!(
            "{}Successfully {} {} / {} files in {:?}{}",
            color::SUCCESS.render_fg(),
            strings::command::DOWNLOADED,
            downloaded_count,
            file_count,
            Instant::now().duration_since(download_start),
            color::TEXT.render_fg(),
        );
    }

    Ok(())
}

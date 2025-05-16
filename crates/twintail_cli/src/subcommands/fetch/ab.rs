use clap::Args;
use std::path::Path;
use tokio::{
    fs::File,
    io::{AsyncReadExt, BufReader},
    sync::watch::Receiver,
    time::Instant,
};
use twintail_common::{
    models::enums::{Platform, Server},
    utils::progress::ProgressBar,
};
use twintail_core::{
    config::{OptionalBuilder, download_ab_config::DownloadAbConfig, fetch_config::FetchConfig},
    fetch::{DownloadAbState, FetchState, Fetcher},
};
use twintail_sekai::models::AssetbundleInfo;

use crate::{Error, color, strings};

#[derive(Debug, Args)]
pub struct AbArgs {
    /// The version of the game app to get the assetbundles for
    #[arg(short, long)]
    pub version: String,

    /// The version of the assets to get. Uses the most recent if not provided
    #[arg(short, long)]
    pub asset_version: Option<String>,

    /// The hash of the game app to get the assetbundles for
    #[arg(long)]
    pub hash: String,

    /// Part of the URL used to download the assetbundles from. Uses the most recent if not provided
    #[arg(long)]
    pub host_hash: Option<String>,

    /// The device platform to get the assetbundles for
    #[arg(short, long, value_enum, default_value_t = Platform::Android)]
    pub platform: Platform,

    /// The server to get the assetbundles from
    #[arg(short, long, value_enum, default_value_t = Server::Japan)]
    pub server: Server,

    /// Path to an assetbundle info file. If not provided, the latest one will be fetched
    #[arg(short, long)]
    pub info: Option<String>,

    /// If set, the assetbundle info file provided with --info will not be updated to the most recent asset version
    #[arg(long, default_value_t = false)]
    pub no_update: bool,

    /// The maximum number of files to download simultaneously
    #[arg(long, short)]
    pub concurrent: Option<usize>,

    /// Only assetbundles that match this regular expression will be downloaded
    #[arg(long, short)]
    pub filter: Option<String>,

    /// The maximum number of times to retry a download if it fails
    #[arg(long, short, default_value_t = 3)]
    pub retry: usize,

    /// If present, the downloaded assetbundles will not be decrypted
    #[arg(long, short, default_value_t = false)]
    pub encrypt: bool,

    /// Whether to output status messages
    #[arg(short, long, default_value_t = false)]
    pub quiet: bool,

    /// The directory to output the assetbundles to
    pub out_dir: String,
}

/// Watches a [`tokio::sync::watch::Receiver`] for DownloadSuite state changes.
///
/// Prints information related to the progress of a suite download.
async fn watch_fetch_ab_state(mut receiver: Receiver<FetchState>) {
    let mut progress_bar: Option<indicatif::ProgressBar> = None;
    while receiver.changed().await.is_ok() {
        let fetch_state = *receiver.borrow_and_update();
        if let FetchState::DownloadAb(download_ab_state) = fetch_state {
            match download_ab_state {
                DownloadAbState::RetrieveAbInfo => {
                    println!(
                        "{}[1/2] {}{}",
                        color::TEXT_VARIANT.render_fg(),
                        color::TEXT.render_fg(),
                        strings::command::RETRIEVING_AB_INFO,
                    );
                    progress_bar = Some(ProgressBar::spinner());
                }
                DownloadAbState::InvalidRegEx => {
                    println!(
                        "{}{}{}",
                        color::ERROR.render_fg(),
                        strings::command::INVALID_RE,
                        color::TEXT.render_fg()
                    )
                }
                DownloadAbState::DownloadStart(total_bytes) => {
                    if let Some(spinner) = &progress_bar {
                        spinner.finish_and_clear();
                    }

                    println!(
                        "{}[2/2] {}{}",
                        color::TEXT_VARIANT.render_fg(),
                        color::TEXT.render_fg(),
                        strings::command::DOWNLOADING,
                    );

                    progress_bar = Some(ProgressBar::download(total_bytes));
                }
                DownloadAbState::FileDownload(file_size_bytes) => {
                    if let Some(progress) = &progress_bar {
                        progress.inc(file_size_bytes);
                    }
                }
                DownloadAbState::Finish => {
                    if let Some(progress) = &progress_bar {
                        progress.finish_and_clear();
                    }
                    break;
                }
            }
        }
    }
}

/// Reads and deserializes an assetbundle info from a .json file.
async fn read_assetbundle_info(path: &str) -> Result<AssetbundleInfo, Error> {
    // read file
    let file = File::open(path).await?;
    let mut reader = BufReader::new(file);
    let mut file_buf = Vec::new();
    reader.read_to_end(&mut file_buf).await?;

    // deserialize
    Ok(serde_json::from_slice(&file_buf)?)
}

pub async fn fetch_ab(args: AbArgs) -> Result<(), Error> {
    // read ab info if it was provided
    let info = if let Some(string_path) = args.info {
        let assetbundle_info_path = Path::new(&string_path);
        let file_exists = assetbundle_info_path.try_exists().unwrap_or(false);

        if file_exists {
            // read file
            let info = read_assetbundle_info(&string_path).await?;
            Some(info)
        } else {
            None
        }
    } else {
        None
    };

    // build ab_config
    let download_ab_config = DownloadAbConfig::builder()
        .update(!args.no_update)
        .map(info, |config, info| config.info(info))
        .map(args.asset_version, |config, asset_version| {
            config.asset_version(asset_version)
        })
        .map(args.host_hash, |config, host_hash| {
            config.host_hash(host_hash)
        })
        .map(args.filter, |config, filter| config.filter(filter))
        .build();

    // build config
    let quiet = args.quiet;
    let fetch_config = FetchConfig::builder(args.version, args.hash)
        .platform(args.platform)
        .server(args.server)
        .retry(args.retry)
        .decrypt(!args.encrypt)
        .quiet(quiet)
        .map(args.concurrent, |config, concurrency| {
            config.concurrency(concurrency)
        })
        .build();

    // create fetcher
    let (mut fetcher, state_recv) = Fetcher::new(fetch_config).await?;

    // spawn thread for watching state_recv
    let state_watcher = if quiet {
        None
    } else {
        Some(tokio::spawn(watch_fetch_ab_state(state_recv)))
    };

    // download assetbundles
    let download_start = Instant::now();
    let (success_count, total_file_count) = fetcher
        .download_ab(args.out_dir, download_ab_config)
        .await?;

    if let Some(watcher) = state_watcher {
        watcher.await?;
        println!(
            "{}Successfully {} {} / {} files in {:?}{}",
            color::SUCCESS.render_fg(),
            strings::command::DOWNLOADED,
            success_count,
            total_file_count,
            Instant::now().duration_since(download_start),
            color::TEXT.render_fg(),
        );
    }

    Ok(())
}

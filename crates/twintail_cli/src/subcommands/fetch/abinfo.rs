use clap::Args;
use indicatif::ProgressBar;
use std::{path::Path, time::Duration};
use tokio::{
    fs::{File, create_dir_all},
    io::AsyncWriteExt,
};
use twintail_common::{
    color,
    models::enums::{Platform, Server},
};
use twintail_core::{config::fetch_config::FetchConfig, fetch::Fetcher};

use crate::{Error, strings};

#[derive(Debug, Args)]
pub struct AbInfoArgs {
    /// The version of the game app get the assetbundle information for
    #[arg(short, long)]
    pub version: String,

    /// The version of the assets to get information about. Uses the most recent if not provided
    #[arg(short, long)]
    pub asset_version: Option<String>,

    /// The app hash to get the assetbundle information for
    #[arg(long)]
    pub hash: String,

    /// Part of the URL used to download the info from. Uses the most recent if not provided
    #[arg(long)]
    pub host_hash: Option<String>,

    /// The device platform to get the assetbundle information for
    #[arg(short, long, value_enum, default_value_t = Platform::Android)]
    pub platform: Platform,

    /// The server to get the assetbundle information from
    #[arg(short, long, value_enum, default_value_t = Server::Japan)]
    pub server: Server,

    /// Whether to output status messages
    #[arg(short, long, default_value_t = false)]
    pub quiet: bool,

    /// The directory to output the assetbundle info file to
    pub out_dir: Option<String>,
}

pub async fn abinfo(args: AbInfoArgs) -> Result<(), Error> {
    let show_progress = !args.quiet;

    // create spinner
    let communicate_spinner = if show_progress {
        println!(
            "{}[1/1] {}{}",
            color::TEXT_VARIANT.render_fg(),
            color::TEXT.render_fg(),
            strings::command::COMMUNICATING,
        );
        let spinner = ProgressBar::new_spinner();
        spinner.enable_steady_tick(Duration::from_millis(100));
        Some(spinner)
    } else {
        None
    };

    // get assetbundle info
    let fetch_config = FetchConfig::builder(args.version, args.hash)
        .platform(args.platform)
        .server(args.server)
        .quiet(args.quiet)
        .build();
    let mut fetcher = Fetcher::new(fetch_config).await?;

    let assetbundle_info = fetcher
        .get_ab_info(args.asset_version, args.host_hash)
        .await?;

    // serialize assetbundle info
    let assetbundle_info_serialized = serde_json::to_vec(&assetbundle_info)?;

    // stop previous spinner
    if let Some(spinner) = communicate_spinner {
        spinner.finish_and_clear()
    }

    // determine the directory to save the info file.
    let out_dir = args.out_dir.unwrap_or_default();
    let out_dir_path = Path::new(&out_dir);
    let out_path = out_dir_path.join(Path::new(&format!("{}.json", assetbundle_info.version)));

    // Write assetbundleinfo file.
    if let Some(parent) = out_path.parent() {
        create_dir_all(parent).await?;
    }
    let mut out_file = File::options()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&out_path)
        .await?;
    out_file.write_all(&assetbundle_info_serialized).await?;

    if show_progress {
        println!(
            "{}{}{}{}",
            color::SUCCESS.render_fg(),
            strings::command::PATHS_SAVED_TO,
            out_path.to_str().unwrap_or(""),
            color::TEXT.render_fg()
        )
    }

    Ok(())
}

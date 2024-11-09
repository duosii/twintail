use std::{
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use crate::{
    api::{
        sekai_client::SekaiClient,
        url::{server_provider::ServerUrlProvider, UrlProvider},
    },
    constants::{color, strings},
    crypto::assetbundle,
    error::CommandError,
    models::{
        api::Assetbundle,
        enums::{Platform, Server},
    },
    subcommands::fetch::get_assetbundle_info,
};
use clap::Args;
use futures::{stream, StreamExt};
use humansize::{format_size, DECIMAL};
use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;
use tokio::{
    fs::{create_dir_all, File},
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
};
use tokio_retry::{strategy::FixedInterval, Retry};

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

    /// Path to an assetbundle info file that was output by ``fetch ab-info``
    #[arg(short, long)]
    pub info: Option<String>,

    /// The maximum number of files to download simultaneously
    #[arg(long, short, default_value_t = crate::utils::available_parallelism())]
    pub concurrent: usize,

    /// Only assetbundles that match this regular expression will be downloaded
    #[arg(long, short)]
    pub filter: Option<String>,

    /// The maximum number of times to retry a download if it fails
    #[arg(long, short, default_value_t = 3)]
    pub retry: usize,

    /// If present, the downloaded assetbundles will not be decrypted
    #[arg(long, short, default_value_t = false)]
    pub encrypt: bool,

    /// The directory to output the assetbundles to
    pub out_dir: String,
}

struct AssetbundlePathArgs {
    asset_version: String,
    asset_hash: String,
    host_hash: String,
}

/// Downloads an assetbundle to a provided path.
///
/// If decrypt is false, the downloaded assetbundle will remain encrypted.
async fn download_bundle(
    client: &SekaiClient<ServerUrlProvider>,
    bundle: &Assetbundle,
    out_path: &Path,
    path_args: &AssetbundlePathArgs,
    download_progress: &ProgressBar,
    decrypt: bool,
) -> Result<(), CommandError> {
    // check hash of existing file

    // download
    let mut ab_data = client
        .get_assetbundle(
            &path_args.asset_version,
            &path_args.asset_hash,
            &path_args.host_hash,
            &bundle.bundle_name,
        )
        .await?;

    // decrypt if desired
    if decrypt {
        assetbundle::decrypt_in_place(&mut ab_data).await?;
    }

    // write file
    if let Some(parent) = out_path.parent() {
        create_dir_all(parent).await?;
    }
    let mut out_file = File::options()
        .write(true)
        .create(true)
        .truncate(true)
        .open(out_path)
        .await?;
    out_file.write_all(&ab_data).await?;

    // increment progress
    download_progress.inc(bundle.file_size as u64);
    Ok(())
}

pub async fn fetch_ab(args: AbArgs) -> Result<(), CommandError> {
    let mut client = SekaiClient::new(args.version, args.hash, args.platform, args.server).await?;

    // create assetbundle spinner
    println!(
        "{}[1/] {}{}",
        color::TEXT_VARIANT.render_fg(),
        color::TEXT.render_fg(),
        strings::command::RETRIEVING_AB_INFO,
    );
    let ab_info_spinner = ProgressBar::new_spinner();
    ab_info_spinner.enable_steady_tick(Duration::from_millis(100));

    // get assetbundle info
    let assetbundle_info = if let Some(path) = args.info {
        // read file
        let file = File::open(path).await?;
        let mut reader = BufReader::new(file);
        let mut file_buf = Vec::new();
        reader.read_to_end(&mut file_buf).await?;

        // deserialize
        serde_json::from_slice(&file_buf)?
    } else {
        get_assetbundle_info(&mut client, args.asset_version, args.host_hash).await?
    };

    // stop assetbundle info spinner
    ab_info_spinner.finish();

    // extract data from assetbundle_info
    let ab_path_args = AssetbundlePathArgs {
        asset_version: assetbundle_info.version.clone(),
        asset_hash: assetbundle_info.hash.clone().unwrap_or_default(),
        host_hash: assetbundle_info.host_hash.clone().unwrap_or_default(),
    };

    // convert out_dir to a path.
    let out_dir = Path::new(&args.out_dir);

    // calculate out paths
    let mut total_bundle_size = 0;
    let mut to_download_bundles: Vec<(Assetbundle, PathBuf)> = Vec::new();

    let bundle_name_re = args
        .filter
        .as_ref()
        .and_then(|filter| Regex::new(filter).ok());

    for (_, bundle) in assetbundle_info.bundles {
        if bundle_name_re
            .as_ref()
            .map_or(true, |re| re.find(&bundle.bundle_name).is_some())
        {
            let out_path = out_dir.join(client.url_provider.assetbundle_path(
                &ab_path_args.asset_version,
                &ab_path_args.asset_hash,
                &client.app.platform,
                &bundle.bundle_name,
            ));

            total_bundle_size += bundle.file_size;
            to_download_bundles.push((bundle, out_path));
        }
    }

    if args.filter.is_some() && bundle_name_re.is_none() {
        println!(
            "{}{}{}",
            color::clap::ERROR.render_fg(),
            strings::command::INVALID_RE,
            color::TEXT.render_fg()
        )
    }

    // make sure the out_dir has enough space
    let available_space = fs2::available_space(out_dir)?;
    if total_bundle_size > available_space {
        return Err(CommandError::NotEnoughSpace(format!(
            "this operation requires {} of free space. you only have {} available.",
            format_size(total_bundle_size, DECIMAL),
            format_size(available_space, DECIMAL)
        )));
    }

    // create download progress bar
    println!(
        "{}[2/] {}{}",
        color::TEXT_VARIANT.render_fg(),
        color::TEXT.render_fg(),
        strings::command::DOWNLOADING,
    );
    let download_progress = ProgressBar::new(total_bundle_size).with_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})",
        )
        .unwrap_or(ProgressStyle::default_bar())
        .progress_chars("#-"),
    );

    // download bundles
    let download_start = Instant::now();
    let retry_strat = FixedInterval::from_millis(200).take(args.retry);
    let do_decrypt = !args.encrypt;

    let download_results: Vec<Result<(), CommandError>> = stream::iter(&to_download_bundles)
        .map(|(bundle, out_path)| async {
            Retry::spawn(retry_strat.clone(), || {
                download_bundle(
                    &client,
                    bundle,
                    out_path,
                    &ab_path_args,
                    &download_progress,
                    do_decrypt,
                )
            })
            .await
        })
        .buffer_unordered(args.concurrent)
        .collect()
        .await;
    let success_count = download_results
        .iter()
        .filter(|&result| result.is_ok())
        .count();

    // stop progress bar & print the sucess message
    download_progress.finish_and_clear();
    println!(
        "{}Successfully {} {} / {} files in {:?}{}",
        color::SUCCESS.render_fg(),
        strings::command::DOWNLOADED,
        success_count,
        to_download_bundles.len(),
        Instant::now().duration_since(download_start),
        color::TEXT.render_fg(),
    );

    Ok(())
}

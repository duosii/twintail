use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    time::Instant,
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
        api::{Assetbundle, AssetbundleInfo},
        enums::{Platform, Server},
    },
    subcommands::fetch::get_assetbundle_info,
    utils::{fs::write_file, progress::ProgressBar},
};
use clap::Args;
use futures::{stream, StreamExt};
use humansize::{format_size, DECIMAL};
use regex::Regex;
use tokio::{
    fs::{create_dir_all, File},
    io::{AsyncReadExt, BufReader},
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

    /// Path to an assetbundle info file. If not provided, the latest one will be fetched
    #[arg(short, long)]
    pub info: Option<String>,

    /// If set, the assetbundle info file provided with --info will not be updated to the most recent asset version
    #[arg(long, default_value_t = false)]
    pub no_update: bool,

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

    /// If present, will print debug messages.
    #[arg(long, default_value_t = false)]
    pub debug: bool,

    /// The directory to output the assetbundles to
    pub out_dir: String,
}

#[derive(Debug)]
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
    download_progress: &indicatif::ProgressBar,
    decrypt: bool,
) -> Result<(), CommandError> {
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
    write_file(out_path, &ab_data).await?;

    // increment progress
    download_progress.inc(bundle.file_size);
    Ok(())
}

/// Reads and deserializes an assetbundle info from a .json file.
async fn read_assetbundle_info(path: &str) -> Result<AssetbundleInfo, CommandError> {
    // read file
    let file = File::open(path).await?;
    let mut reader = BufReader::new(file);
    let mut file_buf = Vec::new();
    reader.read_to_end(&mut file_buf).await?;

    // deserialize
    Ok(serde_json::from_slice(&file_buf)?)
}

/// Compares two HashMaps of [crate::models::api::Assetbundle].
///
/// Returns a new HashMap of [crate::models::api::Assetbundle] where
/// a bundle in main_bundles exists in compare_bundles,
/// but has a different hash value.
fn get_assetbundles_differences(
    main_bundles: HashMap<String, Assetbundle>,
    compare_bundles: &HashMap<String, Assetbundle>,
) -> HashMap<String, Assetbundle> {
    main_bundles
        .into_iter()
        .filter(|(bundle_name, bundle)| {
            compare_bundles
                .get(bundle_name)
                .map_or(true, |compare_bundle| compare_bundle.hash != bundle.hash)
        })
        .collect()
}

pub async fn fetch_ab(args: AbArgs) -> Result<(), CommandError> {
    let mut client = SekaiClient::new(args.version, args.hash, args.platform, args.server).await?;

    // create assetbundle spinner
    println!(
        "{}[1/2] {}{}",
        color::TEXT_VARIANT.render_fg(),
        color::TEXT.render_fg(),
        strings::command::RETRIEVING_AB_INFO,
    );
    let ab_info_spinner = ProgressBar::spinner();

    // get assetbundle info
    let assetbundle_info = match args.info {
        None => get_assetbundle_info(&mut client, args.asset_version, args.host_hash).await?,
        Some(string_path) => {
            let assetbundle_info_path = Path::new(&string_path);
            let file_exists = assetbundle_info_path.try_exists().unwrap_or(false);

            if file_exists {
                // read file
                let main_info = read_assetbundle_info(&string_path).await?;

                // update the assetbundle info if it should be updated
                if args.no_update {
                    main_info
                } else {
                    let latest_info =
                        get_assetbundle_info(&mut client, args.asset_version, args.host_hash)
                            .await?;

                    AssetbundleInfo {
                        bundles: get_assetbundles_differences(
                            latest_info.bundles,
                            &main_info.bundles,
                        ),
                        ..latest_info
                    }
                }
            } else {
                let latest_info =
                    get_assetbundle_info(&mut client, args.asset_version, args.host_hash).await?;

                // write to assetbundle_info_file_path
                let serialized = serde_json::to_vec(&latest_info)?;
                write_file(assetbundle_info_path, &serialized).await?;

                latest_info
            }
        }
    };

    // stop assetbundle info spinner
    ab_info_spinner.finish_and_clear();

    // extract data from assetbundle_info
    let ab_path_args = AssetbundlePathArgs {
        asset_version: assetbundle_info.version.clone(),
        asset_hash: assetbundle_info.hash.clone().unwrap_or_default(),
        host_hash: assetbundle_info.host_hash.clone().unwrap_or_default(),
    };

    // convert out_dir to a path.
    let out_dir = Path::new(&args.out_dir);
    create_dir_all(out_dir).await?;

    // calculate out paths
    let mut total_bundle_size = 0;
    let mut to_download_bundles: Vec<(Assetbundle, PathBuf)> = Vec::new();

    let bundle_name_re = args
        .filter
        .as_ref()
        .and_then(|filter| Regex::new(filter).ok());

    for (bundle_name, bundle) in assetbundle_info.bundles {
        if bundle_name_re
            .as_ref()
            .map_or(true, |re| re.find(&bundle_name).is_some())
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
        "{}[2/2] {}{}",
        color::TEXT_VARIANT.render_fg(),
        color::TEXT.render_fg(),
        strings::command::DOWNLOADING,
    );
    let download_progress = ProgressBar::download(total_bundle_size);

    // download bundles
    let download_start = Instant::now();
    let retry_strat = FixedInterval::from_millis(200).take(args.retry);
    let do_decrypt = !args.encrypt;
    let do_debug = args.debug;

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

    // count successes & print errors if debug is enabled
    let success_count = download_results
        .iter()
        .filter(|&result| {
            if let Err(err) = result {
                if do_debug {
                    println!("assetbundle download error: {:?}", err);
                }
                false
            } else {
                true
            }
        })
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

use clap::Args;
use std::path::Path;
use tokio::{
    fs::File,
    io::{AsyncReadExt, BufReader},
};
use twintail::{
    Fetcher,
    config::{OptionalBuilder, download_ab_config::DownloadAbConfig, fetch_config::FetchConfig},
    models::{
        api::AssetbundleInfo,
        enums::{Platform, Server},
    },
};

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

/// Reads and deserializes an assetbundle info from a .json file.
async fn read_assetbundle_info(path: &str) -> Result<AssetbundleInfo, twintail::Error> {
    // read file
    let file = File::open(path).await?;
    let mut reader = BufReader::new(file);
    let mut file_buf = Vec::new();
    reader.read_to_end(&mut file_buf).await?;

    // deserialize
    Ok(serde_json::from_slice(&file_buf)?)
}

pub async fn fetch_ab(args: AbArgs) -> Result<(), twintail::Error> {
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
    let fetch_config = FetchConfig::builder(args.version, args.hash)
        .platform(args.platform)
        .server(args.server)
        .retry(args.retry)
        .decrypt(!args.encrypt)
        .quiet(args.quiet)
        .map(args.concurrent, |config, concurrency| {
            config.concurrency(concurrency)
        })
        .build();

    // create fetcher
    let mut fetcher = Fetcher::new(fetch_config).await?;

    // download assetbundles
    fetcher
        .download_ab(args.out_dir, download_ab_config)
        .await?;

    Ok(())
}

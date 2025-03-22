use clap::Args;
use twintail::{
    config::{fetch_config::FetchConfig, OptionalBuilder},
    models::enums::{Platform, Server},
    Fetcher,
};

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

pub async fn fetch_suite(args: SuiteArgs) -> Result<(), twintail::Error> {
    // create fetcher
    let fetch_config = FetchConfig::builder(args.version, args.hash)
        .platform(args.platform)
        .server(args.server)
        .retry(args.retry)
        .decrypt(!args.encrypt)
        .quiet(args.quiet)
        .pretty_json(!args.compact)
        .map(args.concurrent, |config, concurrency| {
            config.concurrency(concurrency)
        })
        .build();
    let mut fetcher = Fetcher::new(fetch_config).await?;

    // download suitemaster files
    fetcher.download_suite(args.out_path).await?;

    Ok(())
}

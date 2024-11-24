use clap::Args;
use twintail::{
    config::{crypt_config::CryptConfig, OptionalBuilder},
    enums::Server,
    Encrypter,
};

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

    /// The total number of files to split the suitemaster files into
    #[arg(short, long, default_value_t = 6)]
    pub split: usize,

    /// Whether to output status messages
    #[arg(short, long, default_value_t = false)]
    pub quiet: bool,

    /// Path to the file or directory to encrypt
    pub in_path: String,

    /// Path to a directory or file to output to.
    pub out_path: String,
}

pub async fn encrypt_suite(args: EncryptSuiteArgs) -> Result<(), twintail::Error> {
    let config = CryptConfig::builder()
        .recursive(args.recursive)
        .server(args.server)
        .quiet(args.quiet)
        .map(args.concurrent, |config, concurrency| {
            config.concurrency(concurrency)
        })
        .build();

    let encrypter = Encrypter::new(config);

    encrypter
        .encrypt_suite_path(args.in_path, args.out_path, args.split)
        .await?;

    Ok(())
}

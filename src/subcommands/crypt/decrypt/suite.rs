use clap::Args;
use twintail::{
    config::{crypt_config::CryptConfig, OptionalBuilder},
    enums::Server,
    Decrypter,
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

    /// Path to the file or directory to decrypt
    pub in_path: String,

    /// Path to a directory or file to output to.
    pub out_path: String,
}

/// Decrypts encrypted suitemaster files into individual .json files.
pub async fn decrypt_suite(args: DecryptSuiteArgs) -> Result<(), twintail::Error> {
    let config = CryptConfig::builder()
        .recursive(args.recursive)
        .server(args.server)
        .quiet(args.quiet)
        .map(args.concurrent, |config, concurrency| {
            config.concurrency(concurrency)
        })
        .build();

    let decrypter = Decrypter::new(config);

    decrypter
        .decrypt_suite_path(args.in_path, args.out_path)
        .await?;

    Ok(())
}

use clap::Args;
use twintail::{
    config::{crypt_config::CryptConfig, OptionalBuilder},
    Encrypter,
};

#[derive(Debug, Args)]
pub struct EncryptAbArgs {
    /// If the input is a directory, whether to recursively encrypt valid files in that directory
    #[arg(long, short, default_value_t = false)]
    pub recursive: bool,

    /// The maximum number of files to encrypt simultaneously
    #[arg(long, short)]
    pub concurrent: Option<usize>,

    /// Whether to output status messages
    #[arg(short, long, default_value_t = false)]
    pub quiet: bool,

    /// Path to the file or directory to encrypt
    pub in_path: String,

    /// Path to a directory or file to output to. If not provided, files are encrypted in-place.
    pub out_path: Option<String>,
}

/// Encrypts a file/folder using the provided arguments.
pub async fn encrypt_ab(args: EncryptAbArgs) -> Result<(), twintail::Error> {
    let config = CryptConfig::builder()
        .recursive(args.recursive)
        .quiet(args.quiet)
        .map(args.concurrent, |config, concurrency| {
            config.concurrency(concurrency)
        })
        .build();

    let encrypter = Encrypter::new(config);

    encrypter
        .encrypt_ab_path(args.in_path, args.out_path)
        .await?;

    Ok(())
}

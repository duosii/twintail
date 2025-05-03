use crate::Error;
use clap::Args;
use twintail_core::{
    config::{OptionalBuilder, crypt_config::CryptConfig},
    decrypt::Decrypter,
};

#[derive(Debug, Args)]
pub struct DecryptAbArgs {
    /// If the input is a directory, whether to recursively decrypt valid files in that directory
    #[arg(long, short, default_value_t = false)]
    pub recursive: bool,

    /// The maximum number of files to decrypt simultaneously
    #[arg(long, short)]
    pub concurrent: Option<usize>,

    /// Whether to output status messages
    #[arg(short, long, default_value_t = false)]
    pub quiet: bool,

    /// Path to the file or directory to decrypt
    pub in_path: String,

    /// Path to a directory or file to output to. If not provided, files are decrypted in-place
    pub out_path: Option<String>,
}

/// Decrypts a file/folder using the provided arguments.
pub async fn decrypt_ab(args: DecryptAbArgs) -> Result<(), Error> {
    let config = CryptConfig::builder()
        .recursive(args.recursive)
        .quiet(args.quiet)
        .map(args.concurrent, |config, val| config.concurrency(val))
        .build();

    let decrypter = Decrypter::new(config);

    decrypter
        .decrypt_ab_path(args.in_path, args.out_path)
        .await?;

    Ok(())
}

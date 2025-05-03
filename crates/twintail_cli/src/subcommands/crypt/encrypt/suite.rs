use clap::Args;
use tokio::time::Instant;
use twintail_common::{color, models::enums::Server};
use twintail_core::{
    config::{OptionalBuilder, crypt_config::CryptConfig},
    encrypt::Encrypter,
};

use crate::{Error, strings};

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

    /// The number of files to split the encrypted suitemaster files into
    #[arg(long, default_value_t = 7)]
    pub split: usize,

    /// Whether to output status messages
    #[arg(short, long, default_value_t = false)]
    pub quiet: bool,

    /// Path to the file or directory to encrypt
    pub in_path: String,

    /// Path to a directory or file to output to.
    pub out_path: String,
}

pub async fn encrypt_suite(args: EncryptSuiteArgs) -> Result<(), Error> {
    let encrypt_start = Instant::now();

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

    if !args.quiet {
        println!(
            "{}Successfully {} suite master files in {:?}.{}",
            color::SUCCESS.render_fg(),
            strings::crypto::encrypt::PROCESSED,
            Instant::now().duration_since(encrypt_start),
            color::TEXT.render_fg(),
        );
    }

    Ok(())
}

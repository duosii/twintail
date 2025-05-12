use crate::{Error, strings};
use clap::Args;
use tokio::time::Instant;
use twintail_common::{color, models::enums::Server};
use twintail_core::{config::crypt_config::CryptConfig, encrypt::Encrypter};

#[derive(Debug, Args)]
pub struct EncryptJsonArgs {
    /// Whether to output status messages
    #[arg(short, long, default_value_t = false)]
    pub quiet: bool,

    /// The server to encrypt the suitemasterfiles for
    #[arg(short, long, value_enum, default_value_t = Server::Japan)]
    pub server: Server,

    /// Path to a JSON file
    pub in_path: String,

    /// Path to file to output to. If not provided, files are encrypted in-place.
    pub out_path: Option<String>,
}

/// Encrypts a file/folder using the provided arguments.
pub async fn encrypt_json(args: EncryptJsonArgs) -> Result<(), Error> {
    let quiet = args.quiet;

    let config = CryptConfig::builder()
        .server(args.server)
        .quiet(quiet)
        .build();

    let (encrypter, _) = Encrypter::new(config);

    let in_path = args.in_path;
    let out_path = args.out_path.unwrap_or(in_path.clone());

    let encrypt_start = Instant::now();

    if !quiet {
        println!(
            "{}{} json file...{}",
            color::SUCCESS.render_fg(),
            strings::crypto::encrypt::PROCESS,
            color::TEXT.render_fg(),
        );
    }

    encrypter
        .encrypt_file_aes_msgpack(in_path, out_path)
        .await?;

    if !quiet {
        println!(
            "{}Successfully {} json file in {:?}.{}",
            color::SUCCESS.render_fg(),
            strings::crypto::encrypt::PROCESSED,
            Instant::now().duration_since(encrypt_start),
            color::TEXT.render_fg(),
        );
    }

    Ok(())
}

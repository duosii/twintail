mod api;
mod constants;
mod crypto;
mod error;
mod models;
mod subcommands;
mod utils;

use clap::{Parser, Subcommand};
use constants::color;
use error::Error;
use subcommands::{
    crypt::{decrypt, encrypt},
    fetch,
};

#[derive(Debug, Subcommand)]
enum Commands {
    /// Fetches game assetbundle files from the game's official servers
    Fetch(fetch::FetchArgs),
    /// Decrypts the game's assetbundle files
    Decrypt(decrypt::DecryptArgs),
    /// Encrypts Unity assetbundle files
    Encrypt(encrypt::EncryptArgs),
}

#[derive(Debug, Parser)]
#[command(version, about, long_about = None, styles=utils::styles::get_clap_styles())]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

// cargo run fetch --version 4.0.5 --hash 2179da72-9de5-23a6-f388-9e5835098ce1 /assets
#[tokio::main]
async fn main() -> Result<(), Error> {
    //println!("{}NO!", anstyle::RgbColor(255, 200, 255).render_fg());
    let cli = Cli::parse();

    match cli.command {
        Commands::Fetch(args) => {
            if let Err(err) = fetch::fetch(args).await {
                panic!(
                    "{}{}{}",
                    color::clap::ERROR.render_fg(),
                    err,
                    color::TEXT.render_fg()
                )
            }
        }
        Commands::Decrypt(args) => {
            decrypt::decrypt(&args).await?;
        }
        Commands::Encrypt(args) => {
            encrypt::encrypt(&args).await?;
        }
    }

    Ok(())
}

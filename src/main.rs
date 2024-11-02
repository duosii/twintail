mod api;
mod constants;
mod crypto;
mod error;
mod models;
mod subcommands;
mod utils;

use clap::{Parser, Subcommand};
use error::Error;
use subcommands::{decrypt, encrypt, fetch};

#[derive(Debug, Subcommand)]
enum Commands {
    /// Fetches game assetbundle files from the game's official servers.
    Fetch(fetch::FetchArgs),
    /// Decrypts the game's assetbundle files.
    Decrypt(decrypt::DecryptArgs),
    /// Encrypts unity assetbundle files.
    Encrypt(encrypt::EncryptArgs),
}

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

// cargo run fetch --version 4.0.5 --hash 2179da72-9de5-23a6-f388-9e5835098ce1 /assets
#[tokio::main]
async fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Fetch(args) => {
            fetch::fetch(args).await?;
        }
        Commands::Decrypt(args) => {
            decrypt::decrypt(args).await?;
        }
        Commands::Encrypt(args) => {
            encrypt::encrypt(args).await?;
        }
    }

    Ok(())
}

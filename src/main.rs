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
    /// Commands that fetch assets from the game
    Fetch(fetch::FetchArgs),
    /// Commands that decrypt files related to the game
    Decrypt(decrypt::DecryptArgs),
    /// Commands that encrypt files related to the game
    Encrypt(encrypt::EncryptArgs),
}

#[derive(Debug, Parser)]
#[command(version, about, long_about = None, styles=utils::styles::get_clap_styles())]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Fetch(args) => fetch::fetch(args).await,
        Commands::Decrypt(args) => decrypt::decrypt(args).await,
        Commands::Encrypt(args) => encrypt::encrypt(args).await,
    };

    // print error if result is an error
    if let Err(err) = result {
        println!(
            "{}{}{}",
            color::clap::ERROR.render_fg(),
            err,
            color::TEXT.render_fg()
        )
    }

    Ok(())
}

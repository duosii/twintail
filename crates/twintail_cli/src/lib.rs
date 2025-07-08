mod color;
mod error;
mod progress;
mod strings;
mod subcommands;

use color::get_clap_styles;
pub use error::Error;

use clap::{Parser, Subcommand};
use subcommands::{
    app_info,
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
    /// Extract app version & hash from an apk file
    AppInfo(app_info::AppInfoArgs),
}

#[derive(Debug, Parser)]
#[command(version, about, long_about = None, styles=get_clap_styles())]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Runs the twintail CLI.
pub async fn run() -> Result<(), clap::Error> {
    let cli = Cli::try_parse()?;

    let command_result = match cli.command {
        Commands::Fetch(args) => fetch::fetch(args).await,
        Commands::Decrypt(args) => decrypt::decrypt(args).await,
        Commands::Encrypt(args) => encrypt::encrypt(args).await,
        Commands::AppInfo(args) => app_info::app_info(args),
    };

    // print error if result is an error
    if let Err(err) = command_result {
        println!(
            "{}{}{}",
            color::ERROR.render_fg(),
            err,
            color::TEXT.render_fg()
        )
    }

    Ok(())
}

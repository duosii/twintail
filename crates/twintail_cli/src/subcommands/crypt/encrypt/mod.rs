pub mod ab;
pub mod json;
pub mod suite;

use crate::Error;
use ab::EncryptAbArgs;
use clap::{Args, Subcommand};
use json::EncryptJsonArgs;
use suite::EncryptSuiteArgs;

#[derive(Debug, Subcommand)]
enum Commands {
    /// Encrypt assetbundles.
    Ab(EncryptAbArgs),
    /// Encrypt suitemaster .json files.
    Suite(EncryptSuiteArgs),
    /// Encrypt .json files.
    Json(EncryptJsonArgs),
}

#[derive(Debug, Args)]
pub struct EncryptArgs {
    #[command(subcommand)]
    command: Commands,
}

/// Command handler for the decrypt subcommand.
pub async fn encrypt(args: EncryptArgs) -> Result<(), Error> {
    match args.command {
        Commands::Ab(args) => ab::encrypt_ab(args).await,
        Commands::Suite(args) => suite::encrypt_suite(args).await,
        Commands::Json(args) => json::encrypt_json(args).await,
    }
}

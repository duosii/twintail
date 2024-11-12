pub mod ab;
pub mod suite;

use ab::EncryptAbArgs;
use clap::{Args, Subcommand};
use suite::EncryptSuiteArgs;

use crate::error::CommandError;

#[derive(Debug, Subcommand)]
enum Commands {
    /// Encrypt assetbundles.
    Ab(EncryptAbArgs),
    /// Encrypt suitemaster .json files.
    Suite(EncryptSuiteArgs),
}

#[derive(Debug, Args)]
pub struct EncryptArgs {
    #[command(subcommand)]
    command: Commands,
}

/// Command handler for the decrypt subcommand.
pub async fn encrypt(args: EncryptArgs) -> Result<(), CommandError> {
    match args.command {
        Commands::Ab(args) => ab::encrypt_ab(&args).await,
        Commands::Suite(args) => suite::encrypt_suite(args).await,
    }
}

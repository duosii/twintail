pub mod ab;
pub mod suite;
pub mod json;

use ab::DecryptAbArgs;
use clap::{Args, Subcommand};
use json::DecryptJsonArgs;
use suite::DecryptSuiteArgs;

#[derive(Debug, Subcommand)]
enum Commands {
    /// Decrypt assetbundles
    Ab(DecryptAbArgs),
    /// Decrypt suitemaster files
    Suite(DecryptSuiteArgs),
    /// Decrypt encrypted JSON files
    Json(DecryptJsonArgs)
}

#[derive(Debug, Args)]
pub struct DecryptArgs {
    #[command(subcommand)]
    command: Commands,
}

/// Command handler for the decrypt subcommand.
pub async fn decrypt(args: DecryptArgs) -> Result<(), twintail::Error> {
    match args.command {
        Commands::Ab(args) => ab::decrypt_ab(args).await,
        Commands::Suite(args) => suite::decrypt_suite(args).await,
        Commands::Json(args) => json::decrypt_json(args).await
    }
}

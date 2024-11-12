pub mod ab;

use ab::DecryptAbArgs;
use clap::{Args, Subcommand};

use crate::error::CommandError;

#[derive(Debug, Subcommand)]
enum Commands {
    /// Decrypt assetbundles.
    Ab(DecryptAbArgs),
}

#[derive(Debug, Args)]
pub struct DecryptArgs {
    #[command(subcommand)]
    command: Commands,
}

/// Command handler for the decrypt subcommand.
pub async fn decrypt(args: DecryptArgs) -> Result<(), CommandError> {
    match args.command {
        Commands::Ab(args) => ab::decrypt_ab(&args).await,
    }
}

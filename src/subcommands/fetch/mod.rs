pub mod ab;
pub mod abinfo;
pub mod suite;

use ab::AbArgs;
use abinfo::AbInfoArgs;
use clap::{Args, Subcommand};
use suite::SuiteArgs;

#[derive(Debug, Subcommand)]
enum Commands {
    /// Fetch assetbundles.
    Ab(AbArgs),
    /// Fetch what assetbundles are available for download.
    AbInfo(AbInfoArgs),
    Suite(SuiteArgs),
}

#[derive(Debug, Args)]
pub struct FetchArgs {
    #[command(subcommand)]
    command: Commands,
}

pub async fn fetch(fetch_args: FetchArgs) -> Result<(), twintail::Error> {
    match fetch_args.command {
        Commands::AbInfo(args) => abinfo::abinfo(args).await,
        Commands::Ab(args) => ab::fetch_ab(args).await,
        Commands::Suite(args) => suite::fetch_suite(args).await,
    }
}

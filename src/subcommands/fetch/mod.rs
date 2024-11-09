pub mod ab;
pub mod abinfo;

use ab::AbArgs;
use abinfo::AbInfoArgs;
use clap::{Args, Subcommand};

use crate::{
    api::{sekai_client::SekaiClient, url::UrlProvider},
    constants::strings,
    error::CommandError,
    models::api::AssetbundleInfo,
};

#[derive(Debug, Subcommand)]
enum Commands {
    /// Fetch assetbundles.
    Ab(AbArgs),
    /// Fetch what assetbundles are available for download.
    AbInfo(AbInfoArgs),
}

#[derive(Debug, Args)]
pub struct FetchArgs {
    #[command(subcommand)]
    command: Commands,
}

/// Gets assetbundle info from the provided SekaiClient.
///
/// If asset_version or host_hash are not provided, their most recent values will be used.
pub async fn get_assetbundle_info<T: UrlProvider>(
    client: &mut SekaiClient<T>,
    asset_version: Option<String>,
    host_hash: Option<String>,
) -> Result<AssetbundleInfo, CommandError> {
    // get asset hash only if we got the most recent versions of the asset_version & host_hash
    let asset_hash = if asset_version.is_none() && host_hash.is_none() {
        let user_signup = client.user_signup().await?;
        let user_auth_response = client
            .user_login(
                user_signup.user_registration.user_id,
                user_signup.credential,
            )
            .await?;
        Some(user_auth_response.asset_hash)
    } else {
        None
    };

    // get the assetbundle host hash
    let host_hash = if let Some(host_hash) = host_hash {
        host_hash
    } else {
        client.get_game_version().await?.assetbundle_host_hash
    };

    // get system information
    let asset_version = if let Some(version) = asset_version {
        Ok(version)
    } else {
        let system_info = client.get_system().await?;
        if let Some(most_recent_version) = system_info.app_versions.last() {
            Ok(most_recent_version.asset_version.clone())
        } else {
            Err(CommandError::NotFound(
                strings::command::error::NO_RECENT_VERSION.to_string(),
            ))
        }
    }?;

    // get the assetbundle info
    let mut assetbundle_info = client
        .get_assetbundle_info(&asset_version, &host_hash)
        .await?;
    assetbundle_info.host_hash = Some(host_hash.to_string());
    assetbundle_info.hash = asset_hash;

    Ok(assetbundle_info)
}

pub async fn fetch(fetch_args: FetchArgs) -> Result<(), CommandError> {
    match fetch_args.command {
        Commands::AbInfo(args) => abinfo::abinfo(args).await?,
        Commands::Ab(args) => ab::fetch_ab(args).await?,
    }

    Ok(())
}

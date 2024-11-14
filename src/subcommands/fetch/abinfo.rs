use std::path::Path;

use clap::Args;

use crate::{
    api::sekai_client::SekaiClient,
    constants::{color, strings},
    error::CommandError,
    models::enums::{Platform, Server},
    subcommands::fetch::get_assetbundle_info,
    utils::{fs::write_file, progress::ProgressBar},
};

#[derive(Debug, Args)]
pub struct AbInfoArgs {
    /// The version of the game app get the assetbundle information for
    #[arg(short, long)]
    pub version: String,

    /// The version of the assets to get information about. Uses the most recent if not provided
    #[arg(short, long)]
    pub asset_version: Option<String>,

    /// The app hash to get the assetbundle information for
    #[arg(long)]
    pub hash: String,

    /// Part of the URL used to download the info from. Uses the most recent if not provided
    #[arg(long)]
    pub host_hash: Option<String>,

    /// The device platform to get the assetbundle information for
    #[arg(short, long, value_enum, default_value_t = Platform::Android)]
    pub platform: Platform,

    /// The server to get the assetbundle information from
    #[arg(short, long, value_enum, default_value_t = Server::Japan)]
    pub server: Server,

    /// The directory to output the assetbundle info file to
    pub out_dir: Option<String>,
}

pub async fn abinfo(args: AbInfoArgs) -> Result<(), CommandError> {
    // create spinner
    println!(
        "{}[1/1] {}{}",
        color::TEXT_VARIANT.render_fg(),
        color::TEXT.render_fg(),
        strings::command::COMMUNICATING,
    );
    let communicate_spinner = ProgressBar::spinner();

    let mut client = SekaiClient::new(args.version, args.hash, args.platform, args.server).await?;

    // get assetbundle info
    let assetbundle_info =
        get_assetbundle_info(&mut client, args.asset_version, args.host_hash).await?;

    // serialize assetbundle info
    let assetbundle_info_serialized = serde_json::to_vec(&assetbundle_info)?;

    // stop previous spinner
    communicate_spinner.finish();

    // determine the directory to save the info file.
    let out_dir = args.out_dir.unwrap_or_default();
    let out_dir_path = Path::new(&out_dir);
    let out_path = out_dir_path.join(Path::new(&format!("{}.json", assetbundle_info.version)));

    // create parent folders if they do not exist
    write_file(&out_path, &assetbundle_info_serialized).await?;

    println!(
        "{}{}{}{}",
        color::SUCCESS.render_fg(),
        strings::command::PATHS_SAVED_TO,
        out_path.to_str().unwrap_or(""),
        color::TEXT.render_fg()
    );

    Ok(())
}

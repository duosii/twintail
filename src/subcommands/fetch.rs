use clap::Args;

use crate::{
    api::sekai_client::SekaiClient,
    error::CommandError,
    models::{api::UserSignup, enums::Platform},
};

#[derive(Debug, Args)]
pub struct FetchArgs {
    /// The version of the app to download assets from
    #[arg(short, long)]
    pub version: String,

    /// The app hash to use when downloading assets
    #[arg(long)]
    pub hash: String,

    /// The device platform to download the assets for
    #[arg(short, long, value_enum, default_value_t = Platform::Android)]
    pub platform: Platform,

    /// The directory to output the downloaded files to
    pub out_dir: String,
}

pub async fn fetch(args: FetchArgs) -> Result<(), CommandError> {
    let mut sekai_client = SekaiClient::new(args.version, args.hash, args.platform)?;

    // get cloudfront cookies
    sekai_client.issue_signature().await?;

    // get the assetbundleHostHash from the game-version api
    let game_version = sekai_client.get_game_version().await?;

    // create a user account
    let user_sign_up: UserSignup = sekai_client.user_signup().await?;

    // authorize user account
    let user_auth_response = sekai_client
        .user_login(
            user_sign_up.user_registration.user_id,
            user_sign_up.credential,
        )
        .await?;

    println!("{:?}", user_auth_response);

    Ok(())
}

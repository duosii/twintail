use clap::Args;

use crate::{
    api::headers::Headers,
    constants::{header, url},
    crypto::aes_msgpack,
    error::ApiError,
    models::{
        api::{GameVersion, UserAuthRequest, UserAuthResponse, UserRequest, UserResponse},
        enums::Platform,
    },
};

#[derive(Debug, Args)]
pub struct FetchArgs {
    /// The version of the app to download assets from
    #[arg(short, long)]
    pub version: String,

    /// The app hash to use when downloading assets
    #[arg(long)]
    pub hash: String,

    /// The device platform to download the assets for.
    #[arg(short, long, value_enum, default_value_t=Platform::Android)]
    pub platform: Platform,

    /// The directory to output the downloaded files to
    pub out_dir: String,
}

pub async fn fetch(args: &FetchArgs) -> Result<(), ApiError> {
    let app_version = &args.version;
    let app_hash = &args.hash;
    let platform = &args.platform;
    let client = reqwest::Client::new();

    // get headers
    let mut headers = Headers::builder()?
        .version(app_version)
        .hash(app_hash)
        .platform(platform)
        .build()?;

    // get cloudfront cookies
    let mut issue_signature_response = client
        .post(url::sekai::ISSUE_SIGNATURE)
        .body(b"ffa3bd6214f33fe73cb72fee2262bedb".to_vec())
        .headers(headers.clone_inner())
        .send()
        .await?;

    // set the cookie that is inside of issue_signature_response
    let set_cookie_header = issue_signature_response
        .headers_mut()
        .remove(header::name::SET_COOKIE)
        .ok_or(ApiError::NotFound("set-cookie header not found.".into()))?;
    headers.insert(header::name::COOKIE, set_cookie_header);

    // get the assetbundleHostHash from the game-version api
    let game_version_response = client
        .get(format!(
            "{}/{}/{}",
            url::sekai::GAME_VERSION,
            app_version,
            app_hash
        ))
        .headers(headers.clone_inner())
        .send()
        .await?;

    let game_version: GameVersion = aes_msgpack::from_slice(&game_version_response.bytes().await?)?;

    // create a user account
    let user_request_body = aes_msgpack::into_vec(&UserRequest {
        platform: platform.clone(),
        device_model: "samsung SM-T860".into(), //header::value::DEVICE_MODEL.into(),
        operating_system: "Android OS 12 / API-32 (SP2A.220305.013/T860XXS5DWH1)".into(),
    })?;

    let user_response = client
        .post(url::sekai::USER)
        .headers(headers.clone_inner())
        .body(user_request_body)
        .send()
        .await?;

    let user_response: UserResponse = aes_msgpack::from_slice(&user_response.bytes().await?)?;

    // authorize user account
    let user_auth_request_body = aes_msgpack::into_vec(&UserAuthRequest {
        credential: user_response.credential,
        deviceId: None,
    })?;
    let user_auth_response = client
        .put(url::sekai::user_auth(
            user_response.user_registration.user_id,
        ))
        .headers(headers.clone_inner())
        .body(user_auth_request_body)
        .send()
        .await?;

    let user_auth_response: UserAuthResponse =
        aes_msgpack::from_slice(&user_auth_response.bytes().await?)?;

    println!("{:?}", user_auth_response);

    Ok(())
}

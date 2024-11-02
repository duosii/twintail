use serde::{Deserialize, Serialize};

use super::enums::Platform;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GameVersion {
    pub profile: String,
    pub assetbundle_host_hash: String,
    pub domain: String,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UserRegistration {
    pub user_id: usize,
    pub signature: String,
    pub platform: Platform,
    pub device_model: String,
    pub operating_system: String,
    pub registered_at: usize,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UserRequest {
    pub platform: Platform,
    pub device_model: String,
    pub operating_system: String,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UserResponse {
    pub user_registration: UserRegistration,
    pub credential: String,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UserAuthRequest {
    pub credential: String,
    pub deviceId: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UserAuthResponse {
    session_token: String,
    app_version: String,
    multi_play_version: String,
    data_version: String,
    asset_version: String,
    remove_asset_version: String,
    asset_hash: String,
    app_version_status: String,
    is_streaming_virtual_live_force_open_user: bool,
    suite_master_split_path: Vec<String>,
}

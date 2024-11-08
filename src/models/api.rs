use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::enums::{AssetbundleCategory, Platform};

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
pub struct UserSignup {
    pub user_registration: UserRegistration,
    pub credential: String,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UserAuthRequest {
    pub credential: String,
    pub device_id: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UserAuthResponse {
    pub session_token: String,
    pub app_version: String,
    pub multi_play_version: String,
    pub data_version: String,
    pub asset_version: String,
    pub remove_asset_version: String,
    pub asset_hash: String,
    pub app_version_status: String,
    pub is_streaming_virtual_live_force_open_user: bool,
    pub suite_master_split_path: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Assetbundle {
    pub bundle_name: String,
    pub cache_file_name: String,
    pub cache_directory_name: String,
    pub hash: String,
    pub category: AssetbundleCategory,
    pub crc: usize,
    pub file_size: usize,
    pub dependencies: Vec<String>,
    pub paths: Vec<String>,
    pub is_builtin: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AssetbundleInfo {
    pub version: String,
    pub os: String,
    pub hash: Option<String>,
    pub host_hash: Option<String>,
    pub bundles: HashMap<String, Assetbundle>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AppVersion {
    pub system_profile: String,
    pub app_version: String,
    pub multi_play_version: String,
    pub asset_version: String,
    pub app_version_status: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfo {
    pub server_date: usize,
    pub timezone: String,
    pub profile: String,
    pub maintenance_status: String,
    pub app_versions: Vec<AppVersion>,
}

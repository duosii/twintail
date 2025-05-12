use twintail_common::models::enums::Platform;

pub mod global_provider;
pub mod japan_provider;
pub mod server_provider;

mod urls;

#[cfg(test)]
pub mod test_provider;

/// Stores the hosts that a urlsClient should use when making requests.
#[derive(Clone)]
pub struct SekaiHosts {
    issue: String,
    game_version: String,
    game: String,
}

impl SekaiHosts {
    /// Get a new urlsHosts using hostnames from the Japan server
    fn japan() -> Self {
        Self {
            issue: urls::issue::JAPAN_HOST.to_string(),
            game_version: urls::game_version::JAPAN_HOST.to_string(),
            game: urls::game::JAPAN_HOST.to_string(),
        }
    }

    /// Get a new urlsHosts using hostnames from the global server
    fn global() -> Self {
        Self {
            issue: String::default(),
            game_version: urls::game_version::GLOBAL_HOST.to_string(),
            game: urls::game::GLOBAL_HOST.to_string(),
        }
    }
}

/// Trait that provides urls for game endpoints.
pub trait UrlProvider: Clone {
    fn issue_signature(&self) -> Option<String>;
    fn game_version(&self, version: &str, hash: &str) -> String;
    fn user(&self) -> String;
    fn system(&self) -> String;
    fn user_auth(&self, user_id: usize) -> String;
    fn assetbundle_info(
        &self,
        host_hash: &str,
        asset_version: &str,
        asset_hash: &str,
        platform: &Platform,
    ) -> String;
    fn assetbundle(&self, host_hash: &str, assetbundle_path: &str) -> String;
    fn assetbundle_path(
        &self,
        asset_version: &str,
        asset_hash: &str,
        platform: &Platform,
        bundle_name: &str,
    ) -> String;
    fn suitemasterfile(&self, file_path: &str) -> String;
    fn inherit(&self, inherit_id: &str, execute: bool) -> String;
    fn user_suite(&self, user_id: usize) -> String;
}

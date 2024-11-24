use super::{SekaiHosts, UrlProvider};
use crate::{constants::url::sekai, models::enums::Platform};

#[derive(Clone)]
pub struct JapanUrlProvider {
    hosts: SekaiHosts,
}

impl Default for JapanUrlProvider {
    fn default() -> Self {
        Self {
            hosts: SekaiHosts::japan(),
        }
    }
}

impl UrlProvider for JapanUrlProvider {
    fn issue_signature(&self) -> Option<String> {
        Some(format!("{}{}", self.hosts.issue, sekai::issue::SIGNATURE))
    }

    fn game_version(&self, version: &str, hash: &str) -> String {
        format!("{}/{}/{}", self.hosts.game_version, version, hash)
    }

    fn user(&self) -> String {
        format!("{}{}", self.hosts.game, sekai::game::USER)
    }

    fn system(&self) -> String {
        format!("{}{}", self.hosts.game, sekai::game::SYSTEM)
    }

    fn user_auth(&self, user_id: usize) -> String {
        format!(
            "{}{}/{}/auth?refreshUpdatedResources=False",
            self.hosts.game,
            sekai::game::USER_AUTH,
            user_id
        )
    }

    fn assetbundle_info(
        &self,
        host_hash: &str,
        asset_version: &str,
        platform: &Platform,
    ) -> String {
        format!(
            "https://production-{}-assetbundle-info.sekai.colorfulpalette.org{}/{}/os/{}",
            host_hash,
            sekai::assetbundle::INFO,
            asset_version,
            platform.to_string()
        )
    }

    fn assetbundle(&self, host_hash: &str, assetbundle_path: &str) -> String {
        format!(
            "https://production-{}-assetbundle.sekai.colorfulpalette.org/{}",
            host_hash, assetbundle_path
        )
    }

    fn assetbundle_path(
        &self,
        asset_version: &str,
        asset_hash: &str,
        platform: &Platform,
        bundle_name: &str,
    ) -> String {
        format!(
            "{}/{}/{}/{}",
            asset_version,
            asset_hash,
            platform.to_string(),
            bundle_name
        )
    }

    fn suitemasterfile(&self, file_path: &str) -> String {
        format!("{}{}/{}", self.hosts.game, sekai::game::API, file_path)
    }
}

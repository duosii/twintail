use super::{SekaiHosts, UrlProvider};
use crate::url::urls;
use twintail_common::models::enums::Platform;

#[derive(Clone)]
pub struct GlobalUrlProvider {
    hosts: SekaiHosts,
}

impl Default for GlobalUrlProvider {
    fn default() -> Self {
        Self {
            hosts: SekaiHosts::global(),
        }
    }
}

impl UrlProvider for GlobalUrlProvider {
    fn issue_signature(&self) -> Option<String> {
        None
    }

    fn game_version(&self, version: &str, hash: &str) -> String {
        format!("{}/{}/{}", self.hosts.game_version, version, hash)
    }

    fn user(&self) -> String {
        format!("{}{}", self.hosts.game, urls::game::USER)
    }

    fn system(&self) -> String {
        format!("{}{}", self.hosts.game, urls::game::SYSTEM)
    }

    fn user_auth(&self, user_id: usize) -> String {
        format!(
            "{}{}/{}/auth?refreshUpdatedResources=False",
            self.hosts.game,
            urls::game::USER_AUTH,
            user_id
        )
    }

    fn assetbundle_info(
        &self,
        _: &str,
        asset_version: &str,
        _: &str,
        platform: &Platform,
    ) -> String {
        format!(
            "https://assetbundle-info.urls-en.com{}/{}/os/{}",
            urls::assetbundle::INFO,
            asset_version,
            platform.to_string()
        )
    }

    fn assetbundle(&self, _: &str, assetbundle_path: &str) -> String {
        format!("https://assetbundle.urls-en.com/{}", assetbundle_path)
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
        format!("{}{}/{}", self.hosts.game, urls::game::API, file_path)
    }

    fn inherit(&self, inherit_id: &str, execute: bool) -> String {
        format!(
            "{}{}/{}?isExecuteInherit={}&isAdult=True&tAge=16",
            self.hosts.game,
            urls::game::INHERIT,
            inherit_id,
            if execute { "True" } else { "False" }
        )
    }

    fn user_suite(&self, user_id: usize) -> String {
        format!("{}{}/{}", self.hosts.game, urls::game::USER_SUITE, user_id)
    }
}

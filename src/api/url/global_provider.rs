use super::{SekaiHosts, UrlProvider};
use crate::{constants::url::sekai, models::enums::Platform};

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

    fn assetbundle_info(&self, _: &str, asset_version: &str, platform: &Platform) -> String {
        format!(
            "https://assetbundle-info.sekai-en.com{}/{}/os/{}",
            sekai::assetbundle::INFO,
            asset_version,
            platform.to_string()
        )
    }

    fn assetbundle(&self, _: &str, assetbundle_path: &str) -> String {
        format!("https://assetbundle.sekai-en.com/{}", assetbundle_path)
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

    fn inherit(&self, inherit_id: &str, execute: bool) -> String {
        format!(
            "{}{}/{}?isExecuteInherit={}&isAdult=True&tAge=16",
            self.hosts.game,
            sekai::game::INHERIT,
            inherit_id,
            if execute { "True" } else { "False" }
        )
    }

    fn user_suite(&self, user_id: usize) -> String {
        format!("{}{}/{}", self.hosts.game, sekai::game::USER_SUITE, user_id)
    }
}

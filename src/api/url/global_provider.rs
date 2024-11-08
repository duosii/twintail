use super::{SekaiHosts, UrlProvider};
use crate::{constants::url::sekai, models::enums::Platform};

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
}

use crate::{constants::url::sekai, models::enums::Platform};

/// Stores the hosts that a SekaiClient should use when making requests.
pub struct SekaiHosts {
    issue: String,
    game_version: String,
    game: String,
}

impl Default for SekaiHosts {
    fn default() -> Self {
        Self {
            issue: sekai::issue::HOST.to_string(),
            game_version: sekai::game_version::HOST.to_string(),
            game: sekai::game::HOST.to_string(),
        }
    }
}

/// Trait that provides urls for game endpoints.
pub trait SekaiUrlProvider {
    fn issue_signature(&self) -> String;
    fn game_version(&self, version: &str, hash: &str) -> String;
    fn user(&self) -> String;
    fn system(&self) -> String;
    fn user_auth(&self, user_id: usize) -> String;
    fn assetbundle_info(&self, asset_version: &str, platform: &Platform) -> String;
}

#[derive(Default)]
pub struct SekaiProductionUrlProvider {
    hosts: SekaiHosts,
}

impl SekaiUrlProvider for SekaiProductionUrlProvider {
    fn issue_signature(&self) -> String {
        format!("{}{}", self.hosts.issue, sekai::issue::SIGNATURE)
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

    fn assetbundle_info(&self, asset_version: &str, platform: &Platform) -> String {
        format!(
            "{}/{}/os/{}",
            sekai::assetbundle::INFO,
            asset_version,
            platform.to_string()
        )
    }
}

#[cfg(test)]
pub struct SekaiTestUrlProvider {
    url: String,
}

#[cfg(test)]
impl SekaiTestUrlProvider {
    pub fn new(url: String) -> Self {
        Self { url }
    }
}

#[cfg(test)]
impl SekaiUrlProvider for SekaiTestUrlProvider {
    fn issue_signature(&self) -> String {
        self.url.clone()
    }

    fn game_version(&self, _: &str, _: &str) -> String {
        self.url.clone()
    }

    fn user(&self) -> String {
        self.url.clone()
    }

    fn system(&self) -> String {
        self.url.clone()
    }

    fn user_auth(&self, _: usize) -> String {
        self.url.clone()
    }

    fn assetbundle_info(&self, _: &str, _: &Platform) -> String {
        self.url.clone()
    }
}

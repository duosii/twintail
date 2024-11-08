use crate::models::enums::Platform;

use super::{global_provider::GlobalUrlProvider, japan_provider::JapanUrlProvider, UrlProvider};

pub enum ServerUrlProvider {
    Japan(JapanUrlProvider),
    Global(GlobalUrlProvider),
}

impl UrlProvider for ServerUrlProvider {
    fn issue_signature(&self) -> Option<String> {
        match self {
            Self::Japan(provider) => provider.issue_signature(),
            Self::Global(provider) => provider.issue_signature(),
        }
    }

    fn game_version(&self, version: &str, hash: &str) -> String {
        match self {
            Self::Japan(provider) => provider.game_version(version, hash),
            Self::Global(provider) => provider.game_version(version, hash),
        }
    }

    fn user(&self) -> String {
        match self {
            Self::Japan(provider) => provider.user(),
            Self::Global(provider) => provider.user(),
        }
    }

    fn system(&self) -> String {
        match self {
            Self::Japan(provider) => provider.system(),
            Self::Global(provider) => provider.system(),
        }
    }

    fn user_auth(&self, user_id: usize) -> String {
        match self {
            Self::Japan(provider) => provider.user_auth(user_id),
            Self::Global(provider) => provider.user_auth(user_id),
        }
    }

    fn assetbundle_info(
        &self,
        host_hash: &str,
        asset_version: &str,
        platform: &Platform,
    ) -> String {
        match self {
            Self::Japan(provider) => provider.assetbundle_info(host_hash, asset_version, platform),
            Self::Global(provider) => provider.assetbundle_info(host_hash, asset_version, platform),
        }
    }
}

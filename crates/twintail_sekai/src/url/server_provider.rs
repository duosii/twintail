use twintail_common::models::enums::{Platform, Server};

use super::{UrlProvider, global_provider::GlobalUrlProvider, japan_provider::JapanUrlProvider};

#[derive(Clone)]
pub enum ServerUrlProvider {
    Japan(JapanUrlProvider),
    Global(GlobalUrlProvider),
}

impl Default for ServerUrlProvider {
    /// Creates a default ServerUrlProvider using the JapanUrlProvider.
    fn default() -> Self {
        Self::Japan(JapanUrlProvider::default())
    }
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
        asset_hash: &str,
        platform: &Platform,
    ) -> String {
        match self {
            Self::Japan(provider) => provider.assetbundle_info(host_hash, asset_version, asset_hash, platform),
            Self::Global(provider) => provider.assetbundle_info(host_hash, asset_version, asset_hash, platform),
        }
    }

    fn assetbundle(&self, host_hash: &str, assetbundle_path: &str) -> String {
        match self {
            Self::Japan(provider) => provider.assetbundle(host_hash, assetbundle_path),
            Self::Global(provider) => provider.assetbundle(host_hash, assetbundle_path),
        }
    }

    fn assetbundle_path(
        &self,
        asset_version: &str,
        asset_hash: &str,
        platform: &Platform,
        bundle_name: &str,
    ) -> String {
        match self {
            Self::Japan(provider) => {
                provider.assetbundle_path(asset_version, asset_hash, platform, bundle_name)
            }
            Self::Global(provider) => {
                provider.assetbundle_path(asset_version, asset_hash, platform, bundle_name)
            }
        }
    }

    fn suitemasterfile(&self, file_path: &str) -> String {
        match self {
            Self::Japan(provider) => provider.suitemasterfile(file_path),
            Self::Global(provider) => provider.suitemasterfile(file_path),
        }
    }

    fn inherit(&self, inherit_id: &str, execute: bool) -> String {
        match self {
            Self::Japan(provider) => provider.inherit(inherit_id, execute),
            Self::Global(provider) => provider.inherit(inherit_id, execute),
        }
    }

    fn user_suite(&self, user_id: usize) -> String {
        match self {
            Self::Japan(provider) => provider.user_suite(user_id),
            Self::Global(provider) => provider.user_suite(user_id),
        }
    }
}

impl From<Server> for ServerUrlProvider {
    fn from(value: Server) -> Self {
        match value {
            Server::Japan => ServerUrlProvider::Japan(JapanUrlProvider::default()),
            Server::Global => ServerUrlProvider::Global(GlobalUrlProvider::default()),
        }
    }
}

use super::UrlProvider;
use crate::url::urls;
use twintail_common::models::enums::Platform;

#[derive(Clone)]
pub struct TestUrlProvider {
    host: String,
}

impl TestUrlProvider {
    pub fn new(url: String) -> Self {
        Self { host: url }
    }
}

impl UrlProvider for TestUrlProvider {
    fn issue_signature(&self) -> Option<String> {
        Some(format!("{}{}", self.host, urls::issue::SIGNATURE))
    }

    fn game_version(&self, version: &str, hash: &str) -> String {
        format!("{}/{}/{}", self.host, version, hash)
    }

    fn user(&self) -> String {
        format!("{}{}", self.host, urls::game::USER)
    }

    fn system(&self) -> String {
        format!("{}{}", self.host, urls::game::SYSTEM)
    }

    fn user_auth(&self, user_id: usize) -> String {
        format!(
            "{}{}/{}/auth?refreshUpdatedResources=False",
            self.host,
            urls::game::USER_AUTH,
            user_id
        )
    }

    fn assetbundle_info(
        &self,
        host_hash: &str,
        asset_version: &str,
        asset_hash: &str,
        platform: &Platform,
    ) -> String {
        format!(
            "{}-{}{}/{}/{}/os/{}",
            self.host,
            host_hash,
            urls::assetbundle::INFO,
            asset_version,
            asset_hash,
            platform.to_string()
        )
    }

    fn assetbundle(&self, host_hash: &str, assetbundle_path: &str) -> String {
        format!("{}-{}/{}", self.host, host_hash, assetbundle_path)
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
        format!("{}{}/{}", self.host, urls::game::API, file_path)
    }

    fn inherit(&self, inherit_id: &str, execute: bool) -> String {
        format!(
            "{}{}/{}?isExecuteInherit={}",
            self.host,
            urls::game::INHERIT,
            inherit_id,
            if execute { "True" } else { "False" }
        )
    }

    fn user_suite(&self, user_id: usize) -> String {
        format!("{}{}/{}", self.host, urls::game::USER_SUITE, user_id)
    }
}

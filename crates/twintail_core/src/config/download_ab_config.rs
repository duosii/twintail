use twintail_common::models::OptionalBuilder;
use twintail_sekai::models::AssetbundleInfo;

// constants
const DEFAULT_UPDATE: bool = false;

/// Configuration for encryption and decryption.
pub struct DownloadAbConfig {
    pub asset_version: Option<String>,
    pub host_hash: Option<String>,
    pub info: Option<AssetbundleInfo>,
    pub update: bool,
    pub filter: Option<String>,
}

impl Default for DownloadAbConfig {
    fn default() -> Self {
        Self {
            asset_version: None,
            host_hash: None,
            info: None,
            update: DEFAULT_UPDATE,
            filter: None,
        }
    }
}

impl DownloadAbConfig {
    /// Create a default builder for the CryptConfig struct.
    pub fn builder() -> DownloadAbConfigBuilder {
        DownloadAbConfigBuilder::default()
    }
}

/// Builder for CryptConfig
#[derive(Default)]
pub struct DownloadAbConfigBuilder {
    config: DownloadAbConfig,
}

impl OptionalBuilder for DownloadAbConfigBuilder {}

impl DownloadAbConfigBuilder {
    /// The version of the assets to get. Uses the most recent if not provided
    pub fn asset_version(mut self, asset_version: String) -> Self {
        self.config.asset_version = Some(asset_version);
        self
    }

    /// Part of the URL used to download the assetbundles from. Uses the most recent if not provided
    pub fn host_hash(mut self, host_hash: String) -> Self {
        self.config.host_hash = Some(host_hash);
        self
    }

    /// Path to an assetbundle info file. If not provided, the latest one will be fetched
    pub fn info(mut self, info: AssetbundleInfo) -> Self {
        //self.config.host_hash = info.host_hash.clone();
        //self.config.asset_version = Some(info.version.clone());
        self.config.info = Some(info);
        self
    }

    /// If true, the assetbundle info file value will not be updated to the most recent asset version
    pub fn update(mut self, update: bool) -> Self {
        self.config.update = update;
        self
    }

    /// Only assetbundles that match this regular expression will be downloaded
    pub fn filter(mut self, filter: String) -> Self {
        self.config.filter = Some(filter);
        self
    }

    /// Returns the CryptConfig that was constructed.
    pub fn build(self) -> DownloadAbConfig {
        self.config
    }
}

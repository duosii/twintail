use crate::{
    api::url::{japan_provider::JapanUrlProvider, server_provider::ServerUrlProvider, UrlProvider},
    models::enums::{Platform, Server},
    utils::available_parallelism,
};

use super::{AesConfig, OptionalBuilder};

// constants
const DEFAULT_SERVER: Server = Server::Japan;
const DEFAULT_RECURSIVE: bool = false;
const DEFAULT_QUIET: bool = false;
const DEFAULT_PLATFORM: Platform = Platform::Android;
const DEFAULT_RETRY: usize = 3;
const DEFAULT_DECRYPT: bool = true;

/// Configuration for encryption and decryption.
pub struct FetchConfig<P: UrlProvider> {
    pub aes_config: AesConfig,
    pub concurrency: usize,
    pub recursive: bool,
    pub quiet: bool,
    pub version: String,
    pub hash: String,
    pub platform: Platform,
    pub retry: usize,
    pub decrypt: bool,
    pub url_provider: P,
    pub pretty_json: bool,
}

impl FetchConfig<ServerUrlProvider> {
    /// Create a new FetchConfig with the provided version and hash.
    ///
    /// Uses a default Japan url provider.
    pub fn new(version: String, hash: String) -> Self {
        Self::new_with_provider(
            version,
            hash,
            ServerUrlProvider::Japan(JapanUrlProvider::default()),
        )
    }

    /// Create a default builder for the CryptConfig struct.
    pub fn builder(version: String, hash: String) -> FetchConfigBuilder<ServerUrlProvider> {
        FetchConfigBuilder::new(version, hash)
    }
}

impl<P: UrlProvider> FetchConfig<P> {
    /// Create a new FetchConfig with the provided version, hash, and url_provider using default values.
    pub fn new_with_provider(version: String, hash: String, url_provider: P) -> Self {
        Self {
            aes_config: DEFAULT_SERVER.get_aes_config(),
            url_provider,
            concurrency: available_parallelism(),
            recursive: DEFAULT_RECURSIVE,
            quiet: DEFAULT_QUIET,
            version,
            hash,
            platform: DEFAULT_PLATFORM,
            retry: DEFAULT_RETRY,
            decrypt: DEFAULT_DECRYPT,
            pretty_json: false
        }
    }
}

/// Builder for CryptConfig
pub struct FetchConfigBuilder<P: UrlProvider> {
    config: FetchConfig<P>,
}

impl<P: UrlProvider> OptionalBuilder for FetchConfigBuilder<P> {}

impl FetchConfigBuilder<ServerUrlProvider> {
    /// Creates a new FetchConfigBuilder with the provided version and hash
    pub fn new(version: String, hash: String) -> Self {
        Self {
            config: FetchConfig::new(version, hash),
        }
    }
}

impl<P: UrlProvider> FetchConfigBuilder<P> {
    /// Sets the aes configuration.
    ///
    /// By default, this will use the AesConfig for the Japan server.
    pub fn aes(mut self, aes_config: AesConfig) -> Self {
        self.config.aes_config = aes_config;
        self
    }

    /// Sets the FetchConfig to use the configurations required by the provided server.
    ///
    /// By default this will be the Japan server.
    pub fn server(self, server: Server) -> Self {
        self.aes(server.get_aes_config())
    }

    /// Sets the maximum number of tokio threads that can be used.
    ///
    /// By default, this is the result of [`crate::utils::available_parallelism`],
    /// the machine's available parallelism.
    pub fn concurrency(mut self, concurrency: usize) -> Self {
        self.config.concurrency = concurrency;
        self
    }

    /// When performing operations on paths, whether to recursively operate
    /// on that path.
    ///
    /// By default, this is false.
    pub fn recursive(mut self, recursive: bool) -> Self {
        self.config.recursive = recursive;
        self
    }

    /// When performing operations, whether to print information
    /// regarding the progress of the operation.
    ///
    /// By default, this is false.
    pub fn quiet(mut self, quiet: bool) -> Self {
        self.config.quiet = quiet;
        self
    }

    /// Sets the game version that this config will use
    ///
    /// This field is required and has no default value.
    pub fn version(mut self, version: String) -> Self {
        self.config.version = version;
        self
    }

    /// Sets the app hash that this config will use
    ///
    /// This field is required and has no default value.
    pub fn hash(mut self, hash: String) -> Self {
        self.config.hash = hash;
        self
    }

    /// Sets the platform (operating system) that this config will use.
    ///
    /// By default, this is ``Platform::Android``
    pub fn platform(mut self, platform: Platform) -> Self {
        self.config.platform = platform;
        self
    }

    /// Sets the amount of time to retry failed operations.
    ///
    /// By default, this is 3 times.
    pub fn retry(mut self, retries: usize) -> Self {
        self.config.retry = retries;
        self
    }

    /// Sets whether to automatically decrypt encrypted assets where applicable.
    ///
    /// By default, this is true.
    pub fn decrypt(mut self, decrypt: bool) -> Self {
        self.config.decrypt = decrypt;
        self
    }

    /// Sets what URLs to access when performing operations.
    ///
    /// By default, this is the URLs for the Japan server.
    pub fn url_provider(mut self, provider: P) -> Self {
        self.config.url_provider = provider;
        self
    }

    /// When performing operations with JSON files, whether to
    /// format those files in a more readable format.
    /// 
    /// This will slightly increase the size of any output .json files
    /// due to extra spaces and newlines.
    pub fn pretty_json(mut self, pretty: bool) -> Self {
        self.config.pretty_json = pretty;
        self
    }

    /// Returns the FetchConfig that was constructed.
    pub fn build(self) -> FetchConfig<P> {
        self.config
    }
}

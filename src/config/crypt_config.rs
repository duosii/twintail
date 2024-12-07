use crate::{models::enums::Server, utils::available_parallelism};

use super::{AesConfig, OptionalBuilder};

// constants
const DEFAULT_SERVER: Server = Server::Japan;
const DEFAULT_RECURSIVE: bool = false;
const DEFAULT_QUIET: bool = false;

/// Configuration for encryption and decryption.
pub struct CryptConfig {
    pub aes_config: AesConfig,
    pub concurrency: usize,
    pub recursive: bool,
    pub quiet: bool,
    pub pretty_json: bool,
}

impl Default for CryptConfig {
    fn default() -> Self {
        Self {
            aes_config: DEFAULT_SERVER.get_aes_config(),
            concurrency: available_parallelism(),
            recursive: DEFAULT_RECURSIVE,
            quiet: DEFAULT_QUIET,
            pretty_json: false
        }
    }
}

impl CryptConfig {
    /// Create a default builder for the CryptConfig struct.
    pub fn builder() -> CryptConfigBuilder {
        CryptConfigBuilder::default()
    }
}

/// Builder for CryptConfig
#[derive(Default)]
pub struct CryptConfigBuilder {
    config: CryptConfig,
}

impl OptionalBuilder for CryptConfigBuilder {}

impl CryptConfigBuilder {
    /// Sets the aes configuration.
    ///
    /// By default, this will use the AesConfig for the Japan server.
    pub fn aes(mut self, aes_config: AesConfig) -> Self {
        self.config.aes_config = aes_config;
        self
    }

    /// Sets the CryptConfig to use the configurations required by the provided server.
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
    
    /// When performing operations with JSON files, whether to
    /// format those files in a more readable format.
    /// 
    /// This will slightly increase the size of any output .json files
    /// due to extra spaces and newlines.
    pub fn pretty_json(mut self, pretty: bool) -> Self {
        self.config.pretty_json = pretty;
        self
    }

    /// Returns the CryptConfig that was constructed.
    pub fn build(self) -> CryptConfig {
        self.config
    }
}

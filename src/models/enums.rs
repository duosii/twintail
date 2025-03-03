use clap::ValueEnum;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

use crate::{
    api::url::{
        global_provider::GlobalUrlProvider, japan_provider::JapanUrlProvider,
        server_provider::ServerUrlProvider,
    },
    config::AesConfig,
    constants::crypto,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Serialize, Deserialize)]
pub enum Platform {
    Android,
    Ios,
}

impl ToString for Platform {
    fn to_string(&self) -> String {
        match self {
            Platform::Android => String::from("android"),
            Platform::Ios => String::from("ios"),
        }
    }
}

#[derive(PartialEq)]
pub enum CryptOperation {
    Encrypt,
    Decrypt,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
pub enum AssetbundleCategory {
    OnDemand,
    StartApp,
    AdditionalVoice,
    Tutorial,
    MysekaiVoice,
}

/// Represents a server for the game in a specific region
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Serialize, Deserialize)]
pub enum Server {
    Japan,
    Global,
}

impl Server {
    /// Returns the AesConfig for a server.
    pub fn get_aes_config(&self) -> AesConfig {
        match self {
            Self::Japan => AesConfig {
                key: *crypto::JAPAN_KEY,
                iv: *crypto::JAPAN_IV,
            },
            Self::Global => AesConfig {
                key: *crypto::GLOBAL_KEY,
                iv: *crypto::GLOBAL_IV,
            },
        }
    }

    /// Returns the JSON Web Token HMAC SHA-256 key for a server.
    pub fn get_jwt_key(&self) -> Hmac<Sha256> {
        match self {
            Self::Japan => Hmac::new_from_slice(
                b"dRmS5U3jP9XJDFzoI7eeXhzT826v2qJRO9n14h9JR1phTL6so3v7YBiODRdrrfMOl3Y8FOI3pS5UTYC5",
            )
            .unwrap(),
            Self::Global => Hmac::new_from_slice(
                b"uYf0cGqbgapejhc8bhba6G1cf5BBznOZeDz9NyFWZOgiiYsfUVNLT3wRUpCH6iDe1umsreAYuo35s8TP",
            )
            .unwrap(),
        }
    }

    pub fn get_url_provider(&self) -> ServerUrlProvider {
        match self {
            Self::Japan => ServerUrlProvider::Japan(JapanUrlProvider::default()),
            Self::Global => ServerUrlProvider::Global(GlobalUrlProvider::default()),
        }
    }
}

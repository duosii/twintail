use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use crate::constants::crypto;

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

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
pub enum AssetbundleCategory {
    OnDemand,
    StartApp,
    AdditionalVoice,
    Tutorial,
}

/// Represents a server for the game in a specific region
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Serialize, Deserialize)]
pub enum Server {
    Japan,
    Global,
}

pub struct AesConfig {
    pub key: &'static [u8],
    pub iv: &'static [u8],
}
impl Server {
    pub fn get_aes_config(&self) -> AesConfig {
        match self {
            Self::Japan => AesConfig {
                key: crypto::JAPAN_KEY,
                iv: crypto::JAPAN_IV,
            },
            Self::Global => AesConfig {
                key: crypto::GLOBAL_KEY,
                iv: crypto::GLOBAL_IV,
            },
        }
    }
}

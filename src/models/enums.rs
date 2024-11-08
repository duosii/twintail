use clap::ValueEnum;
use serde::{Deserialize, Serialize};

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

/// Represents a server for the game in a specific region.\
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Serialize, Deserialize)]
pub enum Server {
    Japan,
    Global,
}

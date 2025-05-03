use thiserror::Error;
use twintail_common::multi_error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("not an encrypted assetbundle")]
    NotEncrypted,

    #[error("not an assetbundle")]
    NotAssetbundle,

    #[error("zip archive error: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("regex error: {0}")]
    Regex(#[from] regex::Error),

    #[error("join error: {0}")]
    Join(#[from] tokio::task::JoinError),

    #[error("JSON de/serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("common error: {0}")]
    Crypto(#[from] twintail_common::error::CryptoError),

    #[error("sekai error: {0}")]
    Sekai(#[from] twintail_sekai::Error),

    #[error("rmp_serde decode error: {0}")]
    RmpSerdeDecode(#[from] rmp_serde::decode::Error),

    #[error("rmp_serde encode error: {0}")]
    RmpSerdeEncode(#[from] rmp_serde::encode::Error),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("not enough space: {0}")]
    NotEnoughSpace(String),

    #[error("multiple errors: {0}")]
    Multi(String),
}
multi_error!(Error);

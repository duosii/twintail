use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("invalid header value: {0}")]
    InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),

    #[error("serde_plain error: {0}")]
    SerdePlain(#[from] serde_plain::Error),

    #[error("rmp_serde decode error: {0}")]
    RmpSerdeDecode(#[from] rmp_serde::decode::Error),

    #[error("rmp_serde encode error: {0}")]
    RmpSerdeEncode(#[from] rmp_serde::encode::Error),

    #[error("multiple errors: {0}")]
    Multi(String),

    #[error("invalid request: {0}")]
    InvalidRequest(String),

    #[error("missing url: {0}")]
    MissingUrl(String),
}

impl From<Vec<ApiError>> for ApiError {
    fn from(value: Vec<ApiError>) -> Self {
        let errs_joined: String = value
            .into_iter()
            .map(|e| e.to_string())
            .collect::<Vec<String>>()
            .join(", ");

        Self::Multi(errs_joined)
    }
}

#[derive(Error, Debug)]
pub enum CommandError {
    #[error("api error: {0}")]
    Api(#[from] ApiError),

    #[error("io error: {0}")]
    Io(#[from] tokio::io::Error),

    #[error("assetbundle error: {0}")]
    Assetbundle(#[from] AssetbundleError),

    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("rmp_serde encode error: {0}")]
    RmpSerdeEncode(#[from] rmp_serde::encode::Error),

    #[error("not enough space: {0}")]
    NotEnoughSpace(String),

    #[error("not found: {0}")]
    NotFound(String),
}

#[derive(Error, Debug)]
pub enum AssetbundleError {
    #[error("io error: {0}")]
    Io(#[from] tokio::io::Error),

    #[error("not an encrypted assetbundle")]
    NotEncrypted(),

    #[error("not an assetbundle")]
    NotAssetbundle(),
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("command error: {0}")]
    Command(#[from] CommandError),
}

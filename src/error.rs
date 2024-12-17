use thiserror::Error;

macro_rules! multi_error {
    ($error_enum:ident) => {
        impl From<Vec<$error_enum>> for $error_enum {
            fn from(value: Vec<$error_enum>) -> Self {
                let errs_joined: String = value
                    .into_iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<String>>()
                    .join(", ");

                Self::Multi(errs_joined)
            }
        }
    };
}

#[derive(Error, Debug)]
pub enum CommonError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serde_plain error: {0}")]
    SerdePlain(#[from] serde_plain::Error),

    #[error("rmp_serde decode error: {0}")]
    RmpSerdeDecode(#[from] rmp_serde::decode::Error),

    #[error("rmp_serde encode error: {0}")]
    RmpSerdeEncode(#[from] rmp_serde::encode::Error),

    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("not found: {0}")]
    NotFound(String),
}

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("serde_plain error: {0}")]
    SerdePlain(#[from] serde_plain::Error),

    #[error("rmp_serde decode error: {0}")]
    RmpSerdeDecode(#[from] rmp_serde::decode::Error),

    #[error("rmp_serde encode error: {0}")]
    RmpSerdeEncode(#[from] rmp_serde::encode::Error),

    #[error("invalid header value: {0}")]
    InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),

    #[error("jwt error: {0}")]
    Jwt(#[from] jwt::Error),

    #[error("invalid request: {0}")]
    InvalidRequest(String),

    #[error("missing url: {0}")]
    MissingUrl(String),

    #[error("multiple errors: {0}")]
    Multi(String),
}
multi_error!(ApiError);

#[derive(Error, Debug)]
pub enum ApkExtractError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("zip archive error: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("regex error: {0}")]
    Regex(#[from] regex::Error),

    #[error("join error: {0}")]
    Join(#[from] tokio::task::JoinError),
}

#[derive(Error, Debug)]
pub enum AssetbundleError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("not an encrypted assetbundle")]
    NotEncrypted,

    #[error("not an assetbundle")]
    NotAssetbundle,
}

#[derive(Error, Debug)]
pub enum CommandError {
    #[error("common error: {0}")]
    Common(#[from] CommonError),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("api error: {0}")]
    Api(#[from] ApiError),

    #[error("assetbundle error: {0}")]
    Assetbundle(#[from] AssetbundleError),

    #[error("apk extract error: {0}")]
    ApkExtract(#[from] ApkExtractError),

    #[error("not enough space: {0}")]
    NotEnoughSpace(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("serde_plain error: {0}")]
    SerdePlain(#[from] serde_plain::Error),

    #[error("rmp_serde decode error: {0}")]
    RmpSerdeDecode(#[from] rmp_serde::decode::Error),

    #[error("rmp_serde encode error: {0}")]
    RmpSerdeEncode(#[from] rmp_serde::encode::Error),

    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("file stem missing for file at: {0}")]
    FileStem(String),

    #[error("multiple errors: {0}")]
    Multi(String),
}
multi_error!(CommandError);

#[derive(Error, Debug)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("rmp_serde decode error: {0}")]
    RmpSerdeDecode(#[from] rmp_serde::decode::Error),

    #[error("rmp_serde encode error: {0}")]
    RmpSerdeEncode(#[from] rmp_serde::encode::Error),

    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("command error: {0}")]
    Command(#[from] CommandError),

    #[error("assetbundle error: {0}")]
    Assetbundle(#[from] AssetbundleError),

    #[error("apk extract error: {0}")]
    ApkExtract(#[from] ApkExtractError),

    #[error("api error: {0}")]
    Api(#[from] ApiError),

    #[error("common error: {0}")]
    Common(#[from] CommonError),

    #[error("zip archive error: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("regex error: {0}")]
    Regex(#[from] regex::Error),

    #[error("join error: {0}")]
    Join(#[from] tokio::task::JoinError),
}

// Allow multiple errors to be joined together for CommonError

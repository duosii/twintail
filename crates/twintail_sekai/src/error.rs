use thiserror::Error;
use twintail_common::multi_error;

#[derive(Error, Debug)]
pub enum Error {
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
multi_error!(Error);

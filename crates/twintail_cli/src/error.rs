use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("core error: {0}")]
    TwintailCore(#[from] twintail_core::Error),

    #[error("JSON de/serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
}

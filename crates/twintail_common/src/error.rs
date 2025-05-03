use thiserror::Error;

#[macro_export]
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
pub enum CryptoError {
    #[error("error when parsing int: {0}")]
    ParseInt(#[from] std::num::ParseIntError),

    #[error("invalid key length: must be 16 bytes long")]
    InvalidKeyLength(),
}

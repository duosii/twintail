use crate::{error::CommonError, utils::decode_hex, Error};

pub mod crypt_config;
pub mod download_ab_config;
pub mod fetch_config;

#[derive(Clone)]
pub struct AesConfig {
    pub key: [u8; 16],
    pub iv: [u8; 16],
}

impl AesConfig {
    /// Generates an AesConfig using hexadecimal key & IV values.
    /// 
    /// The hexadecimal values should be strings.
    /// 
    /// This function may error if parsing the hexadecimal strings fails.
    pub fn from_hex(hex_key: &str, hex_iv: &str) -> Result<Self, Error> {
        Ok(Self {
            key: decode_hex(hex_key).map_err(CommonError::from)?.try_into().map_err(|_| CommonError::InvalidKeyLength())?,
            iv: decode_hex(hex_iv).map_err(CommonError::from)?.try_into().map_err(|_| CommonError::InvalidKeyLength())?
        })
    }
}

pub trait OptionalBuilder: Sized {
    fn map<T>(self, value: Option<T>, f: impl FnOnce(Self, T) -> Self) -> Self {
        match value {
            Some(v) => f(self, v),
            None => self,
        }
    }
}

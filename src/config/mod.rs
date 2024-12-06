use crate::utils::decode_hex;

pub mod crypt_config;
pub mod download_ab_config;
pub mod fetch_config;

#[derive(Clone)]
pub struct AesConfig {
    pub key: Vec<u8>,
    pub iv: Vec<u8>,
}

impl AesConfig {
    /// Generates an AesConfig using hexadecimal key & IV values.
    /// 
    /// The hexadecimal values should be strings.
    /// 
    /// This function may error if parsing the hexadecimal strings fails.
    pub fn from_hex(hex_key: &str, hex_iv: &str) -> Result<Self, std::num::ParseIntError> {
        Ok(Self {
            key: decode_hex(hex_key)?,
            iv: decode_hex(hex_iv)?
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

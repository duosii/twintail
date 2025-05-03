use reqwest::header::{HeaderMap, HeaderValue, InvalidHeaderValue};
use twintail_common::models::enums::Platform;

use crate::Error;

// header names
pub mod header_name {
    pub const CONTENT_TYPE: &str = "content-type";
    pub const ACCEPT: &str = "accept";
    pub const USER_AGENT: &str = "user-agent";
    pub const INSTALL_ID: &str = "x-install-id";
    pub const APP_VERSION: &str = "x-app-version";
    pub const APP_HASH: &str = "x-app-hash";
    pub const PLATFORM: &str = "x-platform";
    pub const DEVICE_MODEL: &str = "x-devicemodel";
    pub const OPERATING_SYSTEM: &str = "x-operatingsystem";
    pub const UNITY_VERSION: &str = "x-unity-version";
    pub const SET_COOKIE: &str = "set-cookie";
    pub const COOKIE: &str = "cookie";
    pub const SESSION_TOKEN: &str = "x-session-token";
    pub const INHERIT_TOKEN: &str = "x-inherit-id-verify-token";
    pub const DATA_VERSION: &str = "x-data-version";
    pub const ASSET_VERSION: &str = "x-asset-version";
}

// header values
pub mod header_value {
    pub const CONTENT_TYPE: &str = "application/octet-stream";
    pub const ACCEPT: &str = "application/octet-stream";
    pub const USER_AGENT: &str = "UnityPlayer/2022.3.21f1 (UnityWebRequest/1.0, libcurl/8.5.0-DEV)";
    pub const INSTALL_ID: &str = "3efd7166-11b2-4b3e-2f08-94b0e16f76e8";
    pub const DEVICE_MODEL: &str = "39phone";
    pub const OPERATING_SYSTEM: &str = "39os";
    pub const UNITY_VERSION: &str = "2022.3.21f1";
}

pub struct Headers(pub HeaderMap<HeaderValue>);

impl Headers {
    /// Get a new SekaiHeaders object with some default headers.
    fn new() -> Result<Self, InvalidHeaderValue> {
        let mut headers = Self::default();

        headers.insert_str(header_name::CONTENT_TYPE, header_value::CONTENT_TYPE)?;
        headers.insert_str(header_name::ACCEPT, header_value::ACCEPT)?;
        headers.insert_str(header_name::USER_AGENT, header_value::USER_AGENT)?;
        headers.insert_str(header_name::INSTALL_ID, header_value::INSTALL_ID)?;
        headers.insert_str(header_name::DEVICE_MODEL, header_value::DEVICE_MODEL)?;
        headers.insert_str(
            header_name::OPERATING_SYSTEM,
            header_value::OPERATING_SYSTEM,
        )?;
        headers.insert_str(header_name::UNITY_VERSION, header_value::UNITY_VERSION)?;

        Ok(headers)
    }

    /// Get a builder for SekaiHeaders
    pub fn builder() -> Result<HeadersBuilder, InvalidHeaderValue> {
        Ok(HeadersBuilder::new(Self::new()?))
    }

    pub fn insert(&mut self, name: &'static str, value: HeaderValue) {
        self.0.insert(name, value);
    }

    /// Insert a str value as a header.
    pub fn insert_str(
        &mut self,
        name: &'static str,
        value: &str,
    ) -> Result<(), InvalidHeaderValue> {
        self.insert(name, HeaderValue::from_str(value)?);
        Ok(())
    }

    /// Clones the inner HeaderMap and returns it.
    pub fn get_map(&self) -> HeaderMap {
        self.0.clone()
    }
}

impl Default for Headers {
    fn default() -> Self {
        Self(HeaderMap::new())
    }
}

#[derive(Default)]
pub struct HeadersBuilder {
    headers: Headers,
    errors: Vec<Error>,
}

impl HeadersBuilder {
    /// Create a new header.
    pub fn new(headers: Headers) -> Self {
        Self {
            headers,
            errors: Vec::new(),
        }
    }

    /// Add a version header
    pub fn version(mut self, version: &str) -> Self {
        match HeaderValue::from_str(version) {
            Ok(header_value) => self
                .headers
                .0
                .insert(header_name::APP_VERSION, header_value),
            Err(e) => {
                self.errors.push(Error::InvalidHeaderValue(e));
                None
            }
        };
        self
    }

    /// Add a hash header.
    pub fn hash(mut self, hash: &str) -> Self {
        match HeaderValue::from_str(hash) {
            Ok(header_value) => self.headers.0.insert(header_name::APP_HASH, header_value),
            Err(e) => {
                self.errors.push(Error::InvalidHeaderValue(e));
                None
            }
        };
        self
    }

    /// Add a platform header.
    pub fn platform(mut self, platform: &Platform) -> Self {
        match serde_plain::to_string(platform) {
            Ok(platform_string) => match HeaderValue::from_str(&platform_string) {
                Ok(header_value) => self.headers.0.insert(header_name::PLATFORM, header_value),
                Err(e) => {
                    self.errors.push(Error::InvalidHeaderValue(e));
                    None
                }
            },
            Err(e) => {
                self.errors.push(Error::SerdePlain(e));
                None
            }
        };
        self
    }

    /// Build this builder, returning a result with any errors that occured.
    pub fn build(self) -> Result<Headers, Vec<Error>> {
        if self.errors.is_empty() {
            Ok(self.headers)
        } else {
            Err(self.errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Headers;
    use super::*;

    #[test]
    fn test_new_sekai_headers() {
        assert!(Headers::new().is_ok())
    }

    #[test]
    fn test_new_sekai_headers_builder() {
        assert!(Headers::builder().is_ok())
    }

    #[test]
    fn test_sekai_headers_builder_version() {
        let builder = Headers::builder().unwrap();

        let version = "4.0.5";
        let headers = builder.version(version).build().unwrap();

        assert_eq!(headers.0.get(header_name::APP_VERSION).unwrap(), version);
    }

    #[test]
    fn test_sekai_headers_builder_hash() {
        let builder = Headers::builder().unwrap();

        let hash = "2179da72-9de5-23a6-f388-9e5835098ce1";
        let headers = builder.hash(hash).build().unwrap();

        assert_eq!(headers.0.get(header_name::APP_HASH).unwrap(), hash);
    }

    #[test]
    fn test_sekai_headers_builder_platform() {
        let builder = Headers::builder().unwrap();

        let platform = Platform::Ios;
        let headers = builder.platform(&platform).build().unwrap();

        let serialized = serde_plain::to_string(&platform).unwrap();
        assert_eq!(headers.0.get(header_name::PLATFORM).unwrap(), &serialized);
    }
}

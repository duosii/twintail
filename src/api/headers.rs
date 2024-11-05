use crate::{constants::header, error::ApiError, models::enums::Platform};
use reqwest::header::{HeaderMap, HeaderValue, InvalidHeaderValue};

pub struct Headers(pub HeaderMap<HeaderValue>);

impl Headers {
    /// Get a new SekaiHeaders obeject with some default headers.
    fn new() -> Result<Self, InvalidHeaderValue> {
        let mut headers = Self::default();

        headers.insert_str(header::name::CONTENT_TYPE, header::value::CONTENT_TYPE)?;
        headers.insert_str(header::name::ACCEPT, header::value::ACCEPT)?;
        headers.insert_str(header::name::USER_AGENT, header::value::USER_AGENT)?;
        headers.insert_str(header::name::INSTALL_ID, header::value::INSTALL_ID)?;
        headers.insert_str(header::name::DEVICE_MODEL, header::value::DEVICE_MODEL)?;
        headers.insert_str(
            header::name::OPERATING_SYSTEM,
            header::value::OPERATING_SYSTEM,
        )?;
        headers.insert_str(header::name::UNITY_VERSION, header::value::UNITY_VERSION)?;

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
    errors: Vec<ApiError>,
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
                .insert(header::name::APP_VERSION, header_value),
            Err(e) => {
                self.errors.push(ApiError::InvalidHeaderValue(e));
                None
            }
        };
        self
    }

    /// Add a hash header.
    pub fn hash(mut self, hash: &str) -> Self {
        match HeaderValue::from_str(hash) {
            Ok(header_value) => self.headers.0.insert(header::name::APP_HASH, header_value),
            Err(e) => {
                self.errors.push(ApiError::InvalidHeaderValue(e));
                None
            }
        };
        self
    }

    /// Add a platform header.
    pub fn platform(mut self, platform: &Platform) -> Self {
        match serde_plain::to_string(platform) {
            Ok(platform_string) => match HeaderValue::from_str(&platform_string) {
                Ok(header_value) => self.headers.0.insert(header::name::PLATFORM, header_value),
                Err(e) => {
                    self.errors.push(ApiError::InvalidHeaderValue(e));
                    None
                }
            },
            Err(e) => {
                self.errors.push(ApiError::SerdePlain(e));
                None
            }
        };
        self
    }

    /// Build this builder, returning a result with any errors that occured.
    pub fn build(self) -> Result<Headers, Vec<ApiError>> {
        if self.errors.is_empty() {
            Ok(self.headers)
        } else {
            Err(self.errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{constants::header, models::enums::Platform};

    use super::Headers;

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

        assert_eq!(headers.0.get(header::name::APP_VERSION).unwrap(), version);
    }

    #[test]
    fn test_sekai_headers_builder_hash() {
        let builder = Headers::builder().unwrap();

        let hash = "2179da72-9de5-23a6-f388-9e5835098ce1";
        let headers = builder.hash(hash).build().unwrap();

        assert_eq!(headers.0.get(header::name::APP_HASH).unwrap(), hash);
    }

    #[test]
    fn test_sekai_headers_builder_platform() {
        let builder = Headers::builder().unwrap();

        let platform = Platform::Ios;
        let headers = builder.platform(&platform).build().unwrap();

        let serialized = serde_plain::to_string(&platform).unwrap();
        assert_eq!(headers.0.get(header::name::PLATFORM).unwrap(), &serialized);
    }
}

use super::{headers::Headers, url::UrlProvider};
use crate::{
    config::AesConfig,
    constants::{
        header::{self, name::INHERIT_TOKEN},
        strings,
    },
    crypto::aes_msgpack,
    error::ApiError,
    models::{
        api::{
            AssetbundleInfo, GameVersion, SystemInfo, UserAuthRequest, UserAuthResponse,
            UserInherit, UserInheritJWT, UserRequest, UserSignup,
        },
        enums::Platform,
    },
};
use hmac::Hmac;
use jwt::SignWithKey;
use reqwest::{header::HeaderValue, Client, StatusCode};
use serde_json::Value;
use sha2::Sha256;

/// A simple struct that stores information about the game's app.
pub struct SekaiApp {
    pub version: String,
    pub hash: String,
    pub platform: Platform,
}

impl SekaiApp {
    pub fn new(version: String, hash: String, platform: Platform) -> Self {
        Self {
            version,
            hash,
            platform,
        }
    }
}

/// An API client that interfaces with the game's servers, providing various functions to query endpoints.
pub struct SekaiClient<T: UrlProvider> {
    headers: Headers,
    client: Client,
    aes_config: AesConfig,
    jwt_key: Hmac<Sha256>,
    pub url_provider: T,
    pub app: SekaiApp,
}

impl<T: UrlProvider> SekaiClient<T> {
    /// Creates a new SekaiClient that uses a specific url provider.
    pub async fn new_with_url_provider(
        version: String,
        hash: String,
        platform: Platform,
        aes_config: AesConfig,
        jwt_key: Hmac<Sha256>,
        url_provider: T,
    ) -> Result<Self, ApiError> {
        let headers = Headers::builder()?
            .version(&version)
            .hash(&hash)
            .platform(&platform)
            .build()?;

        let mut client = Self {
            headers,
            client: Client::new(),
            aes_config,
            jwt_key,
            url_provider,
            app: SekaiApp::new(version, hash, platform),
        };

        // save the cloudfront signature only if required
        if client.url_provider.issue_signature().is_some() {
            client.issue_signature().await?;
        }

        Ok(client)
    }

    /// Performs a request to [`constants::url::sekai::ISSUE_SIGNATURE`].
    ///
    /// This endpoint responds with a CloudFront cookie value,
    /// which we need in order to communicate with the CDN.
    ///
    /// The function will automatically assign this cookie value to its Headers.
    async fn issue_signature(&mut self) -> Result<(), ApiError> {
        let url = self
            .url_provider
            .issue_signature()
            .ok_or(ApiError::MissingUrl("issue_signature".to_string()))?;

        let request = self
            .client
            .post(url)
            .body(b"ffa3bd6214f33fe73cb72fee2262bedb".to_vec())
            .headers(self.headers.get_map());

        match request.send().await?.error_for_status() {
            Ok(mut response) => {
                // set the cookie that is inside of issue_signature_response
                let set_cookie_header = response
                    .headers_mut()
                    .remove(header::name::SET_COOKIE)
                    .ok_or(ApiError::InvalidRequest(
                        strings::api::error::SET_COOKIE_NOT_FOUND.into(),
                    ))?;
                self.headers.insert(header::name::COOKIE, set_cookie_header);
                Ok(())
            }
            Err(err) => Err(ApiError::InvalidRequest(err.to_string())),
        }
    }

    /// Performs a request to [`constants::url::sekai::GAME_VERSION`].
    ///
    /// This endpoint will respond with info about the game version that the URL corresponds to.
    /// The version, hash, and platform values that this [`SekaiClient`] was created with determine this.
    ///
    /// Returns the parsed GameVersion data if it was found.
    pub async fn get_game_version(&self) -> Result<GameVersion, ApiError> {
        let request = self
            .client
            .get(
                self.url_provider
                    .game_version(&self.app.version, &self.app.hash),
            )
            .headers(self.headers.get_map());

        match request.send().await?.error_for_status() {
            Ok(response) => {
                let bytes = response.bytes().await?;
                Ok(aes_msgpack::from_slice(&bytes, &self.aes_config)?)
            }
            Err(err) => match err.status() {
                Some(StatusCode::FORBIDDEN) => Err(ApiError::InvalidRequest(
                    strings::api::error::INVALID_HASH_VERSION.into(),
                )),
                _ => Err(ApiError::InvalidRequest(err.to_string())),
            },
        }
    }

    /// Performs a request to [`constants::url::sekai::USER`].
    ///
    /// This endpoint will sign up for a new account, returning the account's default data
    /// and a credential to login later.
    ///
    /// This function will return a portion of this response; the user_registration info
    /// and the credential.
    pub async fn user_signup(&self) -> Result<UserSignup, ApiError> {
        let request_body = aes_msgpack::into_vec(
            &UserRequest {
                platform: self.app.platform,
                device_model: header::value::DEVICE_MODEL.into(),
                operating_system: header::value::OPERATING_SYSTEM.into(),
            },
            &self.aes_config,
        )?;

        let request = self
            .client
            .post(self.url_provider.user())
            .headers(self.headers.get_map())
            .body(request_body);

        match request.send().await?.error_for_status() {
            Ok(response) => {
                let bytes = response.bytes().await?;
                Ok(aes_msgpack::from_slice(&bytes, &self.aes_config)?)
            }
            Err(err) => match err.status() {
                Some(StatusCode::UPGRADE_REQUIRED) => Err(ApiError::InvalidRequest(
                    strings::api::error::UPGRADE_REQUIRED.into(),
                )),
                _ => Err(ApiError::InvalidRequest(err.to_string())),
            },
        }
    }

    /// Performs a request to [`constants::url::sekai::user_auth`]
    ///
    /// This endpoint will create and respond with a login session for
    /// a specific user when given a valid login credential.
    ///
    /// It also contains relative URLs to the split suite master files along
    /// with the current data, asset, and multiplay versions.
    ///
    /// This function will store the session token as a header
    /// and respond with the entire response from the server.
    pub async fn user_login(
        &mut self,
        user_id: usize,
        credential: String,
    ) -> Result<UserAuthResponse, ApiError> {
        let request_body = aes_msgpack::into_vec(
            &UserAuthRequest {
                credential,
                device_id: None,
            },
            &self.aes_config,
        )?;

        let request = self
            .client
            .put(self.url_provider.user_auth(user_id))
            .headers(self.headers.get_map())
            .body(request_body);

        match request.send().await?.error_for_status() {
            Ok(response) => {
                // parse body
                let bytes = response.bytes().await?;
                let auth_response: UserAuthResponse =
                    aes_msgpack::from_slice(&bytes, &self.aes_config)?;

                // insert session token
                self.headers
                    .insert_str(header::name::SESSION_TOKEN, &auth_response.session_token)?;
                self.headers
                    .insert_str(header::name::ASSET_VERSION, &auth_response.asset_version)?;
                self.headers
                    .insert_str(header::name::DATA_VERSION, &auth_response.data_version)?;

                Ok(auth_response)
            }
            Err(err) => match err.status() {
                Some(StatusCode::NOT_FOUND) => Err(ApiError::InvalidRequest(
                    strings::api::error::NOT_FOUND_USER_AUTH.into(),
                )),
                _ => Err(ApiError::InvalidRequest(err.to_string())),
            },
        }
    }

    /// Performs a request to [`constants::url::sekai::ASSETBUNDLE_INFO`]
    ///
    /// This endpoint responds with information about the assetbundles available to download
    /// for the asset version provided to it.
    ///
    /// This endpoint requires that the cloudfront cookies have been set.
    ///
    /// Returns the assetbundle info provided by the endpoint.
    pub async fn get_assetbundle_info(
        &self,
        asset_version: &str,
        asstbundle_host_hash: &str,
    ) -> Result<AssetbundleInfo, ApiError> {
        let request = self
            .client
            .get(self.url_provider.assetbundle_info(
                asstbundle_host_hash,
                asset_version,
                &self.app.platform,
            ))
            .headers(self.headers.get_map());

        match request.send().await?.error_for_status() {
            Ok(response) => {
                // parse body
                let bytes = response.bytes().await?;
                Ok(aes_msgpack::from_slice(&bytes, &self.aes_config)?)
            }
            Err(err) => match err.status() {
                Some(StatusCode::FORBIDDEN) => Err(ApiError::InvalidRequest(
                    strings::api::error::FORBIDDEN_ASSETBUNDLE_INFO.into(),
                )),
                _ => Err(ApiError::InvalidRequest(err.to_string())),
            },
        }
    }

    /// Performs a request to download an assetbundle.
    ///
    ///
    /// This endpoint requires that the cloudfront cookies have been set.
    ///
    /// Returns a Vec of bytes, which is the assetbundle data.
    pub async fn get_assetbundle(
        &self,
        asset_version: &str,
        asset_hash: &str,
        assetbundle_host_hash: &str,
        bundle_name: &str,
    ) -> Result<Vec<u8>, ApiError> {
        let request = self
            .client
            .get(self.url_provider.assetbundle(
                assetbundle_host_hash,
                &self.url_provider.assetbundle_path(
                    asset_version,
                    asset_hash,
                    &self.app.platform,
                    bundle_name,
                ),
            ))
            .headers(self.headers.get_map());

        match request.send().await?.error_for_status() {
            Ok(response) => {
                // parse body
                Ok(response.bytes().await?.to_vec())
            }
            Err(err) => Err(ApiError::InvalidRequest(err.to_string())),
        }
    }

    /// Performs a request to [`constants::url::sekai::SYSTEM`]
    ///
    /// This endpoint reports information about the current status of the game server
    /// including available app and asset versions.
    ///
    /// This endpoint requires that the cloudfront cookies have been set.
    ///
    /// This function responds with this information
    pub async fn get_system(&self) -> Result<SystemInfo, ApiError> {
        let request = self
            .client
            .get(self.url_provider.system())
            .headers(self.headers.get_map());

        match request.send().await?.error_for_status() {
            Ok(response) => {
                // parse body
                let bytes = response.bytes().await?;
                Ok(aes_msgpack::from_slice(&bytes, &self.aes_config)?)
            }
            Err(err) => Err(ApiError::InvalidRequest(err.to_string())),
        }
    }

    /// Performs a request to download a suitemasterfile.
    ///
    /// The suitemasterfile endpoint is used for download split suite master files.
    ///
    /// These files contain information about what character cards and gacha banners exist among many other things.
    ///
    /// This function will, if successful, return bytes representing an encrypted suitemasterfile.
    pub async fn get_suitemasterfile(&self, file_path: &str) -> Result<Vec<u8>, ApiError> {
        let request = self
            .client
            .get(self.url_provider.suitemasterfile(file_path))
            .headers(self.headers.get_map());

        match request.send().await?.error_for_status() {
            Ok(response) => {
                // parse body
                let bytes = response.bytes().await?;
                Ok(bytes.to_vec())
            }
            Err(err) => Err(ApiError::InvalidRequest(err.to_string())),
        }
    }

    /// Performs a request to download a suitemasterfile.
    ///
    /// The suitemasterfile endpoint is used for download split suite master files.
    ///
    /// These files contain information about what character cards and gacha banners exist among many other things.
    ///
    /// This function will, if successful, return a ``serde_json::Value`` representing a decrypted suitemasterfile.
    pub async fn get_suitemasterfile_as_value(&self, file_path: &str) -> Result<Value, ApiError> {
        let bytes = self.get_suitemasterfile(file_path).await?;
        Ok(aes_msgpack::from_slice(&bytes, &self.aes_config)?)
    }

    /// Performs a request to get a user's account inherit details.
    ///
    /// If execute is true, the account will be inherited and the returned UserInherit will contain an authentication credential JWT.
    ///
    /// This credential is used for performing authenticated requests.
    pub async fn get_user_inherit(
        &self,
        inherit_id: &str,
        password: &str,
        execute: bool,
    ) -> Result<UserInherit, ApiError> {
        let mut headers = self.headers.get_map();

        // create X-Inherit-Id-Verify-Token header
        let jwt_payload = UserInheritJWT {
            inherit_id: inherit_id.into(),
            password: password.into(),
        };
        let token_str = jwt_payload.sign_with_key(&self.jwt_key)?;
        headers.append(INHERIT_TOKEN, HeaderValue::from_str(&token_str)?);

        let request = self
            .client
            .post(self.url_provider.inherit(inherit_id, execute))
            .headers(headers);

        match request.send().await?.error_for_status() {
            Ok(response) => {
                // parse body
                let bytes = response.bytes().await?;
                Ok(aes_msgpack::from_slice(&bytes, &self.aes_config)?)
            }
            Err(err) => match err.status() {
                Some(StatusCode::NOT_FOUND) | Some(StatusCode::FORBIDDEN) => {
                    Err(ApiError::InvalidRequest(
                        strings::api::error::INVALID_INHERIT_CREDENTIALS.into(),
                    ))
                }
                _ => Err(ApiError::InvalidRequest(err.to_string())),
            },
        }
    }

    /// Gets a user's suite data as a [`serde_json::Value`].
    ///
    /// This is an authenticated request, and therefore requires [`Self::user_login`]
    /// to have been previously successfully called.
    pub async fn get_user_suite(&self, user_id: usize) -> Result<Value, ApiError> {
        let request = self
            .client
            .get(self.url_provider.user_suite(user_id))
            .headers(self.headers.get_map());

        match request.send().await?.error_for_status() {
            Ok(response) => {
                // parse body
                let bytes = response.bytes().await?;
                Ok(aes_msgpack::from_slice(&bytes, &self.aes_config)?)
            }
            Err(err) => Err(ApiError::InvalidRequest(err.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{api::url::test_provider::TestUrlProvider, enums::Server, models::api::AppVersion};

    const SIGNATURE_COOKIE_VALUE: &str = "signature_cookie";
    const SUITEMASTER_FILE_CONTENT: &str = "HELLO SUITEMASTER FILE!!!";
    const SUITEMASTER_FILE_PATH: &str = "suitemasterfile/1.0.0/test_file";

    fn get_aes_config() -> AesConfig {
        Server::Japan.get_aes_config()
    }

    fn get_jwt_key() -> Hmac<Sha256> {
        Server::Japan.get_jwt_key()
    }

    async fn get_client(server_url: String) -> SekaiClient<TestUrlProvider> {
        SekaiClient::new_with_url_provider(
            "3.9".to_string(),
            "393939".to_string(),
            Platform::Android,
            get_aes_config(),
            get_jwt_key(),
            TestUrlProvider::new(server_url),
        )
        .await
        .unwrap()
    }

    async fn get_server() -> mockito::ServerGuard {
        let mut server = mockito::Server::new_async().await;

        server
            .mock("POST", "/api/signature")
            .with_status(200)
            .with_header(header::name::SET_COOKIE, SIGNATURE_COOKIE_VALUE)
            .create_async()
            .await;

        // suitemaster file
        let encrypted_suitemaster_file =
            aes_msgpack::into_vec(&SUITEMASTER_FILE_CONTENT.as_bytes(), &get_aes_config()).unwrap();
        server
            .mock("GET", "/api/suitemasterfile/1.0.0/test_file")
            .with_status(200)
            .with_body(encrypted_suitemaster_file)
            .create_async()
            .await;

        server
    }

    #[tokio::test]
    async fn test_get_system() {
        let mut server = get_server().await;
        let client = get_client(server.url()).await;

        // create body
        let mock_system_info = SystemInfo {
            server_date: 1730780277695,
            timezone: "Asia/Tokyo".into(),
            profile: "production".into(),
            maintenance_status: "maintenance_out".into(),
            app_versions: vec![AppVersion {
                system_profile: "production".into(),
                app_version: "4.0.5".into(),
                multi_play_version: "miku".into(),
                asset_version: "4.0.5.10".into(),
                app_version_status: "available".into(),
            }],
        };
        let mock_body = aes_msgpack::into_vec(&mock_system_info, &client.aes_config).unwrap();

        let mock = server
            .mock("GET", "/api/system")
            .with_status(200)
            .with_body(&mock_body)
            .create_async()
            .await;

        let result = client.get_system().await;

        mock.assert();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), mock_system_info)
    }

    #[tokio::test]
    async fn test_issue_signature() {
        let server = get_server().await;

        let client = get_client(server.url()).await;

        assert_eq!(
            client.headers.0.get(header::name::COOKIE).unwrap(),
            SIGNATURE_COOKIE_VALUE
        )
    }

    #[tokio::test]
    async fn test_get_suitemasterfile() {
        let server = get_server().await;
        let client = get_client(server.url()).await;

        let response = client
            .get_suitemasterfile(SUITEMASTER_FILE_PATH)
            .await
            .unwrap();

        // compare
        let encrypted_suitemaster_file =
            aes_msgpack::into_vec(&SUITEMASTER_FILE_CONTENT.as_bytes(), &get_aes_config()).unwrap();
        assert_eq!(
            response, encrypted_suitemaster_file,
            "response from server and computed value should be the same"
        );
    }

    #[tokio::test]
    async fn test_get_suitemasterfile_as_value() {
        let server = get_server().await;
        let client = get_client(server.url()).await;

        let response = client
            .get_suitemasterfile_as_value(SUITEMASTER_FILE_PATH)
            .await
            .unwrap();

        // get value to compare response with
        let aes_config = get_aes_config();
        let encrypted_suitemaster_file =
            aes_msgpack::into_vec(&SUITEMASTER_FILE_CONTENT.as_bytes(), &aes_config).unwrap();
        let decrypted_suitemaster_file: Value =
            aes_msgpack::from_slice(&encrypted_suitemaster_file, &aes_config).unwrap();

        assert_eq!(
            response, decrypted_suitemaster_file,
            "response from server and computed value should be the same"
        )
    }
}

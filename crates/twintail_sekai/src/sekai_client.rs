use super::{headers::Headers, url::UrlProvider};
use crate::{
    Error,
    headers::{header_name, header_value},
    models::{
        AppInfo, AssetbundleInfo, GameVersion, SystemInfo, UserAuthRequest, UserAuthResponse,
        UserInherit, UserInheritJWT, UserRequest, UserSignup,
    },
};
use hmac::Hmac;
use jwt::SignWithKey;
use reqwest::{Client, StatusCode, header::HeaderValue};
use serde_json::Value;
use sha2::Sha256;
use twintail_common::{
    crypto::{aes::AesConfig, aes_msgpack},
    models::{OptionalBuilder, enums::Platform},
};

mod error_string {
    pub const UPGRADE_REQUIRED: &str =
        "app version and/or hash are for an older version of the app. use newer values";
    pub const INVALID_HASH_VERSION: &str = "invalid app version and/or hash provided";
    pub const SET_COOKIE_NOT_FOUND: &str = "set-cookie header not found";
    pub const FORBIDDEN_ASSETBUNDLE_INFO: &str = "invalid or outdated asset version provided";
    pub const INVALID_INHERIT_CREDENTIALS: &str = "could not find any account with the provided transfer id or password. ensure that both values are correct";
    pub const NOT_FOUND_USER_AUTH: &str = "(404: not found) error when logging in to an account. ensure that the app version and hash values are correct";
    pub const GET_APP_INFO: &str = "error when attempting to retrieve the latest app info";
}

/// An API client that interfaces with the game's servers, providing various functions to query endpoints.
pub struct SekaiClient<T: UrlProvider> {
    aes_config: AesConfig,
    app_hash: String,
    app_version: String,
    client: Client,
    headers: Headers,
    jwt_key: Hmac<Sha256>,
    pub platform: Platform,
    pub url_provider: T,
}

impl<T: UrlProvider> SekaiClient<T> {
    /// Creates a new SekaiClient that uses a specific url provider.
    pub async fn new(
        app_hash: String,
        app_version: String,
        aes_config: AesConfig,
        jwt_key: Hmac<Sha256>,
        platform: Platform,
        url_provider: T,
    ) -> Result<Self, Error> {
        let headers = Headers::builder()?
            .version(&app_version)
            .hash(&app_hash)
            .platform(&platform)
            .build()?;

        let mut client = Self {
            headers,
            client: Client::new(),
            platform,
            app_version,
            app_hash,
            aes_config,
            jwt_key,
            url_provider,
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
    async fn issue_signature(&mut self) -> Result<(), Error> {
        let url = self
            .url_provider
            .issue_signature()
            .ok_or(Error::MissingUrl("issue_signature".to_string()))?;

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
                    .remove(header_name::SET_COOKIE)
                    .ok_or(Error::InvalidRequest(
                        error_string::SET_COOKIE_NOT_FOUND.into(),
                    ))?;
                self.headers.insert(header_name::COOKIE, set_cookie_header);
                Ok(())
            }
            Err(err) => Err(Error::InvalidRequest(err.to_string())),
        }
    }

    /// Performs a request to [`constants::url::sekai::GAME_VERSION`].
    ///
    /// This endpoint will respond with info about the game version that the URL corresponds to.
    /// The version, hash, and platform values that this [`SekaiClient`] was created with determine this.
    ///
    /// Returns the parsed GameVersion data if it was found.
    pub async fn get_game_version(&self) -> Result<GameVersion, Error> {
        let request = self
            .client
            .get(
                self.url_provider
                    .game_version(&self.app_version, &self.app_hash),
            )
            .headers(self.headers.get_map());

        match request.send().await?.error_for_status() {
            Ok(response) => {
                let bytes = response.bytes().await?;
                Ok(aes_msgpack::from_slice(&bytes, &self.aes_config)?)
            }
            Err(err) => match err.status() {
                Some(StatusCode::FORBIDDEN) => Err(Error::InvalidRequest(
                    error_string::INVALID_HASH_VERSION.into(),
                )),
                _ => Err(Error::InvalidRequest(err.to_string())),
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
    pub async fn user_signup(&self) -> Result<UserSignup, Error> {
        let request_body = aes_msgpack::into_vec(
            &UserRequest {
                platform: self.platform,
                device_model: header_value::DEVICE_MODEL.into(),
                operating_system: header_value::OPERATING_SYSTEM.into(),
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
                Some(StatusCode::UPGRADE_REQUIRED) => {
                    Err(Error::InvalidRequest(error_string::UPGRADE_REQUIRED.into()))
                }
                _ => Err(Error::InvalidRequest(err.to_string())),
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
    ) -> Result<UserAuthResponse, Error> {
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
                    .insert_str(header_name::SESSION_TOKEN, &auth_response.session_token)?;
                self.headers
                    .insert_str(header_name::ASSET_VERSION, &auth_response.asset_version)?;
                self.headers
                    .insert_str(header_name::DATA_VERSION, &auth_response.data_version)?;

                Ok(auth_response)
            }
            Err(err) => match err.status() {
                Some(StatusCode::NOT_FOUND) => Err(Error::InvalidRequest(
                    error_string::NOT_FOUND_USER_AUTH.into(),
                )),
                _ => Err(Error::InvalidRequest(err.to_string())),
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
        asset_hash: &str,
        asstbundle_host_hash: &str,
    ) -> Result<AssetbundleInfo, Error> {
        let request = self
            .client
            .get(self.url_provider.assetbundle_info(
                asstbundle_host_hash,
                asset_version,
                asset_hash,
                &self.platform,
            ))
            .headers(self.headers.get_map());

        match request.send().await?.error_for_status() {
            Ok(response) => {
                // parse body
                let bytes = response.bytes().await?;
                Ok(aes_msgpack::from_slice(&bytes, &self.aes_config)?)
            }
            Err(err) => match err.status() {
                Some(StatusCode::FORBIDDEN) => Err(Error::InvalidRequest(
                    error_string::FORBIDDEN_ASSETBUNDLE_INFO.into(),
                )),
                _ => Err(Error::InvalidRequest(err.to_string())),
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
    ) -> Result<Vec<u8>, Error> {
        let request = self
            .client
            .get(self.url_provider.assetbundle(
                assetbundle_host_hash,
                &self.url_provider.assetbundle_path(
                    asset_version,
                    asset_hash,
                    &self.platform,
                    bundle_name,
                ),
            ))
            .headers(self.headers.get_map());

        match request.send().await?.error_for_status() {
            Ok(response) => {
                // parse body
                Ok(response.bytes().await?.to_vec())
            }
            Err(err) => Err(Error::InvalidRequest(err.to_string())),
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
    pub async fn get_system(&self) -> Result<SystemInfo, Error> {
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
            Err(err) => Err(Error::InvalidRequest(err.to_string())),
        }
    }

    /// Performs a request to download a suitemasterfile.
    ///
    /// The suitemasterfile endpoint is used for download split suite master files.
    ///
    /// These files contain information about what character cards and gacha banners exist among many other things.
    ///
    /// This function will, if successful, return bytes representing an encrypted suitemasterfile.
    pub async fn get_suitemasterfile(&self, file_path: &str) -> Result<Vec<u8>, Error> {
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
            Err(err) => Err(Error::InvalidRequest(err.to_string())),
        }
    }

    /// Performs a request to download a suitemasterfile.
    ///
    /// The suitemasterfile endpoint is used for download split suite master files.
    ///
    /// These files contain information about what character cards and gacha banners exist among many other things.
    ///
    /// This function will, if successful, return a ``serde_json::Value`` representing a decrypted suitemasterfile.
    pub async fn get_suitemasterfile_as_value(&self, file_path: &str) -> Result<Value, Error> {
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
    ) -> Result<UserInherit, Error> {
        let mut headers = self.headers.get_map();

        // create X-Inherit-Id-Verify-Token header
        let jwt_payload = UserInheritJWT {
            inherit_id: inherit_id.into(),
            password: password.into(),
        };
        let token_str = jwt_payload.sign_with_key(&self.jwt_key)?;
        headers.append(
            header_name::INHERIT_TOKEN,
            HeaderValue::from_str(&token_str)?,
        );

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
                Some(StatusCode::NOT_FOUND) | Some(StatusCode::FORBIDDEN) => Err(
                    Error::InvalidRequest(error_string::INVALID_INHERIT_CREDENTIALS.into()),
                ),
                _ => Err(Error::InvalidRequest(err.to_string())),
            },
        }
    }

    /// Gets a user's suite data as a [`serde_json::Value`].
    ///
    /// This is an authenticated request, and therefore requires [`Self::user_login`]
    /// to have been previously successfully called.
    pub async fn get_user_suite(&self, user_id: usize) -> Result<Value, Error> {
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
            Err(err) => Err(Error::InvalidRequest(err.to_string())),
        }
    }

    /// Gets the game's current app hash & app version from
    /// [https://github.com/mos9527/sekai-apphash]
    pub async fn get_app_version(url_provider: &T) -> Result<AppInfo, Error> {
        let request = Client::new().get(url_provider.apphash());

        match request.send().await?.error_for_status() {
            Ok(response) => {
                // parse body
                let bytes = response.bytes().await?;
                let app_hash = serde_json::from_slice(&bytes)?;
                Ok(app_hash)
            }
            Err(err) => Err(Error::InvalidRequest(format!(
                "{}: {}",
                error_string::GET_APP_INFO,
                err.to_string()
            ))),
        }
    }
}

pub struct SekaiClientBuilder<T: UrlProvider> {
    aes_config: AesConfig,
    app_hash: Option<String>,
    app_version: Option<String>,
    jwt_key: Hmac<Sha256>,
    platform: Platform,
    url_provider: T,
}

impl<T: UrlProvider> OptionalBuilder for SekaiClientBuilder<T> {}

impl<T: UrlProvider> SekaiClientBuilder<T> {
    /// Create a new SekaiClient builder
    pub fn new(
        aes_config: AesConfig,
        jwt_key: Hmac<Sha256>,
        platform: Platform,
        url_provider: T,
    ) -> Self {
        Self {
            aes_config,
            app_hash: None,
            app_version: None,
            jwt_key,
            platform,
            url_provider,
        }
    }

    /// Set the SekaiClient's app hash
    pub fn app_hash(mut self, hash: String) -> Self {
        self.app_hash = Some(hash);
        self
    }

    /// Set the SekaiClient's app version
    pub fn app_version(mut self, version: String) -> Self {
        self.app_version = Some(version);
        self
    }

    /// Build the SekaiClient
    ///
    /// If app_hash or app_version were not set,
    /// the values will be fetched from the internet.
    pub async fn build(self) -> Result<SekaiClient<T>, Error> {
        let (app_hash, app_version) =
            if let (Some(app_hash), Some(app_version)) = (&self.app_hash, &self.app_version) {
                (app_hash.clone(), app_version.clone())
            } else {
                let app_info = SekaiClient::get_app_version(&self.url_provider).await?;
                (
                    self.app_hash.unwrap_or(app_info.app_hash),
                    self.app_version.unwrap_or(app_info.app_version),
                )
            };

        SekaiClient::new(
            app_hash,
            app_version,
            self.aes_config,
            self.jwt_key,
            self.platform,
            self.url_provider,
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{models::AppVersion, url::test_provider::TestUrlProvider};
    use twintail_common::models::enums::Server;

    const SIGNATURE_COOKIE_VALUE: &str = "signature_cookie";
    const SUITEMASTER_FILE_CONTENT: &str = "HELLO SUITEMASTER FILE!!!";
    const SUITEMASTER_FILE_PATH: &str = "suitemasterfile/1.0.0/test_file";

    fn get_aes_config() -> AesConfig {
        Server::Japan.get_aes_config()
    }

    fn get_jwt_key() -> Hmac<Sha256> {
        Server::Japan.get_jwt_key()
    }

    fn get_app_hash() -> AppInfo {
        AppInfo {
            app_hash: "example-app-hash".into(),
            app_version: "10.0.20".into(),
        }
    }

    async fn get_client(server_url: String) -> SekaiClient<TestUrlProvider> {
        SekaiClient::new(
            "3.9".to_string(),
            "393939".to_string(),
            get_aes_config(),
            get_jwt_key(),
            Platform::Android,
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
            .with_header(header_name::SET_COOKIE, SIGNATURE_COOKIE_VALUE)
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
            .mock("GET", "/apphash")
            .with_status(200)
            .with_body(serde_json::to_string(&get_app_hash()).unwrap())
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
            client.headers.0.get(header_name::COOKIE).unwrap(),
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

    #[tokio::test]
    async fn test_get_app_info() {
        let server = get_server().await;

        let response = SekaiClient::get_app_version(&TestUrlProvider::new(server.url()))
            .await
            .unwrap();

        assert_eq!(response, get_app_hash());
    }
}

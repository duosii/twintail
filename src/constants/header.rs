// Headers for requests

// header names
pub mod name {
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
}

// header values
pub mod value {
    pub const CONTENT_TYPE: &str = "application/octet-stream";
    pub const ACCEPT: &str = "application/octet-stream";
    pub const USER_AGENT: &str = "UnityPlayer/2022.3.21f1 (UnityWebRequest/1.0, libcurl/8.5.0-DEV)";
    pub const INSTALL_ID: &str = "3efd7166-11b2-4b3e-2f08-94b0e16f76e8";
    pub const DEVICE_MODEL: &str = "39phone";
    pub const OPERATING_SYSTEM: &str = "39os";
    pub const UNITY_VERSION: &str = "2022.3.21f1";
}

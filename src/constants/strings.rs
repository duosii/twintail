pub mod crypto {
    pub mod encrypt {
        pub const PROCESS: &str = "Encrypting";
        pub const PROCESSED: &str = "encrypted";
    }
    pub mod decrypt {
        pub const PROCESS: &str = "Decrypting";
        pub const PROCESSED: &str = "decrypted";
    }
}

pub mod api {
    pub mod error {
        pub const UPGRADE_REQUIRED: &str =
            "app version and/or hash are for an older version of the app. use newer values";
        pub const INVALID_HASH_VERSION: &str = "invalid app version and/or hash provided";
        pub const SET_COOKIE_NOT_FOUND: &str = "set-cookie header not found";
        pub const FORBIDDEN_ASSETBUNDLE_INFO: &str = "invalid or outdated asset version provided";
    }
}

pub mod command {
    pub const COMMUNICATING: &str = "Communicating with game servers...";
    pub const PATHS_SAVED_TO: &str = "Paths saved to ";

    pub mod error {
        pub const NO_RECENT_VERSION: &str = "most recent game version not found";
    }
}

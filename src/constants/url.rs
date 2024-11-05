// constants for sekai URLs
pub mod sekai {
    pub mod issue {
        pub const HOST: &str = "https://issue.sekai.colorfulpalette.org";
        pub const SIGNATURE: &str = "/api/signature";
    }

    pub mod game_version {
        pub const HOST: &str = "https://game-version.sekai.colorfulpalette.org";
    }

    pub mod game {
        pub const HOST: &str = "https://production-game-api.sekai.colorfulpalette.org";
        pub const USER: &str = "/api/user";
        pub const USER_AUTH: &str = "/api/user";
        pub const SYSTEM: &str = "/api/system";
    }

    pub mod assetbundle {
        pub const INFO: &str = "/api/version";
    }
}

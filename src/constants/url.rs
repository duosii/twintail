// constants for sekai URLs
pub mod sekai {
    pub mod issue {
        pub const JAPAN_HOST: &str = "https://issue.sekai.colorfulpalette.org";
        pub const SIGNATURE: &str = "/api/signature";
    }

    pub mod game_version {
        pub const JAPAN_HOST: &str = "https://game-version.sekai.colorfulpalette.org";
        pub const GLOBAL_HOST: &str = "https://game-version.sekai-en.com";
    }

    pub mod game {
        pub const JAPAN_HOST: &str = "https://production-game-api.sekai.colorfulpalette.org";
        pub const GLOBAL_HOST: &str = "https://n-production-game-api.sekai-en.com";
        pub const API: &str = "/api";
        pub const USER: &str = "/api/user";
        pub const USER_AUTH: &str = "/api/user";
        pub const SYSTEM: &str = "/api/system";
        pub const INHERIT: &str = "/api/inherit/user";
        pub const USER_SUITE: &str = "/api/suite/user";
    }

    pub mod assetbundle {
        pub const INFO: &str = "/api/version";
    }
}

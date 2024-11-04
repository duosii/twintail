// constants for sekai URLs
pub mod sekai {
    pub const ISSUE_SIGNATURE: &str = "https://issue.sekai.colorfulpalette.org/api/signature";
    pub const GAME_VERSION: &str = "https://game-version.sekai.colorfulpalette.org";
    pub const USER: &str = "https://production-game-api.sekai.colorfulpalette.org/api/user";
    pub fn user_auth(user_id: usize) -> String {
        format!("https://production-game-api.sekai.colorfulpalette.org/api/user/{}/auth?refreshUpdatedResources=False", user_id)
    }
}

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

pub mod command {
    pub const DOWNLOADING: &str = "Downloading files...";
    pub const DOWNLOADED: &str = "downloaded";

    // suite
    pub const SUITE_VERSION: &str = "[Suite Data Version]:";
    pub const SUITE_PROCESSING: &str = "Processing suitemaster files...";
    pub const SUITE_SAVING: &str = "Saving encrypted suitemaster files...";
    pub const SUITE_DECRYPTING: &str = "Decrypting suitemaster files...";

    // ab-info
    pub const RETRIEVING_AB_INFO: &str = "Retrieving assetbundle info...";
    pub const COMMUNICATING: &str = "Communicating with game servers...";
    pub const PATHS_SAVED_TO: &str = "Paths saved to ";

    // assetbundle
    pub const INVALID_RE: &str =
        "Invalid filter regular expression provided. No filter will be applied.";

    // extract hash
    pub const EXTRACTING: &str = "Extracting version and hash from file...";
    pub const EXTRACT_FAIL: &str = "No version/hash found in the provided file.";
    pub const EXTRACT_SUCCESS: &str = "Successfully extracted info from the apk.";
    pub const EXTRACT_VERSION: &str = "[App Version]:";
    pub const EXTRACT_HASH: &str = "[App Hash]:";
    pub const EXTRACT_MISSING: &str = "Not Found";

    // fetch save
    pub const INHERIT_USER_DETAILS: &str = "Player Details:";
    pub const INHERIT_USER_ID: &str = "ID:";
    pub const INHERIT_USER_NAME: &str = "Name:";
    pub const INHERIT_USER_RANK: &str = "Rank:";
    pub const INHERIT_CONTINUE_CONFIRM: &str = "⚠️ Do you want to continue?\nThis action will transfer your account from its original device.\nPlease type 'y' to confirm or 'N' to cancel: ";
    pub const INHERIT_CANCELLED: &str = "Save download was cancelled";
    pub const INHERIT_NO_CREDENTIAL: &str =
        "The credential contained in the inherit data was None.";
    pub const INHERIT_GETTING_USER_DATA: &str = "Getting player information...";
    pub const INHERIT_LOGGING_IN: &str = "Logging into your account...";
    pub const INHERIT_GETTING_SAVE_DATA: &str = "Retrieving your account's save data...";
    pub const INHERIT_FINISH_WARNING: &str = "Don't forget to use the same transfer ID and password to transfer your account back to its original device.";
}

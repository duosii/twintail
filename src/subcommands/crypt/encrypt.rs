use super::{crypt_assetbundle, CryptArgs, CryptStrings};
use crate::{constants::strings, crypto::assetbundle::CryptOperation, error::CommandError};

#[derive(Debug, clap::Args)]
pub struct EncryptArgs {
    /// If the input is a directory, whether to recursively decrypt valid files in that directory
    #[arg(long, short, default_value_t = false)]
    pub recursive: bool,

    /// The maximum number of files to encrypt simultaneously
    #[arg(long, short, default_value_t = crate::utils::available_parallelism())]
    pub concurrent: usize,

    /// Path to the file or directory to encrypt
    pub in_path: String,

    /// Path to a directory or file to output to. If not provided, files are encrypted in-place.
    pub out_path: Option<String>,
}

/// Encrypts a file/folder using the provided arguments.
pub async fn encrypt(args: &EncryptArgs) -> Result<(), CommandError> {
    crypt_assetbundle(CryptArgs {
        in_path: &args.in_path,
        recursive: args.recursive,
        concurrent: args.concurrent,
        operation: CryptOperation::Encrypt,
        strings: CryptStrings {
            process: strings::crypto::encrypt::PROCESS,
            processed: strings::crypto::encrypt::PROCESSED,
        },
        out_path: &args.out_path,
    })
    .await
}

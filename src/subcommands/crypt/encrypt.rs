use super::{crypt_assetbundle, CryptArgs};
use crate::{constants::strings, crypto::assetbundle::CryptOperation, error::CommandError};

#[derive(Debug, clap::Args)]
pub struct EncryptArgs {
    /// If the input is a directory, whether to recursively decrypt valid files in that directory
    #[arg(long, short, default_value_t = false)]
    pub recursive: bool,

    /// The maximum number of files to encrypt simultaneously
    #[arg(long, short, default_value_t = 12)]
    pub concurrent: usize,

    /// Path to the file or directory to encrypt
    pub in_path: String,

    /// Path to a directory or file to output to. If not provided, files are encrypted in-place.
    pub out_path: Option<String>,
}

pub async fn encrypt(args: &EncryptArgs) -> Result<(), CommandError> {
    crypt_assetbundle(CryptArgs {
        in_path: &args.in_path,
        recursive: args.recursive,
        concurrent: args.concurrent,
        operation: CryptOperation::Encrypt,
        strings: super::CryptStrings {
            process: &strings::crypto::encrypt::process,
            processed: &strings::crypto::encrypt::processed,
        },
        out_path: &args.out_path,
    })
    .await
}

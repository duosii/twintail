use crate::{crypto::assetbundle::CryptOperation, error::CommandError, utils::crypt_assetbundle};
use clap::Args;

#[derive(Debug, Args)]
pub struct EncryptArgs {
    /// If the input is a directory, whether to recursively decrypt valid files in that directory
    #[arg(long, short, default_value_t = false)]
    pub recursive: bool,

    /// The maximum number of files to encrypt simultaneously
    #[arg(long, short, default_value_t = 12)]
    pub concurrent: usize,

    /// Path to the file or directory to decrypt
    pub in_path: String,

    /// Path to a directory or file to output to. If not provided, files are encrypted in-place.
    pub out_path: Option<String>,
}

pub async fn encrypt(args: &EncryptArgs) -> Result<(), CommandError> {
    crypt_assetbundle(
        &args.in_path,
        args.recursive,
        args.concurrent,
        CryptOperation::Encrypt,
        &args.out_path,
    )
    .await
}

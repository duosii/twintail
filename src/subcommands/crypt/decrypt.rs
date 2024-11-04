use super::{crypt_assetbundle, CryptArgs, CryptStrings};
use crate::{constants::strings, crypto::assetbundle::CryptOperation, error::CommandError};
use clap::Args;

#[derive(Debug, Args)]
pub struct DecryptArgs {
    /// If the input is a directory, whether to recursively decrypt valid files in that directory
    #[arg(long, short, default_value_t = false)]
    pub recursive: bool,

    /// The maximum number of files to decrypt simultaneously
    #[arg(long, short, default_value_t = 12)]
    pub concurrent: usize,

    /// Path to the file or directory to decrypt
    pub in_path: String,

    /// Path to a directory or file to output to. If not provided, files are decrypted in-place
    pub out_path: Option<String>,
}

/// Decrypts a file/folder using the provided arguments.
pub async fn decrypt(args: &DecryptArgs) -> Result<(), CommandError> {
    crypt_assetbundle(CryptArgs {
        in_path: &args.in_path,
        recursive: args.recursive,
        concurrent: args.concurrent,
        operation: CryptOperation::Decrypt,
        strings: CryptStrings {
            process: strings::crypto::decrypt::PROCESS,
            processed: strings::crypto::decrypt::PROCESSED,
        },
        out_path: &args.out_path,
    })
    .await
}

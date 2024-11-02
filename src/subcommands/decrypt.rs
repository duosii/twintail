use crate::{crypto::assetbundle, error::CommandError};
use clap::Args;
use tokio::{
    fs::{self, File},
    io::{AsyncWriteExt, BufReader},
};

#[derive(Debug, Args)]
pub struct DecryptArgs {
    /// If the input is a directory, whether to recursively decrypt valid files in that directory.
    #[arg(long, short, default_value_t = false)]
    pub recursive: bool,

    /// Path to the file or directory to decrypt.
    pub in_path: String,

    /// Path to a directory or file to output to.
    pub out_path: String,
}

pub async fn decrypt(args: &DecryptArgs) -> Result<(), CommandError> {
    if !(fs::try_exists(&args.in_path).await?) {
        return Err(CommandError::InvalidPath(args.in_path.to_string()));
    }

    // decrypt
    let in_file = File::open(&args.in_path).await?;
    let mut reader = BufReader::new(in_file);
    let decrypted = assetbundle::decrypt(&mut reader).await?;

    // write
    let mut out_file = File::options()
        .write(true)
        .create(true)
        .open(&args.out_path)
        .await?;
    out_file.write(&decrypted).await?;

    Ok(())
}

use std::path::PathBuf;

use crate::{
    crypto::assetbundle,
    error::{AssetbundleError, CommandError},
    utils::scan_directory,
};
use clap::Args;
use futures::stream::{self, StreamExt};
use tokio::{
    fs::{create_dir_all, File},
    io::{AsyncWriteExt, BufReader},
    time::Instant,
};

#[derive(Debug, Args)]
pub struct EncryptArgs {
    /// If the input is a directory, whether to recursively decrypt valid files in that directory.
    #[arg(long, short, default_value_t = false)]
    pub recursive: bool,

    /// The maximum number of files to encrypt simultaneously.
    #[arg(long, short, default_value_t = 12)]
    pub concurrent: usize,

    /// Path to the file or directory to decrypt.
    pub in_path: String,

    /// Path to a directory or file to output to.
    pub out_path: String,
}

/// Encrypts a file at the input path into the output path.
pub async fn encrypt_file(in_path: PathBuf, out_path: PathBuf) -> Result<(), AssetbundleError> {
    // decrypt
    let in_file = File::open(in_path).await?;
    let mut reader = BufReader::new(in_file);
    let decrypted = assetbundle::encrypt(&mut reader).await?;

    // create parent folders if they do not exist
    if let Some(parent) = out_path.parent() {
        create_dir_all(parent).await?;
    }
    let mut out_file = File::options()
        .write(true)
        .create(true)
        .open(out_path)
        .await?;
    out_file.write(&decrypted).await?;

    Ok(())
}

pub async fn encrypt(args: &EncryptArgs) -> Result<(), CommandError> {
    let in_path = PathBuf::from(&args.in_path);
    let out_path = PathBuf::from(&args.out_path);

    // get the paths that we need to decrypt
    let paths = if in_path.is_dir() {
        println!("Searching for files to encrypt...");
        let paths = scan_directory(in_path.clone(), args.recursive).await?;
        println!("Found {} file(s) to encrypt.", paths.len());
        paths
    } else {
        println!("Encrypting {:?}", &in_path);
        vec![in_path.clone()]
    };

    // asynchronously decrypt the files
    let encrypt_start = Instant::now();

    let encrypt_results: Vec<Result<(), AssetbundleError>> = stream::iter(paths)
        .map(|path| {
            let relative = path.strip_prefix(&in_path).ok().unwrap_or(&path);
            let out = out_path.join(relative);
            encrypt_file(path, out)
        })
        .buffer_unordered(args.concurrent)
        .collect()
        .await;
    let success_count = encrypt_results
        .iter()
        .filter(|&result| result.is_ok())
        .count();

    println!(
        "Successfully encrypted {} files in {:?}.",
        success_count,
        Instant::now().duration_since(encrypt_start)
    );

    Ok(())
}

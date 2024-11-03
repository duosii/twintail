use futures::stream::{self, StreamExt};
use std::{collections::VecDeque, path::PathBuf};
use tokio::{fs, time::Instant};

use crate::{
    crypto::assetbundle::{crypt_file, CryptOperation},
    error::{AssetbundleError, CommandError},
};

/// Scans a directory and returns all paths to files inside of that directory.
///
/// If recursive is true and a directroy is found, it will also scan that directory.
pub async fn scan_directory(
    dir_path: PathBuf,
    recursive: bool,
) -> Result<Vec<PathBuf>, tokio::io::Error> {
    let mut paths = Vec::new();
    let mut dirs_to_scan = VecDeque::new();
    dirs_to_scan.push_back(dir_path.clone());

    while let Some(scan_dir) = dirs_to_scan.pop_front() {
        if let Some(mut read_dir) = fs::read_dir(scan_dir).await.ok() {
            while let Ok(Some(path)) = read_dir.next_entry().await {
                let path = path.path();

                if path.is_dir() {
                    if recursive {
                        dirs_to_scan.push_back(path);
                    }
                } else {
                    paths.push(path);
                }
            }
        }
    }

    Ok(paths)
}

/// Decrypts or encrypts a file/directory that is/contains an assetbundle.
pub async fn crypt_assetbundle(
    in_path: &str,
    recursive: bool,
    concurrent: usize,
    operation: CryptOperation,
    out_path: &Option<String>,
) -> Result<(), CommandError> {
    let in_path = PathBuf::from(in_path);
    let out_path = if let Some(out) = out_path {
        PathBuf::from(out)
    } else {
        in_path.clone()
    };
    let in_place = in_path == out_path;

    // get the paths that we need to decrypt
    let in_paths = if in_path.is_dir() {
        println!("Searching for files to process...");
        let paths = scan_directory(in_path.clone(), recursive).await?;
        println!("Found {} file(s). Processing...", paths.len());
        paths
    } else {
        println!("Processing {:?}", &in_path);
        vec![in_path.clone()]
    };

    // asynchronously decrypt the files
    let decrypt_start = Instant::now();

    // compute paths
    let in_out_paths: Vec<(PathBuf, PathBuf)> = in_paths
        .into_iter()
        .map(|path| {
            if in_place {
                (path.clone(), path)
            } else {
                let relative = path.strip_prefix(&in_path).ok().unwrap_or(&path);
                let out = out_path.join(relative);
                (path, out)
            }
        })
        .collect();

    let decrypt_result: Vec<Result<(), AssetbundleError>> = stream::iter(&in_out_paths)
        .map(|paths| crypt_file(&paths.0, &paths.1, &operation))
        .buffer_unordered(concurrent)
        .collect()
        .await;
    let success_count = decrypt_result
        .iter()
        .filter(|&result| result.is_ok())
        .count();

    println!(
        "Successfully processed {} / {} files in {:?}.",
        success_count,
        in_out_paths.len(),
        Instant::now().duration_since(decrypt_start)
    );

    Ok(())
}

use std::{collections::VecDeque, path::PathBuf};

use tokio::fs;

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

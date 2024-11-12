use std::{collections::VecDeque, path::PathBuf};
use tokio::fs;

/// Provided a path, will return all files related to that path.
/// 1. If the path corresponds to an individual file, only that file's path will be returned.
/// 2. If it is a directory, all files within that directory will be returned (recursive if given).
pub async fn scan_path(path: &PathBuf, recursive: bool) -> Result<Vec<PathBuf>, tokio::io::Error> {
    let mut paths = Vec::new();

    if path.is_dir() {
        let mut dirs_to_scan = VecDeque::new();
        dirs_to_scan.push_back(path.clone());

        while let Some(scan_dir) = dirs_to_scan.pop_front() {
            if let Ok(mut read_dir) = fs::read_dir(scan_dir).await {
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
    } else {
        paths.push(path.clone())
    }

    Ok(paths)
}

#[cfg(test)]
mod tests {
    use crate::error::AssetbundleError;

    use super::*;
    use tempfile::{tempdir, tempdir_in};

    #[tokio::test]
    async fn test_scan_path() -> Result<(), AssetbundleError> {
        // create temporary directory to scan
        let temp_dir_1 = tempdir()?;

        // create nested directory
        let temp_dir_2 = tempdir_in(&temp_dir_1)?;

        // create temporary files
        let file_1 = temp_dir_1.path().join("file1.txt");
        let file_2 = temp_dir_1.path().join("file2.txt");
        let file_3 = temp_dir_2.path().join("file3.txt");

        tokio::fs::write(&file_1, b"content1").await?;
        tokio::fs::write(&file_2, b"content2").await?;
        tokio::fs::write(&file_3, b"content3").await?;

        // scan without recursive
        let dir_1_path = temp_dir_1.path().to_path_buf();
        let paths_not_recursive = scan_path(&dir_1_path, false).await?;

        // scan with recursive
        let paths_recursive = scan_path(&dir_1_path, true).await?;

        // scan path that leads to a file
        let paths_file = scan_path(&file_1.to_path_buf(), true).await?;

        // validate results
        assert_eq!(paths_not_recursive.len(), 2);
        assert!(paths_not_recursive.contains(&file_1));
        assert!(paths_not_recursive.contains(&file_2));

        assert_eq!(paths_recursive.len(), 3);
        assert!(paths_recursive.contains(&file_1));
        assert!(paths_recursive.contains(&file_2));
        assert!(paths_recursive.contains(&file_3));

        assert_eq!(paths_file.len(), 1);
        assert!(paths_file.contains(&file_1));

        Ok(())
    }
}

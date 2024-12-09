use serde::de::DeserializeOwned;
use serde_json::Value;
use std::{
    collections::VecDeque,
    path::{Path, PathBuf},
};
use tokio::{
    fs::{self, create_dir_all, File},
    io::AsyncWriteExt,
};

use crate::error::CommonError;

/// Provided a path, will return all files related to that path.
/// 1. If the path corresponds to an individual file, only that file's path will be returned.
/// 2. If it is a directory, all files within that directory will be returned (recursive if given).
pub async fn scan_path(path: &Path, recursive: bool) -> Result<Vec<PathBuf>, tokio::io::Error> {
    let mut paths = Vec::new();

    if path.is_dir() {
        let mut dirs_to_scan = VecDeque::new();
        dirs_to_scan.push_back(path.to_path_buf());

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
        paths.push(path.to_path_buf())
    }

    Ok(paths)
}

/// Writes bytes to the given out_path.
///
/// Any missing directories will be created.
/// If a file already exists at [`out_path`], it will be truncated with the new data.
pub async fn write_file(out_path: impl AsRef<Path>, data: &[u8]) -> Result<(), tokio::io::Error> {
    // write file
    if let Some(parent) = out_path.as_ref().parent() {
        create_dir_all(parent).await?;
    }
    let mut out_file = File::options()
        .write(true)
        .create(true)
        .truncate(true)
        .open(out_path)
        .await?;
    out_file.write_all(data).await?;
    Ok(())
}

/// Extracts the inner fields of a suitemaster file and writes them
/// to the provided out_path as .json files.
/// 
/// If pretty is true, the extracted fields will be JSON prettified.
pub async fn extract_suitemaster_file(file: Value, out_path: &Path, pretty: bool) -> Result<(), CommonError> {
    let obj = match file.as_object() {
        Some(obj) => Ok(obj),
        None => Err(CommonError::NotFound(
            "malformed suitemaster file: could not read value as an object".to_string(),
        )),
    }?;

    for (field_key, field_value) in obj.iter() {
        let extracted_out_path = out_path.join(format!("{}.json", field_key));
        let json_bytes = if pretty {
            serde_json::to_vec_pretty(&field_value)
        } else {
            serde_json::to_vec(&field_value)
        }?;
        write_file(extracted_out_path, &json_bytes).await?;
    }

    Ok(())
}

/// Deserializes a .json file into a serde_json Value.
///
/// If successful returns a tuple containing the file's stem and deserialized [`serde_json::Value`].
pub fn deserialize_file<D: DeserializeOwned>(path: &PathBuf) -> Result<D, CommonError> {
    let contents = std::fs::read_to_string(path)?;
    let deserialized = serde_json::from_str(&contents)?;
    Ok(deserialized)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{tempdir, tempdir_in};

    #[tokio::test]
    async fn test_scan_path() -> Result<(), CommonError> {
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

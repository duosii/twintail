use std::{io::SeekFrom, path::PathBuf};

use tokio::{
    fs::{create_dir_all, File},
    io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt, BufReader},
};

use crate::error::AssetbundleError;

const UNITY_ASSETBUNDLE_MAGIC: &[u8] = b"\x55\x6e\x69\x74\x79\x46";
const SEKAI_ASSETBUNDLE_MAGIC: &[u8] = b"\x10\x00\x00\x00";
const HEADER_SIZE: usize = 128;
const CHUNK_SIZE: usize = 65536;
const HEADER_BLOCK_SIZE: usize = 8;
const DECRYPT_SIZE: usize = 5;

#[derive(PartialEq)]
pub enum CryptOperation {
    Encrypt,
    Decrypt,
}

/// Flips specific bytes in the provided reader's header into the provided buffer.
///
/// Writes the rest of the file to the provided buffer.
async fn crypt(
    reader: &mut BufReader<File>,
    out_buf: &mut Vec<u8>,
) -> Result<(), AssetbundleError> {
    // flip header bytes
    let mut header_buf = [0u8; HEADER_SIZE];
    reader.read_exact(&mut header_buf).await?;
    for i in (0..HEADER_SIZE).step_by(HEADER_BLOCK_SIZE) {
        for j in 0..DECRYPT_SIZE {
            header_buf[i + j] = !header_buf[i + j];
        }
    }
    out_buf.write_all(&header_buf).await?;

    // write the rest of the file
    let mut chunk = vec![0; CHUNK_SIZE];
    loop {
        let bytes_read = reader.read(&mut chunk).await?;
        if bytes_read == 0 {
            break;
        }
        out_buf.write_all(&chunk[..bytes_read]).await?;
    }

    Ok(())
}

/// Decrypts an encrypted AssetBundle, returning the decrypted bytes.
///
/// Implementation credit: https://github.com/mos9527/sssekai/blob/main/sssekai/crypto/AssetBundle.py
pub async fn decrypt(reader: &mut BufReader<File>) -> Result<Vec<u8>, AssetbundleError> {
    // see if the file contains the magic
    let mut magic_buf = vec![0; SEKAI_ASSETBUNDLE_MAGIC.len()];
    reader.read_exact(&mut magic_buf).await?;
    if magic_buf != SEKAI_ASSETBUNDLE_MAGIC {
        return Err(AssetbundleError::NotEncrypted());
    }

    let mut out_buffer = Vec::new();
    crypt(reader, &mut out_buffer).await?;

    Ok(out_buffer)
}

/// Encrypts an AssetBundle, returning the encrypted bytes.
///
/// Implementation credit: https://github.com/mos9527/sssekai/blob/main/sssekai/crypto/AssetBundle.py
pub async fn encrypt(reader: &mut BufReader<File>) -> Result<Vec<u8>, AssetbundleError> {
    // check magic to ensure that it's a unity asset bundle.
    let mut magic_buf = vec![0; UNITY_ASSETBUNDLE_MAGIC.len()];
    reader.read_exact(&mut magic_buf).await?;
    reader.seek(SeekFrom::Start(0)).await?;
    if magic_buf != UNITY_ASSETBUNDLE_MAGIC {
        return Err(AssetbundleError::NotAssetbundle());
    }

    let mut out_buffer = Vec::new();
    out_buffer.write_all(SEKAI_ASSETBUNDLE_MAGIC).await?;
    crypt(reader, &mut out_buffer).await?;

    Ok(out_buffer)
}

/// Encrypts or decrypts a file at the input path into the output path.
///
/// Truncates and overwrites the file at out_path.
pub async fn crypt_file(
    in_path: &PathBuf,
    out_path: &PathBuf,
    operation: &CryptOperation,
) -> Result<(), AssetbundleError> {
    // decrypt
    let in_file = File::open(in_path).await?;
    let mut reader = BufReader::new(in_file);
    let crypted: Vec<u8> = if operation == &CryptOperation::Encrypt {
        encrypt(&mut reader).await?
    } else {
        decrypt(&mut reader).await?
    };

    // create parent folders if they do not exist
    if let Some(parent) = out_path.parent() {
        create_dir_all(parent).await?;
    }
    let mut out_file = File::options()
        .write(true)
        .create(true)
        .truncate(true)
        .open(out_path)
        .await?;
    out_file.write_all(&crypted).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio::fs::write;

    #[tokio::test]
    async fn test_encrypt_decrypt() -> Result<(), AssetbundleError> {
        let dir = tempdir()?;
        let input_path = dir.path().join("input.bundle");
        let encrypted_path = dir.path().join("encrypted.bundle");
        let decrypted_path = dir.path().join("decrypted.bundle");

        // Create a mock AssetBundle
        let mut mock_bundle = vec![];
        mock_bundle.extend(UNITY_ASSETBUNDLE_MAGIC);
        mock_bundle.extend((0..CHUNK_SIZE).into_iter().map(|_| 0x0));

        write(&input_path, mock_bundle).await?;

        // Encrypt
        crypt_file(&input_path, &encrypted_path, &CryptOperation::Encrypt).await?;

        // Decrypt
        crypt_file(&encrypted_path, &decrypted_path, &CryptOperation::Decrypt).await?;

        // Compare original and decrypted
        let original = tokio::fs::read(&input_path).await?;
        let decrypted = tokio::fs::read(&decrypted_path).await?;
        assert_eq!(original, decrypted);

        Ok(())
    }

    #[tokio::test]
    async fn test_decrypt_not_encrypted() -> Result<(), AssetbundleError> {
        let dir = tempdir()?;
        let input_path = dir.path().join("not_encrypted.bundle");
        let output_path = dir.path().join("output.bundle");

        // Create a mock unencrypted file
        let mock_file = vec![0x00, 0x01, 0x02, 0x03];
        write(&input_path, &mock_file).await?;

        // Try to decrypt
        let result = crypt_file(&input_path, &output_path, &CryptOperation::Decrypt).await;
        assert!(matches!(result, Err(AssetbundleError::NotEncrypted())));

        Ok(())
    }

    #[tokio::test]
    async fn test_encrypt_not_assetbundle() -> Result<(), AssetbundleError> {
        let dir = tempdir()?;
        let input_path = dir.path().join("not_assetbundle.file");
        let output_path = dir.path().join("output.bundle");

        // Create a mock file that's not an AssetBundle
        let mock_file = vec![0x00, 0x01, 0x02, 0x03, 0x05, 0x12];
        write(&input_path, &mock_file).await?;

        // Try to encrypt
        let result = crypt_file(&input_path, &output_path, &CryptOperation::Encrypt).await;
        assert!(matches!(result, Err(AssetbundleError::NotAssetbundle())));

        Ok(())
    }
}

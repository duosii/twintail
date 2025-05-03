use std::{
    io::SeekFrom,
    path::{Path, PathBuf},
};

use futures::{StreamExt, stream};
use tokio::{
    fs::File,
    io::{AsyncRead, AsyncReadExt, AsyncSeek, AsyncSeekExt, AsyncWrite, AsyncWriteExt, BufReader},
    time::Instant,
};
use twintail_common::{color, models::enums::CryptOperation, utils::progress::ProgressBar};

use crate::{
    Error,
    fs::{scan_path, write_file},
};

const UNITY_ASSETBUNDLE_MAGIC: &[u8] = b"\x55\x6e\x69\x74\x79\x46";
const SEKAI_ASSETBUNDLE_MAGIC: &[u8] = b"\x10\x00\x00\x00";
const HEADER_SIZE: usize = 128;
const CHUNK_SIZE: usize = 65536;
const HEADER_BLOCK_SIZE: usize = 8;
const DECRYPT_SIZE: usize = 5;

pub struct AbCryptStrings {
    pub process: &'static str,
    pub processed: &'static str,
}

pub struct AbCryptArgs {
    pub recursive: bool,
    pub quiet: bool,
    pub concurrent: usize,
    pub operation: CryptOperation,
    pub strings: AbCryptStrings,
}

/// Flips specific bytes in the provided reader's header into the provided buffer.
///
/// Writes the rest of the file to the provided buffer.
async fn crypt(reader: &mut (impl AsyncRead + Unpin), out_buf: &mut Vec<u8>) -> Result<(), Error> {
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

/// Decrypts an encrypted AssetBundle in-place.
///
/// Modifies the input buffer directly.
pub async fn decrypt_in_place(buffer: &mut Vec<u8>) -> Result<(), Error> {
    // Check if the file contains the magic
    if buffer.len() < SEKAI_ASSETBUNDLE_MAGIC.len()
        || &buffer[..SEKAI_ASSETBUNDLE_MAGIC.len()] != SEKAI_ASSETBUNDLE_MAGIC
    {
        return Err(Error::NotEncrypted);
    }

    // Remove the magic bytes
    buffer.drain(..SEKAI_ASSETBUNDLE_MAGIC.len());

    // Flip header bytes in-place
    for i in (0..HEADER_SIZE.min(buffer.len())).step_by(HEADER_BLOCK_SIZE) {
        for j in 0..DECRYPT_SIZE.min(HEADER_BLOCK_SIZE) {
            if i + j < buffer.len() {
                buffer[i + j] = !buffer[i + j];
            }
        }
    }

    Ok(())
}

/// Decrypts an encrypted AssetBundle, returning the decrypted bytes.
///
/// Implementation credit: https://github.com/mos9527/sssekai/blob/main/sssekai/crypto/AssetBundle.py
pub async fn decrypt(
    reader: &mut (impl AsyncWrite + AsyncSeek + AsyncRead + Unpin),
) -> Result<Vec<u8>, Error> {
    // see if the file contains the magic
    let mut magic_buf = vec![0; SEKAI_ASSETBUNDLE_MAGIC.len()];
    reader.read_exact(&mut magic_buf).await?;
    if magic_buf != SEKAI_ASSETBUNDLE_MAGIC {
        return Err(Error::NotEncrypted);
    }

    let mut out_buffer = Vec::new();
    crypt(reader, &mut out_buffer).await?;

    Ok(out_buffer)
}

/// Encrypts an AssetBundle, returning the encrypted bytes.
///
/// Implementation credit: https://github.com/mos9527/sssekai/blob/main/sssekai/crypto/AssetBundle.py
pub async fn encrypt(
    reader: &mut (impl AsyncWrite + AsyncSeek + AsyncRead + Unpin),
) -> Result<Vec<u8>, Error> {
    // check magic to ensure that it's a unity asset bundle.
    let mut magic_buf = vec![0; UNITY_ASSETBUNDLE_MAGIC.len()];
    reader.read_exact(&mut magic_buf).await?;
    reader.seek(SeekFrom::Start(0)).await?;
    if magic_buf != UNITY_ASSETBUNDLE_MAGIC {
        return Err(Error::NotAssetbundle);
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
    out_path: &Path,
    operation: &CryptOperation,
) -> Result<(), Error> {
    // decrypt
    let in_file = File::open(in_path).await?;
    let mut reader = BufReader::new(in_file);
    let crypted: Vec<u8> = if operation == &CryptOperation::Encrypt {
        encrypt(&mut reader).await?
    } else {
        decrypt(&mut reader).await?
    };

    // create parent folders if they do not exist
    write_file(out_path, &crypted).await?;

    Ok(())
}

/// Encrypts or decrypts a an entire path.
///
/// If out_path is not provided, files will be encrypted/decrypted in-place.
/// Truncates and overwrites the file(s) at out_path.
///
/// Returns the number of files that were encrypted or decrypted.
pub async fn crypt_path(
    in_path: impl AsRef<Path>,
    out_path: Option<impl AsRef<Path>>,
    crypt_args: &AbCryptArgs,
) -> Result<usize, Error> {
    let in_path = in_path.as_ref();
    let out_path = out_path.as_ref().map(|p| p.as_ref()).unwrap_or(in_path);
    let in_place = in_path == out_path;
    let show_progress = !crypt_args.quiet;

    // get the paths we need to encrypt
    let scan_progress_bar = if show_progress {
        println!(
            "{}[1/2] {}Scanning files...",
            color::TEXT_VARIANT.render_fg(),
            color::TEXT.render_fg()
        );
        Some(ProgressBar::spinner())
    } else {
        None
    };

    let in_paths = scan_path(in_path, crypt_args.recursive).await?;

    if let Some(scan_progress) = scan_progress_bar {
        scan_progress.finish_and_clear();
    }

    // start processing these files
    let crypt_start = Instant::now();
    if show_progress {
        println!(
            "{}[2/2] {}{} files...",
            color::TEXT_VARIANT.render_fg(),
            color::TEXT.render_fg(),
            crypt_args.strings.process,
        );
    }

    // compute paths
    let in_out_paths: Vec<(PathBuf, PathBuf)> = in_paths
        .into_iter()
        .map(|path| {
            if in_place {
                (path.clone(), path)
            } else {
                let relative = path.strip_prefix(in_path).ok().unwrap_or(&path);
                let out = out_path.join(relative);
                (path, out)
            }
        })
        .collect();

    // asynchronously encrypt the files
    let total_path_count = in_out_paths.len() as u64;
    let progress_bar = ProgressBar::progress(total_path_count);

    let decrypt_result: Vec<Result<(), Error>> = stream::iter(&in_out_paths)
        .map(|paths| async {
            let result = crypt_file(&paths.0, &paths.1, &crypt_args.operation).await;
            if show_progress {
                progress_bar.inc(1);
            }
            result
        })
        .buffer_unordered(crypt_args.concurrent)
        .collect()
        .await;
    let success_count = decrypt_result
        .iter()
        .filter(|&result| result.is_ok())
        .count();

    // stop progress bar & print the sucess message
    progress_bar.finish_and_clear();
    println!(
        "{}Successfully {} {} / {} files in {:?}.{}",
        color::SUCCESS.render_fg(),
        crypt_args.strings.processed,
        success_count,
        total_path_count,
        Instant::now().duration_since(crypt_start),
        color::TEXT.render_fg(),
    );

    Ok(success_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio::fs::write;

    #[tokio::test]
    async fn test_encrypt_decrypt() -> Result<(), Error> {
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
    async fn test_decrypt_in_place() -> Result<(), Error> {
        // Create a mock encrypted AssetBundle
        let mut mock_bundle = vec![];
        mock_bundle.extend(SEKAI_ASSETBUNDLE_MAGIC);
        mock_bundle.extend((0..CHUNK_SIZE).into_iter().map(|_| 0x0));

        // decrypt
        decrypt_in_place(&mut mock_bundle).await?;
        assert_eq!(mock_bundle.len(), CHUNK_SIZE);

        Ok(())
    }

    #[tokio::test]
    async fn test_decrypt_not_encrypted() -> Result<(), Error> {
        let dir = tempdir()?;
        let input_path = dir.path().join("not_encrypted.bundle");
        let output_path = dir.path().join("output.bundle");

        // Create a mock unencrypted file
        let mock_file = vec![0x00, 0x01, 0x02, 0x03];
        write(&input_path, &mock_file).await?;

        // Try to decrypt
        let result = crypt_file(&input_path, &output_path, &CryptOperation::Decrypt).await;
        assert!(matches!(result, Err(Error::NotEncrypted)));

        Ok(())
    }

    #[tokio::test]
    async fn test_encrypt_not_assetbundle() -> Result<(), Error> {
        let dir = tempdir()?;
        let input_path = dir.path().join("not_assetbundle.file");
        let output_path = dir.path().join("output.bundle");

        // Create a mock file that's not an AssetBundle
        let mock_file = vec![0x00, 0x01, 0x02, 0x03, 0x05, 0x12];
        write(&input_path, &mock_file).await?;

        // Try to encrypt
        let result = crypt_file(&input_path, &output_path, &CryptOperation::Encrypt).await;
        assert!(matches!(result, Err(Error::NotAssetbundle)));

        Ok(())
    }
}

use std::io::{SeekFrom, Write};

use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncSeekExt, BufReader},
};

use crate::error::AssetbundleError;

const UNITY_ASSETBUNDLE_MAGIC: &[u8] = b"\x55\x6e\x69\x74\x79\x46";
const SEKAI_ASSETBUNDLE_MAGIC: &[u8] = b"\x10\x00\x00\x00";
const HEADER_SIZE: usize = 128;
const CHUNK_SIZE: usize = 65536;
const HEADER_BLOCK_SIZE: usize = 8;
const DECRYPT_SIZE: usize = 5;

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
            header_buf[i + j] = !header_buf[i + j] & 0xFF;
        }
    }
    out_buf.write(&header_buf)?;

    // write the rest of the file
    let mut chunk = vec![0; CHUNK_SIZE];
    loop {
        let bytes_read = reader.read(&mut chunk).await?;
        if bytes_read == 0 {
            break;
        }
        out_buf.write(&chunk[..bytes_read])?;
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
    if &magic_buf != SEKAI_ASSETBUNDLE_MAGIC {
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
    if &magic_buf != UNITY_ASSETBUNDLE_MAGIC {
        return Err(AssetbundleError::NotAssetbundle());
    }

    let mut out_buffer = Vec::new();
    out_buffer.write(&SEKAI_ASSETBUNDLE_MAGIC)?;
    crypt(reader, &mut out_buffer).await?;

    Ok(out_buffer)
}

use aes::cipher::{
    block_padding::{Pkcs7, UnpadError},
    BlockDecryptMut, BlockEncryptMut, KeyIvInit,
};

type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;
type Aes128CbcEnc = cbc::Encryptor<aes::Aes128>;

const KEY: &[u8] = b"g2fcC0ZczN9MTJ61";
const IV: &[u8] = b"msx3IV0i9XE5uYZ1";

/// Decrypt bytes encrypted with Aes128 using a predefined key & iv.
pub fn decrypt(block: &[u8]) -> Result<Vec<u8>, UnpadError> {
    let cipher = Aes128CbcDec::new(KEY.into(), IV.into());

    cipher.decrypt_padded_vec_mut::<Pkcs7>(block)
}

/// Encrypt bytes using a predefined key & iv.
pub fn encrypt(block: &[u8]) -> Vec<u8> {
    let cipher = Aes128CbcEnc::new(KEY.into(), IV.into());

    cipher.encrypt_padded_vec_mut::<Pkcs7>(block)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes_encrypt_decrypt() {
        let data = b"39393939393".to_vec();

        // encrypt the plaintext
        let encrypted = encrypt(&data.clone());

        // decrypt the ciphertext
        let decrypted = decrypt(&encrypted).expect("Error when decrypting data.");
        assert_eq!(decrypted, data);
    }
}

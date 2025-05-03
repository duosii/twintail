use aes::cipher::{
    BlockDecryptMut, BlockEncryptMut, KeyIvInit,
    block_padding::{Pkcs7, UnpadError},
    generic_array::GenericArray,
};

use crate::error::CryptoError;

type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;
type Aes128CbcEnc = cbc::Encryptor<aes::Aes128>;

#[derive(Clone)]
pub struct AesConfig {
    pub key: [u8; 16],
    pub iv: [u8; 16],
}

impl AesConfig {
    /// Generates an AesConfig using hexadecimal key & IV values.
    ///
    /// The hexadecimal values should be strings.
    ///
    /// This function may error if parsing the hexadecimal strings fails.
    pub fn from_hex(hex_key: &str, hex_iv: &str) -> Result<Self, CryptoError> {
        Ok(Self {
            key: decode_hex(hex_key)?
                .try_into()
                .map_err(|_| CryptoError::InvalidKeyLength())?,
            iv: decode_hex(hex_iv)?
                .try_into()
                .map_err(|_| CryptoError::InvalidKeyLength())?,
        })
    }
}

/// Decrypt bytes encrypted with Aes128 using a predefined key & iv.
pub fn decrypt(block: &[u8], config: &AesConfig) -> Result<Vec<u8>, UnpadError> {
    let key = GenericArray::from_slice(&config.key);
    let iv = GenericArray::from_slice(&config.iv);
    let cipher = Aes128CbcDec::new(key, iv);

    cipher.decrypt_padded_vec_mut::<Pkcs7>(block)
}

/// Encrypt bytes using a predefined key & iv.
pub fn encrypt(block: &[u8], config: &AesConfig) -> Vec<u8> {
    let key = GenericArray::from_slice(&config.key);
    let iv = GenericArray::from_slice(&config.iv);
    let cipher = Aes128CbcEnc::new(key, iv);

    cipher.encrypt_padded_vec_mut::<Pkcs7>(block)
}

/// Parses a hex string into a Vec of bytes.
///
/// Implementation credit: https://stackoverflow.com/a/52992629
pub fn decode_hex(hex_str: &str) -> Result<Vec<u8>, std::num::ParseIntError> {
    (0..hex_str.len())
        .step_by(2)
        .map(|num| u8::from_str_radix(&hex_str[num..num + 2], 16))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::enums::Server;

    #[test]
    fn test_aes_encrypt_decrypt() {
        let data = b"39393939393".to_vec();

        // encrypt the plaintext
        let encrypted = encrypt(&data.clone(), &Server::Japan.get_aes_config());

        // decrypt the ciphertext
        let decrypted = decrypt(&encrypted, &Server::Japan.get_aes_config())
            .expect("Error when decrypting data.");
        assert_eq!(decrypted, data);
    }

    #[test]
    fn test_utils_decode_hex() {
        let decoded = decode_hex("6732666343305a637a4e394d544a3631").unwrap();
        assert_eq!(
            decoded,
            vec![
                103, 50, 102, 99, 67, 48, 90, 99, 122, 78, 57, 77, 84, 74, 54, 49
            ]
        )
    }
}

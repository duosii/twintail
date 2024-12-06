use aes::cipher::{
    block_padding::{Pkcs7, UnpadError}, generic_array::GenericArray, BlockDecryptMut, BlockEncryptMut, KeyIvInit
};

use crate::config::AesConfig;

type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;
type Aes128CbcEnc = cbc::Encryptor<aes::Aes128>;

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

#[cfg(test)]
mod tests {
    use crate::models::enums::Server;

    use super::*;

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
}

use aes::cipher::{
    block_padding::{Pkcs7, UnpadError},
    BlockDecryptMut, BlockEncryptMut, KeyIvInit,
};

use crate::models::enums::Server;

type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;
type Aes128CbcEnc = cbc::Encryptor<aes::Aes128>;

/// Decrypt bytes encrypted with Aes128 using a predefined key & iv.
pub fn decrypt(block: &[u8], server: &Server) -> Result<Vec<u8>, UnpadError> {
    let config = server.get_aes_config();
    let cipher = Aes128CbcDec::new(config.key.into(), config.iv.into());

    cipher.decrypt_padded_vec_mut::<Pkcs7>(block)
}

// /// Decrypt bytes in-place encrypted with Aes128 using a predefined key & iv.
// pub fn decrypt_in_place(block: &mut [u8], server: &Server) -> Result<(), UnpadError> {
//     let config = server.get_aes_config();
//     let cipher = Aes128CbcDec::new(config.key.into(), config.iv.into());

//     cipher.decrypt_padded_mut::<Pkcs7>(block)?;
//     Ok(())
// }

/// Encrypt bytes using a predefined key & iv.
pub fn encrypt(block: &[u8], server: &Server) -> Vec<u8> {
    let config = server.get_aes_config();
    let cipher = Aes128CbcEnc::new(config.key.into(), config.iv.into());

    cipher.encrypt_padded_vec_mut::<Pkcs7>(block)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes_encrypt_decrypt() {
        let data = b"39393939393".to_vec();

        // encrypt the plaintext
        let encrypted = encrypt(&data.clone(), &Server::Japan);

        // decrypt the ciphertext
        let decrypted = decrypt(&encrypted, &Server::Japan).expect("Error when decrypting data.");
        assert_eq!(decrypted, data);
    }
}

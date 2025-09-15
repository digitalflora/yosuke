use std::io;

use aes_gcm::{
    Aes256Gcm, KeyInit, Nonce,
    aead::{Aead, consts::U32, generic_array::GenericArray},
};
use rand::TryRngCore;

#[derive(Clone)] // Add Clone trait for the struct
pub struct Encryption {
    pub cipher: Aes256Gcm,
    pub _key: GenericArray<u8, U32>,
}

impl Encryption {
    pub fn new(key: &GenericArray<u8, U32>) -> Self {
        // let key: &GenericArray<u8, U32> = Key::<Aes256Gcm>::from_slice(key_b);
        let cipher = Aes256Gcm::new(key);
        Self { cipher, _key: *key }
    }

    pub fn encrypt(&self, data: &[u8]) -> Result<([u8; 12], Vec<u8>), std::io::Error> {
        let mut nonce_b = [0u8; 12];
        rand::rngs::OsRng
            .try_fill_bytes(&mut nonce_b)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, "Failed try_fill_bytes()"))?;
        let nonce = Nonce::from_slice(&nonce_b);
        let encrypted = self
            .cipher
            .encrypt(nonce, data)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, "Failed encrypt()"))?;
        Ok((nonce_b, encrypted))
    }

    pub fn decrypt(&self, nonce_b: &[u8; 12], encrypted: &[u8]) -> Result<Vec<u8>, std::io::Error> {
        /*println!("[*] Decrypting...");
        println!("[*] Key (first 8 bytes): {}", hex::encode(&self._key[..8]));
        println!("[*] Nonce: {}", hex::encode(&nonce_b));
        println!("[*] Encrypted data length: {}", encrypted.len());
        println!("[*] Encrypted data (first 32 bytes): {}", hex::encode(&encrypted[..encrypted.len().min(32)]));*/

        let nonce = Nonce::from_slice(nonce_b);

        match self.cipher.decrypt(&nonce, encrypted) {
            Ok(decrypted) => {
                println!("[✓] Decrypted payload");
                Ok(decrypted)
            }
            Err(e) => {
                println!("[!] Decryption failed: {:?}", e);
                Err(io::Error::new(io::ErrorKind::Other, "Failed decrypt()"))
            }
        }
    }
}

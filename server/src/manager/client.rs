use crate::{net, types::WhitelistedClient};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};
use tokio_util::compat::{
    FuturesAsyncWriteCompatExt, TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt,
};

pub enum ClientCommand {
    Write(Vec<u8>),
}
pub enum ClientResponse {
    Read(String, Vec<u8>),
}
pub struct ClientMouthpiece {
    pub to_client: UnboundedSender<ClientCommand>,
    pub from_client: UnboundedReceiver<ClientResponse>,
}
pub struct Client {
    pub whitelisted: WhitelistedClient, // has the key for encryption in it!,
    pub sender: UnboundedSender<ClientCommand>,
    pub handle: JoinHandle<()>, // tokio::task per client
}
pub struct ClientPassthroughMouthpiece {
    pub to_manager: UnboundedSender<ClientResponse>,
    pub from_manager: UnboundedReceiver<ClientCommand>,
}

pub async fn task(
    mutex: String,
    mut stream: TcpStream,
    mut mouthpiece: ClientPassthroughMouthpiece,
) {
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
                        let _ = mouthpiece.to_manager.send(ClientResponse::Read(mutex.clone(), buf));
                    },
                    Err(_e) => {}
                }
            }
            // from TcpStream, read in payload size and then read the actual payload into a buffer based on the size
            manager_read = mouthpiece.from_manager.recv() => {
                if let Some(command) = manager_read {
                    match command {
                        ClientCommand::Write(buf) => {
                            let _ = shared::net::write(&mut write, &buf).await;
                        }
                    }
                }
            }
        }
    }
}

use crate::{net, types::WhitelistedClient};

use tokio::{
    io::AsyncReadExt,
    net::TcpStream,
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

pub enum ClientCommand {
    Write(Vec<u8>),
}
pub enum ClientResponse {
    Read(Vec<u8>),
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

pub async fn task(mut stream: TcpStream, mut mouthpiece: ClientPassthroughMouthpiece) {
    println!("[v] client task! read loop should go here");

    let (read, write) = stream.into_split();
    let mut read = read.compat();
    let mut write = write.compat_write();

    loop {
        tokio::select! {
            stream_read = shared::net::read(&mut read) => {
                match stream_read {
                    Ok(buffer) => {
                        println!("[*] received data from a client");
                    },
                    Err(_e) => {}
                }
            }
            // from TcpStream, read in payload size and then read the actual payload into a buffer based on the size
            manager_read = mouthpiece.from_manager.recv() => {}
        }
    }
}

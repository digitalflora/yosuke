use crate::types::WhitelistedClient;
use tokio::{
    io::AsyncReadExt,
    net::TcpStream,
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};

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

    let mut payload_size_buf = [0u8; std::mem::size_of::<usize>()];

    loop {
        tokio::select! {
            stream_read = stream.read_exact(&mut payload_size_buf) => {},
            manager_read = mouthpiece.from_manager.recv() => {},
        }
    }
}

use std::collections::HashMap;

use tokio::{
    net::TcpStream,
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender, unbounded_channel},
    task::JoinHandle,
};

use crate::{
    manager::client::{
        Client, ClientCommand, ClientMouthpiece, ClientPassthroughMouthpiece, ClientResponse,
    },
    types::WhitelistedClient,
};

pub enum UiManagerCommand {}
pub enum UiManagerResponse {}
pub enum ServerManagerMessage {
    ClientConnect(WhitelistedClient, TcpStream),
}
// pub enum ServerManagerResponse {}

pub struct SharedClientMouthpiece {
    pub from_client: UnboundedReceiver<ClientResponse>,
    pub to_manager: UnboundedSender<ClientResponse>, // clonable
}

pub struct ClientManagerMouthpiece {
    pub from_ui: UnboundedReceiver<UiManagerCommand>,
    pub to_ui: UnboundedSender<UiManagerResponse>,
    pub from_server: UnboundedReceiver<ServerManagerMessage>,
    pub client: SharedClientMouthpiece,
}

//////////////
pub mod client;
//////////////
// ClientManager
pub struct ClientManager {
    pub mouthpiece: ClientManagerMouthpiece,
    clients: HashMap<String, Client>, // Mutex, Client
}
impl ClientManager {
    pub fn new(mouthpiece: ClientManagerMouthpiece) -> Self {
        Self {
            mouthpiece: mouthpiece,
            clients: HashMap::new(),
        }
    }
    pub async fn run(mut self) {
        // implement
        println!("[*] manager spawned");

        loop {
            tokio::select! {
                Some(client_command) = self.mouthpiece.client.from_client.recv() => {
                    match client_command {
                        ClientResponse::Read(mutex, buf) => {
                            let response = String::from_utf8_lossy(&buf);
                            println!("[*][{}] sent data", mutex);
                            println!("{}", response);
                        }
                    }
                }
                Some(ui_command) = self.mouthpiece.from_ui.recv() => {
                    println!("[*] received command from ui");
                    // self.mouthpiece.to_ui.send(UiManagerResponse::...);
                },
                Some(server_command) = self.mouthpiece.from_server.recv() => {
                    println!("[*][manager] received command from server");
                    match server_command {
                        ServerManagerMessage::ClientConnect(whitelisted, stream) => {
                            {
                                let (to_client, from_manager) = unbounded_channel::<ClientCommand>();
                                let mutex = whitelisted.mutex.clone();
                                let task_mutex = mutex.clone();
                                let to_manager = self.mouthpiece.client.to_manager.clone();
                                let client = Client {
                                    whitelisted: whitelisted,
                                    sender: to_client, // no mouthpiece needed because we clone to_manager,
                                    handle: tokio::spawn(async move {
                                        client::task(task_mutex, stream, ClientPassthroughMouthpiece {
                                            to_manager: to_manager,
                                            from_manager: from_manager
                                        }).await;
                                    })
                                };
                                self.clients.insert(mutex, client);
                            }
                        }
                    }
                }
                else => break, // finished running
            }
        }
    }
}

/*
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
*/

use std::collections::HashMap;

use aes_gcm::aead::generic_array::GenericArray;
use shared::{
    commands::{BaseCommand, BaseResponse},
    crypto::Encryption,
};
use tokio::sync::mpsc::unbounded_channel;

use crate::{
    manager::client::{Client, ClientCommand, ClientPassthroughMouthpiece, ClientResponse},
    manager::types::*,
};

pub mod types;

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
                            let (response, _size): (BaseResponse, usize) = bincode::decode_from_slice(&buf, bincode::config::standard()).unwrap();
                            println!("[*][{}] sent data", mutex);
                            // pass to UI?
                            let _ = self.mouthpiece.to_ui.send(UiManagerResponse::GetResponse(mutex, response.response));
                        },
                        ClientResponse::Disconnect(mutex) => {
                            println!("[*][{}] disconnected", mutex);
                            self.clients.remove(&mutex);
                            let _ = self.mouthpiece.to_ui.send(UiManagerResponse::Remove(mutex));
                        }
                    }
                }
                Some(ui_command) = self.mouthpiece.from_ui.recv() => {
                    println!("[*] received command from ui");
                    match ui_command {
                        UiManagerCommand::SendCommand(mutex, command) => {
                            if let Some(client) = self.clients.get_mut(&mutex) {
                                if let Err(_e) = client.sender.send(ClientCommand::Write(
                                    bincode::encode_to_vec(BaseCommand {
                                        id: client.counter,
                                        command: command
                                    }, bincode::config::standard()).unwrap()
                                )) {
                                    println!("[*][manager] failed to send command to client");
                                } else { client.counter += 1; };
                            }
                        },
                        UiManagerCommand::Disconnect(mutex) => {
                            if let Some(client) = self.clients.get_mut(&mutex) {
                                client.handle.abort();
                                self.clients.remove(&mutex);
                                let _ = self.mouthpiece.to_ui.send(UiManagerResponse::Remove(mutex));
                            }
                        }
                    }
                    // self.mouthpiece.to_ui.send(UiManagerResponse::...);
                },
                Some(server_command) = self.mouthpiece.from_server.recv() => {
                    println!("[*][manager] received command from server");
                    match server_command {
                        ServerManagerMessage::ClearClients => {
                            println!("[*] clearing clients");
                            for (_, client) in self.clients.iter() {
                                println!("[*] aborting task {}", client.mutex);
                                client.handle.abort();
                            }
                            self.clients.clear();
                            let _ = self.mouthpiece.to_ui.send(UiManagerResponse::RemoveAll);
                        },
                        ServerManagerMessage::ClientDisconnect(mutex) => {
                            if let Some(client) = self.clients.get(&mutex) {
                                println!("[*] aborting task {}", mutex);
                                client.handle.abort();
                                self.clients.remove(&mutex);
                                let _ = self.mouthpiece.to_ui.send(UiManagerResponse::Remove(mutex));
                            }
                        }
                        ServerManagerMessage::ClientConnect(whitelisted, stream) => {
                            {
                                let (to_client, from_manager) = unbounded_channel::<ClientCommand>();
                                let mutex = whitelisted.mutex.clone();
                                let task_mutex = mutex.clone();
                                let to_manager = self.mouthpiece.client.to_manager.clone();
                                let encryption = Encryption::new(GenericArray::from_slice(&whitelisted.key));
                                let client = Client {
                                    mutex: mutex.clone(),
                                    counter: 1, // 0 is reserved for computer info
                                    sender: to_client, // no mouthpiece needed because we clone to_manager,
                                    handle: tokio::spawn(async move {
                                        if let Err(err) = client::task(task_mutex, encryption, stream, ClientPassthroughMouthpiece {
                                            to_manager: to_manager,
                                            from_manager: from_manager
                                        }).await {
                                            println!("[x][manager] {:?}", err);
                                        };
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

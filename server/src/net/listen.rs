use std::{collections::HashMap, sync::Arc};

use crate::{
    SettingsPointer,
    net::handler,
    types::{mouthpieces::ServerMouthpiece, *},
};
use tokio::{io::AsyncWriteExt, net::TcpListener, sync::Mutex};

pub async fn main(
    mut settings: SettingsPointer,
    mut mouthpiece: ServerMouthpiece,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("[*] server thread spawned");
    let mut listener: Option<TcpListener> = None; // init with no listener
    let mut streams: StreamCollectionPointer = Arc::new(Mutex::new(HashMap::new()));

    loop {
        tokio::select! {
            msg = mouthpiece.from_ui.recv() => {
                match msg {
                    Some(UiMessage::Listen) => {
                        match TcpListener::bind("0.0.0.0:5317").await {
                            Ok(l) => {
                                println!("[*][listen()] listening on port 5317");
                                listener = Some(l);
                                mouthpiece.to_ui.send(ServerMessage::Listening)?;
                            }
                            Err(e) => eprintln!("[x][listen()] {}", e),
                        }
                    }
                    Some(UiMessage::Shutdown) => {
                        listener = None;
                        streams.lock().await.clear();
                        mouthpiece.to_ui.send(ServerMessage::Stopped)?;
                    }, None => {break Ok(())}

                    // send data to a client
                    Some(UiMessage::Send(mutex, data)) => {

                        let client = {
                            let streams_guard = streams.lock().await;
                            streams_guard.get(mutex.as_str()).cloned()
                        };


                            if let Some(client) = client {
                                let mut stream_guard = client.stream.lock().await;
                                println!("[*] send payload size");
                                stream_guard.write_all(&data.len().to_le_bytes()).await;
                                println!("[*] send payload");
                                stream_guard.write_all(&data).await;
                                println!("[v] wrote to {}", mutex);
                            }

                        println!("[*][listen()] send to be implemented");
                    }

                }
            }

            response = async {
                match &listener {
                    Some(l) => l.accept().await,
                    None => core::future::pending().await,
                }
            } => {
                if let Ok((stream, addr)) = response {
                    let stream_pointer = Arc::new(Mutex::new(stream));
                    println!("[*][listen()] new connection from {}", addr);
                    // let _ = mouthpiece.to_ui.send(ServerMessage::NewConnection(addr.to_string()));
                    let settings_clone = settings.clone();
                    let to_ui_clone = mouthpiece.to_ui.clone();
                    let streams_clone = streams.clone();
                    tokio::task::spawn(async move {

                        let stream_pointer_clone = stream_pointer.clone();

                        if let Ok(client) = handler::handshake(stream_pointer, settings_clone, &to_ui_clone).await {
                            let client_clone = client.clone();
                            streams_clone.lock().await.insert(client.mutex, StreamPointerContainer {
                                stream: stream_pointer_clone.clone(),
                                client: client_clone.clone()
                            });
                            let _ = to_ui_clone.send(ServerMessage::NewConnection(client_clone.clone()));
                            let _ = handler::wait(stream_pointer_clone).await; // loop blocks the code below
                            println!("[*] {} disconnected", client_clone.mutex);
                            streams_clone.lock().await.remove(client_clone.mutex.as_str());
                        }
                    });
                }
            }
        }
    }
}

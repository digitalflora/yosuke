use crate::{
    SettingsPointer,
    manager::ServerManagerMessage,
    net::handler,
    types::{mouthpieces::ServerMouthpiece, *},
};
use tokio::net::TcpListener;

pub async fn main(
    settings: SettingsPointer,
    mut mouthpiece: ServerMouthpiece,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("[*] server spawned");
    let mut listener: Option<TcpListener> = None; // init with no listener

    loop {
        tokio::select! {
            msg = mouthpiece.from_ui.recv() => {
                println!("[*] server got a message from the ui");
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
                        println!("[*][listen()] stop listening...");
                        mouthpiece.to_manager.send(ServerManagerMessage::ClearClients)?;
                        listener = None;
                        mouthpiece.to_ui.send(ServerMessage::Stopped)?;
                    }, None => {break Ok(())}
                }
            }

            response = async {
                match &listener {
                    Some(l) => l.accept().await,
                    None => core::future::pending().await,
                }
            } => {
                if let Ok((mut stream, addr)) = response {
                    println!("[*][listen()] new connection from {}", addr);
                    if let Ok(client) = handler::handshake(&mut stream, &settings).await {
                        let _ = mouthpiece.to_manager.send(ServerManagerMessage::ClientConnect(
                            client.clone(), // copy the whitelist info from the whitelist to the manager
                            stream          // move stream to the manager!! nice
                        ));
                    };

                    // let _ = mouthpiece.to_ui.send(ServerMessage::NewConnection(addr.to_string()));
                    // handler::wait(stream_pointer_clone).await;
                }
            }
        }
    }
}

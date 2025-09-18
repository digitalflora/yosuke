use crate::{
    manager::types::{ProcessedResponse, UiManagerResponse},
    types::*,
    ui::{client::ClientView, view::View},
};
use egui::Context;
use shared::commands::CaptureType;

// updates coming in to/from the Server
pub fn server(view: &mut View, _ctx: &Context) {
    while let Ok(msg) = view.mouthpiece.from_server.try_recv() {
        match msg {
            ServerMessage::Listening => {
                view.state.listening = true;
            }
            ServerMessage::Stopped => {
                view.state.listening = false;
            }

            ServerMessage::NewConnection(client) => {
                println!(
                    "[*][updates()] whitelisted mutex {} has connected!",
                    client.mutex
                );
            }
            ServerMessage::Receive(_mutex, _data) => {
                println!("[*][updates()] implement receive");
            }
        }
    }
}

// updates coming in to/from the Manager
pub fn manager(view: &mut View, _ctx: &Context) {
    while let Ok(msg) = view.mouthpiece.from_manager.try_recv() {
        match msg {
            UiManagerResponse::GetResponse(mutex, response) => {
                println!("[*][updates] sup");

                match response {
                    ProcessedResponse::Success => {
                        println!("[v] yay shit just works");
                    }
                    ProcessedResponse::ComputerInfo(info, socket) => {
                        if view.state.clients.contains_key(&mutex) {
                            // we already have info on this client
                            println!("[*] already got you");
                        } else {
                            println!("[*] new client to show on screen");
                            view.state.clients.insert(
                                mutex.clone(),
                                ClientView::new(
                                    socket.to_string(),
                                    mutex,
                                    info.elevated, // grab this because it can change
                                    info,
                                    view.mouthpiece.to_manager.clone(),
                                ),
                            );
                            println!("{}", view.state.clients.len());
                        }
                    }
                    ProcessedResponse::CapturePacket(capture_type, image) => {
                        println!("[*] got img");
                        if let Some(client) = view.state.clients.get_mut(&mutex) {
                            match capture_type {
                                CaptureType::Screen => {
                                    client.state.captures.screen.data = Some(image);
                                    client.state.textures.screen = None;
                                }
                                _ => { /* dont care about audio */ }
                            }
                        }
                    }
                    ProcessedResponse::Error(err) => {
                        println!("[x] oopsie: {}", err);
                    }
                }
            }
            UiManagerResponse::Remove(mutex) => {
                view.state.clients.remove(&mutex);
            }
            UiManagerResponse::RemoveAll => {
                view.state.clients.clear();
            }
        }
    }
}

use crate::{
    manager::types::UiManagerResponse,
    types::*,
    ui::{client::ClientView, view::View},
};
use egui::Context;
use shared::commands::Response;

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
                    Response::Success => {
                        println!("[v] yay shit just works");
                    }
                    Response::ComputerInfo(info) => {
                        if view.state.clients.contains_key(&mutex) {
                            // we already have info on this client
                            println!("[*] already got you");
                        } else {
                            println!("[*] new client to show on screen");
                            view.state.clients.insert(
                                mutex.clone(),
                                ClientView::new(mutex, info, view.mouthpiece.to_manager.clone()),
                            );
                            println!("{}", view.state.clients.len());
                        }
                    }
                    Response::Error(err) => {
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

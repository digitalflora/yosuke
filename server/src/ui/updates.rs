use crate::{types::*, ui::view::View};
use egui::Context;

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
    while let Ok(_msg) = view.mouthpiece.from_manager.try_recv() {}
}

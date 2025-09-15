use shared::commands::{Command, Response};
use tokio::{
    net::TcpStream,
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
};

use crate::{manager::client::ClientResponse, types::WhitelistedClient};

pub enum UiManagerCommand {
    SendCommand(String, Command),
    Disconnect(String),
}
pub enum UiManagerResponse {
    GetResponse(String, Response),
    Remove(String), // remove by mutex
    RemoveAll,
}
pub enum ServerManagerMessage {
    ClearClients,
    ClientConnect(WhitelistedClient, TcpStream),
    ClientDisconnect(String),
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

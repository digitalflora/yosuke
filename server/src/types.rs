use std::{collections::HashMap, sync::Arc};

use bincode::{Decode, Encode};
use tokio::{net::TcpStream, sync::Mutex};

pub type StreamCollectionPointer = Arc<Mutex<HashMap<String, StreamPointerContainer>>>;
#[derive(Clone)]
pub struct StreamPointerContainer {
    pub stream: StreamPointer,
    pub client: WhitelistedClient,
}
pub type StreamPointer = Arc<Mutex<TcpStream>>;

#[derive(Encode, Decode)]
pub struct Settings {
    pub whitelist: Vec<WhitelistedClient>,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            whitelist: Vec::new(),
        }
    }
}

#[derive(Encode, Decode, Clone)]
pub struct WhitelistedClient {
    pub mutex: String,
    pub key: [u8; 32],
}

// mpsc types
pub enum ServerMessage {
    Listening,
    Stopped,

    NewConnection(WhitelistedClient),
    Receive(String, Vec<u8>), // mutex, data
}

pub enum UiMessage {
    Listen,
    Shutdown,

    // send data to client
    Send(String, Vec<u8>), // mutex, data
}

pub struct BuilderSettings {
    pub address: String,
    pub port: u16,
}
pub enum UiBuilderMessage {
    Build(BuilderSettings),
}
pub enum BuilderMessage {
    // stuff will go here
}

pub mod mouthpieces {
    use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

    use crate::types::{BuilderMessage, ServerMessage, UiBuilderMessage, UiMessage};

    pub struct UiMouthpiece {
        pub to_server: UnboundedSender<UiMessage>,
        pub from_server: UnboundedReceiver<ServerMessage>,

        pub to_builder: UnboundedSender<UiBuilderMessage>,
        pub from_builder: UnboundedReceiver<BuilderMessage>,
    }

    pub struct ServerMouthpiece {
        pub to_ui: UnboundedSender<ServerMessage>,
        pub from_ui: UnboundedReceiver<UiMessage>,
    }
}

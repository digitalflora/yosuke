use bincode::{Decode, Encode};

#[derive(Encode, Decode)]
pub struct ClientConfig {
    pub mutex: [u8; 8],
    pub address: String,
    pub port: u16,
}

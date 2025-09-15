use bincode::{Decode, Encode};

#[derive(Encode, Decode)]
pub enum Command {
    ComputerInfo,
}
#[derive(Encode, Decode)]
pub enum Response {
    Error(String),
    ComputerInfo { hostname: String },
}
#[derive(Encode, Decode)]
pub struct BaseCommand {
    pub id: u64,
    pub command: Command,
}

#[derive(Encode, Decode)]
pub struct BaseResponse {
    pub id: u64,
    pub success: bool,
    pub response: Response,
}

use bincode::{Decode, Encode};

#[derive(Encode, Decode)]
pub enum Command {
    ComputerInfo,
    MessageBox(MessageBoxArgs),
}
#[derive(Encode, Decode)]
pub struct MessageBoxArgs {
    pub title: String,
    pub text: String,
}
#[derive(Encode, Decode)]
pub enum Response {
    Success,
    Error(String),
    ComputerInfo(ComputerInfoResponse),
}
#[derive(Encode, Decode)]
pub struct ComputerInfoResponse {
    pub hostname: String,
}

#[derive(Encode, Decode)]
pub struct BaseCommand {
    pub id: u64,
    pub command: Command,
}

#[derive(Encode, Decode)]
pub struct BaseResponse {
    pub id: u64,
    pub response: Response,
}

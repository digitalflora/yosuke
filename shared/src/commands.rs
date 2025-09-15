use bincode::{Decode, Encode};

#[derive(Encode, Decode)]
pub enum Command {
    ComputerInfo,
    MessageBox(MessageBoxArgs),
    Screenshot,
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
    Screenshot(ScreenshotResponse),
}
#[derive(Encode, Decode)]
pub struct ComputerInfoResponse {
    pub hostname: String,
}
#[derive(Encode, Decode)]
pub struct ScreenshotResponse {
    pub height: u32,
    pub width: u32,
    pub data: Vec<u8>,
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

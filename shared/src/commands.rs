use bincode::{Decode, Encode};

#[derive(Encode, Decode, Clone, PartialEq)]
pub enum CaptureQuality {
    Speed,
    Quality,
}

#[derive(Encode, Decode, Clone)]
pub enum CaptureCommand {
    Start(CaptureQuality),
    Stop,
}
#[derive(Encode, Decode, PartialEq, Eq, Hash, Clone)] // thats a lot
pub enum CaptureType {
    Screen,
    Camera,
    Mic,
    //Speaker, // not possible because of windows LOL
}
#[derive(Encode, Decode)]
pub struct VideoPacket {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}
#[derive(Encode, Decode)]
pub struct AudioPacket {
    pub data: Vec<u8>,
    pub rate: u32,
    pub channels: u16,
    pub duration: i32,
}

#[derive(Encode, Decode)]
pub enum CapturePacket {
    Video(VideoPacket),
    Audio(AudioPacket),
}

#[derive(Encode, Decode, Clone)]
pub enum Command {
    ComputerInfo,
    MessageBox(MessageBoxArgs),
    Capture(CaptureCommand, CaptureType),
}
#[derive(Encode, Decode, Clone)]
pub struct MessageBoxArgs {
    pub title: String,
    pub text: String,
}
#[derive(Encode, Decode)]
pub enum Response {
    Success,
    Error(String),
    ComputerInfo(ComputerInfoResponse),
    CapturePacket(CaptureType, CapturePacket),
}
#[derive(Encode, Decode)]
pub struct ComputerInfoResponse {
    pub hostname: String,
    pub elevated: bool, // did the client launch as admin
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

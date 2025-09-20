use egui::Ui;
use shared::commands::CaptureType;

use crate::ui::{client::ClientView, video};

pub fn render(view: &mut ClientView, ui: &mut Ui) {
    video::render(
        ui,
        CaptureType::Camera,
        &view.mutex,
        &view.sender,
        &mut view.state.input,
        &mut view.state.captures.webcam,
        &mut view.state.capturing.webcam,
        &mut view.state.textures.webcam,
    );
}

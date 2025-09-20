use egui::Ui;
use shared::commands::CaptureType;

use crate::ui::{client::ClientView, video};

pub fn render(view: &mut ClientView, ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.checkbox(&mut view.state.input.active, "Control");
    });
    video::render(
        ui,
        CaptureType::Screen,
        &view.mutex,
        &view.sender,
        &mut view.state.input,
        &mut view.state.captures.screen,
        &mut view.state.capturing.screen,
        &mut view.state.textures.screen,
    );
}

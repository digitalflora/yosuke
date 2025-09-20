use egui::{ComboBox, Ui};
use shared::commands::CaptureType;

use crate::ui::{client::ClientView, video};

pub fn render(view: &mut ClientView, ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.label("Cameras");
        ComboBox::new(format!("{}_webcams", view.mutex), "")
            .selected_text(view.info.cameras[view.state.selected_webcam as usize].clone())
            .show_ui(ui, |ui| {
                for (index, monitor) in view.info.cameras.iter().enumerate() {
                    ui.selectable_value(
                        &mut view.state.selected_webcam,
                        index as u32,
                        monitor.clone(),
                    );
                }
            });
    });

    video::render(
        ui,
        CaptureType::Camera,
        &view.mutex,
        &view.sender,
        &mut view.state.input,
        &mut view.state.captures.webcam,
        &mut view.state.capturing.webcam,
        &mut view.state.textures.webcam,
        view.state.selected_webcam,
    );
}

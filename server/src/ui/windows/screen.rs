use egui::{ComboBox, Ui};
use shared::commands::CaptureType;

use crate::ui::{client::ClientView, video};

pub fn render(view: &mut ClientView, ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.label("Monitors");
        ComboBox::new(format!("{}_monitors", view.mutex), "")
            .selected_text(view.info.monitors[view.state.selected_monitor as usize].clone())
            .show_ui(ui, |ui| {
                for (index, monitor) in view.info.monitors.iter().enumerate() {
                    ui.selectable_value(
                        &mut view.state.selected_monitor,
                        index as u32,
                        monitor.clone(),
                    );
                }
            });

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
        view.state.selected_monitor,
    );
}

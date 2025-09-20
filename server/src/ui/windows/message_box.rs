use egui::{Align, Layout, TextEdit, Ui, vec2};
use shared::commands::{Command, MessageBoxArgs};

use crate::{manager::types::UiManagerCommand, ui::client::ClientView};

pub fn render(view: &mut ClientView, ui: &mut Ui) {
    ui.add(TextEdit::singleline(&mut view.state.msgbox.title).desired_width(ui.available_width()));
    ui.add_sized(
        vec2(
            ui.available_width(),
            (ui.available_height() - 28.0).clamp(0.0, ui.available_height()), // worst code ever award
        ),
        TextEdit::multiline(&mut view.state.msgbox.text).desired_width(ui.available_width()),
    );

    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
        if ui.button("Send").clicked() {
            let _ = view.sender.send(UiManagerCommand::SendCommand(
                view.mutex.clone(),
                Command::MessageBox(MessageBoxArgs {
                    title: view.state.msgbox.title.clone(),
                    text: view.state.msgbox.text.clone(),
                }),
            ));
        }
    });
}

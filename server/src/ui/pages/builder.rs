use egui::{
    Align, Color32, Frame, Grid, Layout, RichText, ScrollArea, TextEdit, TextStyle, Ui, vec2,
};

use crate::{
    types::{BuilderSettings, UiBuilderMessage},
    ui::view::View,
};

pub fn render(view: &mut View, ui: &mut Ui) {
    ui.heading("Builder");

    Grid::new("builder_grid")
        .num_columns(2)
        .spacing([10.0, 4.0])
        .show(ui, |ui| {
            ui.label("Address:");
            ui.add(TextEdit::singleline(&mut view.state.builder.address).desired_width(128.0));
            ui.end_row();

            ui.label("Port:");
            ui.add(TextEdit::singleline(&mut view.state.builder.port).desired_width(64.0));
            ui.end_row();

            if ui.button("Build").clicked() {
                if let Ok(num) = view.state.builder.port.parse::<u16>() {
                    let _ =
                        view.mouthpiece
                            .to_builder
                            .send(UiBuilderMessage::Build(BuilderSettings {
                                address: view.state.builder.address.clone(),
                                port: num,
                            }));
                } else {
                    view.state
                        .notifications
                        .error("Invalid port in Builder, cannot build!")
                        .duration(Some(std::time::Duration::from_secs(5)));
                };
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
                            RichText::new(&view.state.logs.builder.join("\n"))
                                .monospace()
                                .size(12.0)
                                .text_style(TextStyle::Monospace),
                        )
                    })
            })
    });
}

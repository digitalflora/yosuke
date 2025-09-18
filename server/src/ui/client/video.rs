use egui::{Image, Slider, TextureHandle, Ui};
use shared::commands::{CaptureCommand, CaptureQuality, CaptureType, Command};
use tokio::sync::mpsc::UnboundedSender;

use crate::{manager::types::UiManagerCommand, ui::client::{types::ClientViewCapture, ClientView}};

pub fn render(
    ui: &mut Ui,
    capture_type: CaptureType,
    mutex: &String,
    sender: &UnboundedSender<UiManagerCommand>,
    capture: &mut ClientViewCapture,
    capturing: &mut bool,
    texture: &mut Option<TextureHandle>,

) {
     if let Some(ref image) = capture.data {
        if texture.is_none() {
           *texture = Some(ui.ctx().load_texture(
                format!("screen_{}", mutex.clone()),
                image.clone(),
                Default::default(),
            ));
        };
        if let Some(texture) = texture.as_ref() {
            let available_size = ui.available_size();
            let image_size = texture.size_vec2();
            let max_width = available_size.x.min(720.0);
            let max_height = available_size.y.min(560.0);
            let scale_x = max_width / image_size.x;
            let scale_y = max_height / image_size.y;
            let scale = scale_x.min(scale_y).min(1.0);
            let display_size = image_size * scale * capture.scale;
            ui.add(Image::new(texture).max_size(display_size));
        }
    }

    if *capturing {
        if ui.button("⏹  Stop").clicked() {
            println!("[*] sending CaptureCommand::Stop");
            let _ = sender.send(UiManagerCommand::SendCommand(
                mutex.clone(),
                Command::Capture(CaptureCommand::Stop, capture_type),
            ));
            *capturing = false;
        }
    } else {
        if ui.button("▶  Start").clicked() {
            println!("[*] sending CaptureCommand::Start");
            let _ = sender.send(UiManagerCommand::SendCommand(
                mutex.clone(),
                Command::Capture(
                    CaptureCommand::Start(
                        capture.quality.clone(),
                    ),
                    capture_type,
                ),
            ));
            *capturing = true;
        };
    }

    //////////////////////////////////////
    // quality toggle
    ui.add_enabled_ui(!*capturing, |ui| {
        ui.horizontal(|ui| {
            ui.label("Quality: ");
            ui.radio_value(
                &mut capture.quality,
                CaptureQuality::Quality,
                "Slow",
            );
            ui.radio_value(
                &mut capture.quality,
                CaptureQuality::Speed,
                "Fast",
            );
        });
    });

    ui.add(Slider::new(&mut capture.scale, 0.15..=2.0).text("Scale"));
}
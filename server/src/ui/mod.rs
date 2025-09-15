use eframe::{NativeOptions, run_native};
use egui::{IconData, ViewportBuilder};

use crate::types::mouthpieces::UiMouthpiece;

mod client;
mod pages;
mod switcher;
mod updates;
mod view;

pub fn main(mouthpiece: UiMouthpiece) -> eframe::Result<()> {
    println!("[*] ui spawned");
    let options = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([720.0, 560.0])
            .with_icon(load_icon()),
        ..Default::default()
    };

    run_native(
        "Yosuke",
        options,
        Box::new(move |_cc| Ok(Box::new(view::View::new(mouthpiece)))),
    )
}

fn load_icon() -> IconData {
    let (icon_rgba, icon_width, icon_height) = {
        let icon = include_bytes!("../../../yosuke.ico");
        let image = image::load_from_memory(icon)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };

    eframe::egui::IconData {
        rgba: icon_rgba,
        width: icon_width,
        height: icon_height,
    }
}

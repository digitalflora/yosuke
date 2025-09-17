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
        hardware_acceleration: eframe::HardwareAcceleration::Preferred, // is it really this easy...?
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

fn get_icon() -> Vec<u8> {
    #[cfg(target_os = "macos")]
    {
        return include_bytes!("../../../assets/yosuke.png").to_vec();
    }
    #[cfg(not(target_os = "macos"))]
    {
        return include_bytes!("../../../assets/yosuke.ico").to_vec();
    }
}

fn load_icon() -> IconData {
    let (icon_rgba, icon_width, icon_height) = {
        let icon = &get_icon();
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

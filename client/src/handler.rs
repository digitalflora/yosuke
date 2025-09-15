use shared::commands::{Command, Response};

use crate::commands::{capture, computer_info, message_box};

pub fn main(command: Command) -> Response {
    match command {
        Command::ComputerInfo => match computer_info::main() {
            Ok(info) => info,
            Err(err) => Response::Error(err.to_string()),
        },
        Command::MessageBox(args) => match message_box::main(args) {
            Ok(info) => info,
            Err(err) => Response::Error(err.to_string()),
        },
        Command::Screenshot => match capture::screen::main() {
            Ok(info) => info,
            Err(err) => Response::Error(err.to_string()),
        },
    }
}

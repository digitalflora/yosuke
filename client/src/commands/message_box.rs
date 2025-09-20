use std::ptr::null_mut;

use shared::commands::{MessageBoxArgs, MessageBoxIcon, Response};
use winapi::um::winuser::{MB_ICONERROR, MB_ICONINFORMATION, MB_ICONWARNING, MB_OK, MessageBoxW};

use crate::wstring;

pub fn main(args: MessageBoxArgs) -> Result<Response, Box<dyn std::error::Error>> {
    let title = wstring(&args.title);
    let text = wstring(&args.text);

    let mut icon_bit = MB_ICONINFORMATION;
    if args.icon == MessageBoxIcon::Error {
        icon_bit = MB_ICONERROR;
    } else if args.icon == MessageBoxIcon::Warning {
        icon_bit = MB_ICONWARNING;
    }

    unsafe {
        MessageBoxW(null_mut(), text.as_ptr(), title.as_ptr(), MB_OK | icon_bit);
    };

    Ok(Response::Success)
}

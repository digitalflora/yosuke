use std::{ptr::null_mut};

use shared::commands::{MessageBoxArgs, Response};
use winapi::um::winuser::{MessageBoxW, MB_ICONINFORMATION, MB_OK};

use crate::wstring;

pub fn main(args: MessageBoxArgs) -> Result<Response, Box<dyn std::error::Error>> {
    let title = wstring(&args.title);
    let text = wstring(&args.text);

    unsafe {
        MessageBoxW(null_mut(), text.as_ptr(), title.as_ptr(), MB_OK | MB_ICONINFORMATION);
    };

    Ok(Response::Success)
}

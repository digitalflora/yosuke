use shared::commands::{MessageBoxArgs, Response};
use winsafe::co;

pub fn main(args: MessageBoxArgs) -> Result<Response, Box<dyn std::error::Error>> {
    let title = &args.title;
    let text = &args.text;

    let hwnd = winsafe::HWND::GetDesktopWindow();
    let msgbox = hwnd.MessageBox(text, title, co::MB::OK | co::MB::ICONINFORMATION);
    match msgbox {
        Ok(_) => Ok(Response::Success),
        Err(err) => Err(Box::new(err)),
    }
}

use is_elevated::is_elevated;
use shared::commands::{ComputerInfoResponse, Response};
use winapi::um::winbase::GetComputerNameW;

pub fn main() -> Result<Response, Box<dyn std::error::Error>> {
    let mut hostname_buf = [0u16; 16];
    unsafe { GetComputerNameW(hostname_buf.as_mut_ptr(), &mut (hostname_buf.len() as u32)) };
    Ok(Response::ComputerInfo(ComputerInfoResponse {
        hostname: String::from_utf16_lossy(&hostname_buf).to_string(),
        elevated: is_elevated(),
    }))
}

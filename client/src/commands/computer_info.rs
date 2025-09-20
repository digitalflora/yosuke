use is_elevated::is_elevated;
use nokhwa::query;
use shared::commands::{ComputerInfoResponse, Response};
use winapi::um::winbase::GetComputerNameW;
use windows_capture::monitor::Monitor;

pub fn main() -> Result<Response, Box<dyn std::error::Error>> {
    let mut hostname_buf = [0u16; 16];
    unsafe { GetComputerNameW(hostname_buf.as_mut_ptr(), &mut (hostname_buf.len() as u32)) };

    let monitors: Vec<String> = Monitor::enumerate()
        .unwrap_or_default()
        .into_iter()
        .map(|m| m.name().unwrap_or_else(|_| "<Unknown>".to_string()))
        .collect();

    let cameras = query(nokhwa::utils::ApiBackend::Auto)
        .unwrap_or_default()
        .into_iter()
        .map(|c| c.human_name())
        .collect::<Vec<_>>();

    Ok(Response::ComputerInfo(ComputerInfoResponse {
        hostname: String::from_utf16_lossy(&hostname_buf).to_string(),
        elevated: is_elevated(),
        monitors,
        cameras,
    }))
}

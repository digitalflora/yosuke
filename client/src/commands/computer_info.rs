use shared::commands::{ComputerInfoResponse, Response};
use winsafe::GetComputerName;

pub fn main() -> Result<Response, Box<dyn std::error::Error>> {
    let hostname = GetComputerName()?;
    Ok(Response::ComputerInfo(ComputerInfoResponse {
        hostname: hostname,
    }))
}

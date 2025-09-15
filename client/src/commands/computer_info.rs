use shared::commands::Response;
use winsafe::GetComputerName;

pub async fn main() -> Result<Response, Box<dyn std::error::Error>> {
    let hostname = GetComputerName()?;
    Ok(Response::ComputerInfo { hostname })
}

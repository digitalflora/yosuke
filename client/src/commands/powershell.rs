use std::{io::Read, process::{Command, Stdio}};

use shared::commands::Response;

fn run(cmd: &str) -> Result<String, String> {
    let mut child = Command::new("powershell.exe")
        .args(&[
            "-NoProfile",
            "-ExecutionPolicy", "Bypass",
            "-WindowStyle", "Hidden",
            "-Command", cmd
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to spawn PowerShell process: {}", e))?;

    // Capture stdout and stderr
    let mut stdout = String::new();
    let mut stderr = String::new();

    if let Some(ref mut stdout_handle) = child.stdout {
        stdout_handle.read_to_string(&mut stdout)
            .map_err(|e| format!("Failed to read stdout: {}", e))?;
    }

    if let Some(ref mut stderr_handle) = child.stderr {
        stderr_handle.read_to_string(&mut stderr)
            .map_err(|e| format!("Failed to read stderr: {}", e))?;
    }

    // Wait for the process to complete
    let exit_status = child.wait()
        .map_err(|e| format!("Failed to wait for PowerShell process: {}", e))?;

    // Check exit status and return appropriate result
    if exit_status.success() {
        Ok(stdout.trim().to_string())
    } else {
        let error_msg = if stderr.trim().is_empty() {
            format!("PowerShell exited with code: {:?}", exit_status.code())
        } else {
            format!("PowerShell error: {}", stderr.trim())
        };
        Err(error_msg)
    }
}

pub fn main(cmd: String) -> Response {
    match run(&cmd) {
        Ok(_) => Response::Success,
        Err(_e) => Response::Error(_e)
    }
}

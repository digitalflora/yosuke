use std::{
    io::Read,
    os::windows::process::CommandExt,
    process::{Command, Stdio},
};

use shared::commands::Response;

fn run(cmd: &str, run_powershell: bool) -> Result<String, String> {
    let mut command = if run_powershell {
        Command::new("powershell.exe")
    } else {
        Command::new("cmd.exe")
    };

    let mut child = if run_powershell {
        command
            .args(&[
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-WindowStyle",
                "Hidden",
                "-Command",
                cmd,
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null())
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .spawn()
            .map_err(|e| format!("Failed to spawn PowerShell process: {}", e))?
    } else {
        command
            .args(&["/C", cmd])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null())
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .spawn()
            .map_err(|e| format!("Failed to spawn CMD process: {}", e))?
    };

    // Capture stdout and stderr
    let mut stdout = String::new();
    let mut stderr = String::new();

    if let Some(ref mut stdout_handle) = child.stdout {
        stdout_handle
            .read_to_string(&mut stdout)
            .map_err(|e| format!("Failed to read stdout: {}", e))?;
    }

    if let Some(ref mut stderr_handle) = child.stderr {
        stderr_handle
            .read_to_string(&mut stderr)
            .map_err(|e| format!("Failed to read stderr: {}", e))?;
    }

    // Wait for the process to complete
    let exit_status = child
        .wait()
        .map_err(|e| format!("Failed to wait for process: {}", e))?;

    // Check exit status and return appropriate result
    if exit_status.success() {
        Ok(stdout.trim().to_string())
    } else {
        if stderr.trim().is_empty() {
            Err(format!(
                "Process exited with code: {:?}",
                exit_status.code()
            ))
        } else {
            Ok(stderr.trim().to_string())
        }
    }
}

pub fn main(cmd: String, run_powershell: bool) -> Response {
    match run(&cmd, run_powershell) {
        Ok(stdout) => Response::PowerShell(stdout),
        Err(_e) => Response::Error(_e),
    }
}

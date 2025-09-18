use std::{ffi::OsStr, os::windows::ffi::OsStrExt};

use shared::commands::Response;

fn elevated_restart() -> Result<(), Box<dyn std::error::Error>> {
    let exe_path = std::env::current_exe()?;
    
    // Convert paths to wide strings (UTF-16)
    let exe_path_wide: Vec<u16> = OsStr::new(&exe_path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    
    let verb: Vec<u16> = OsStr::new("runas")
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        use std::ptr;
        use winapi::um::shellapi::ShellExecuteW;
        use winapi::um::winuser::SW_HIDE;

        let result = ShellExecuteW(
            ptr::null_mut(),           // hwnd
            verb.as_ptr(),             // "runas"
            exe_path_wide.as_ptr(),    // executable path
            ptr::null(),               // parameters (none)
            ptr::null(),               // directory (current)
            SW_HIDE,                   // Hide the window
        );
        
        if result as i32 <= 32 {
            return Err("Failed relaunch!".into());
        }
    }
    
    // Exit the current non-elevated process
    std::process::exit(0);
}

pub fn main() -> Result<Response, Box<dyn std::error::Error>> {
    let _ = elevated_restart();
    Ok(Response::Success) // we wont ever get here
}

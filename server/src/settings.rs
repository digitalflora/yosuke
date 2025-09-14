use bincode::{config, decode_from_slice, encode_to_vec};
use std::fs;

use crate::{builder::running_dir, types::Settings};

pub async fn load() -> Result<Settings, Box<dyn std::error::Error>> {
    let mut settings_path = running_dir();
    settings_path.push("settings.dat");
    match fs::read(settings_path) {
        Ok(settings_content) => {
            let (settings, _length): (Settings, usize) =
                decode_from_slice(settings_content.as_slice(), config::standard())?;
            println!("[v] loaded settings.dat");
            return Ok(settings);
        }
        Err(_e) => {
            /* make a new one */
            println!("[*] could not load settings, reset to default!");
            return Ok(Settings::default());
        }
    }
}

pub async fn save(settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let encoded_settings = encode_to_vec(settings, config::standard())?;
    let mut settings_path = running_dir();
    settings_path.push("settings.dat");
    fs::write(settings_path, encoded_settings)?;
    println!("[v] saved settings.dat");
    Ok(())
}

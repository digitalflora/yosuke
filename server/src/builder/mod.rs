use std::path::PathBuf;

use crate::{
    SettingsPointer, settings,
    types::{UiBuilderMessage, mouthpieces::BuilderMouthpiece},
};
use aes_gcm::aead::rand_core::{self, RngCore};
use rfd::FileDialog;
use shared::types::ClientConfig;

mod build;

fn mutex() -> Result<[u8; 8], Box<dyn std::error::Error>> {
    let mut mutex_buf = [0u8; 8];
    rand_core::OsRng.fill_bytes(&mut mutex_buf);
    Ok(mutex_buf)
}
pub fn running_dir() -> PathBuf {
    let exe_path = std::env::current_exe().expect("Failed to get current_exe");

    #[cfg(target_os = "macos")]
    {
        // /Yosuke.app/Contents/MacOS/server → /Yosuke.app
        exe_path
            .parent() // MacOS
            .and_then(|p| p.parent()) // Contents
            .and_then(|p| p.parent()) // .app
            .and_then(|p| p.parent()) // parent folder
            .map(|p| p.to_path_buf())
            .expect("Failed to resolve .app bundle path")
    }

    #[cfg(not(target_os = "macos"))]
    {
        // /Yosuke.exe
        exe_path
            .parent()
            .map(|p| p.to_path_buf())
            .expect("Failed to resolve executable directory")
    }
}

pub async fn main(settings: SettingsPointer, mut mouthpiece: BuilderMouthpiece) {
    println!("[*] builder spawned");
    while let Some(command) = mouthpiece.from_ui.recv().await {
        match command {
            UiBuilderMessage::Build(builder_settings) => {
                println!("[*] generating mutex");
                let mutex = mutex().unwrap();
                let config = ClientConfig {
                    mutex: mutex,
                    address: builder_settings.address,
                    port: builder_settings.port,
                };
                println!("[*] opening save file dialog");
                if let Some(output_path) = FileDialog::new()
                    .add_filter("Executable", &["exe"])
                    .set_directory(running_dir())
                    .save_file()
                {
                    let out_path = output_path.to_str().unwrap();
                    println!("[*] saving to:\n    {}", out_path);

                    match build::main(&config, out_path).await {
                        Ok(client) => {
                            let mut _settings = settings.lock().await;
                            _settings.whitelist.push(client);
                            println!("[*] added mutex {} to whitelist", hex::encode(mutex));
                            match settings::save(&*_settings).await {
                                Ok(_) => {
                                    println!("[*] settings saved");
                                }
                                Err(e) => {
                                    println!("[!] error saving settings: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            println!("[!] error building: {}", e);
                        }
                    }
                }
            }
        }
    }
}

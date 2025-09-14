use crate::{
    SettingsPointer, settings,
    types::{BuilderMessage, UiBuilderMessage},
};
use aes_gcm::aead::rand_core::{self, RngCore};
use rfd::FileDialog;
use shared::types::ClientConfig;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

mod build;

fn mutex() -> Result<[u8; 8], Box<dyn std::error::Error>> {
    let mut mutex_buf = [0u8; 8];
    rand_core::OsRng.fill_bytes(&mut mutex_buf);
    Ok(mutex_buf)
}
pub fn running_dir() -> std::path::PathBuf {
    std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

pub async fn main(
    mut settings: SettingsPointer,
    mut to_ui: UnboundedSender<BuilderMessage>,
    mut from_ui: UnboundedReceiver<UiBuilderMessage>,
) {
    println!("[*] builder task spawned");
    while let Some(command) = from_ui.recv().await {
        match command {
            UiBuilderMessage::Build(builder_settings) => {
                println!("[*] generating mutex");
                let config = ClientConfig {
                    mutex: mutex().unwrap(),
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

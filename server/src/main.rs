use std::sync::Arc;

use crate::types::{
    mouthpieces::{ServerMouthpiece, UiMouthpiece},
    *,
};
use tokio::sync::{Mutex, mpsc::unbounded_channel};

mod net;
mod types;
mod ui;

mod builder;
mod settings;

pub type SettingsPointer = Arc<Mutex<Settings>>;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = SettingsPointer::new(Mutex::new(settings::load().await?));

    // ui <---> server
    let (to_ui, from_server) = unbounded_channel::<ServerMessage>();
    let (to_server, from_ui) = unbounded_channel::<UiMessage>();

    // ui <---> builder
    let (to_ui_builder, from_builder) = unbounded_channel::<BuilderMessage>();
    let (to_builder, from_ui_builder) = unbounded_channel::<UiBuilderMessage>();

    let mut settings_pointer = settings.clone();
    tokio::spawn(async move {
        if let Err(e) = net::listen::main(
            settings_pointer,
            ServerMouthpiece {
                to_ui: to_ui,
                from_ui: from_ui,
            },
        )
        .await
        {
            eprintln!("[x][main()->listen()] {}", e);
        }
    }); // server thread
    // to_server.send(UiMessage::Listen)?; // tell server to start listening!

    settings_pointer = settings.clone();
    tokio::task::spawn(async move {
        builder::main(settings_pointer, to_ui_builder, from_ui_builder).await
    }); // builder task

    // ui runs on the main thread
    let _ = ui::main(UiMouthpiece {
        to_server: to_server,
        from_server: from_server,

        to_builder: to_builder,
        from_builder: from_builder,
    });

    // ctrl_c().await?;
    Ok(())
}

use std::sync::Arc;

use crate::{
    manager::{ClientManager, client::ClientResponse, types::*},
    types::{
        mouthpieces::{BuilderMouthpiece, ServerMouthpiece, UiMouthpiece},
        *,
    },
};
use tokio::sync::{Mutex, mpsc::unbounded_channel};

mod builder;
mod manager;
mod net;
mod settings;
mod types;
mod ui;

pub type SettingsPointer = Arc<Mutex<Settings>>;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = SettingsPointer::new(Mutex::new(settings::load().await?));

    // ui <---> server
    let (to_ui_server, from_server) = unbounded_channel::<ServerMessage>();
    let (to_server, from_ui_server) = unbounded_channel::<UiMessage>();

    // ui <---> builder
    let (to_ui_builder, from_builder) = unbounded_channel::<BuilderMessage>();
    let (to_builder, from_ui_builder) = unbounded_channel::<UiBuilderMessage>();

    // manager channels
    let (ui_to_manager, manager_from_ui) = unbounded_channel::<UiManagerCommand>();
    let (manager_to_ui, ui_from_manager) = unbounded_channel::<UiManagerResponse>();
    let (server_to_manager, manager_from_server) = unbounded_channel::<ServerManagerMessage>();

    let (client_to_manager /* you can clone this */, manager_from_client) =
        unbounded_channel::<ClientResponse>();

    tokio::spawn(async move {
        let manager: ClientManager = ClientManager::new(ClientManagerMouthpiece {
            from_ui: manager_from_ui,
            to_ui: manager_to_ui,
            from_server: manager_from_server,
            client: SharedClientMouthpiece {
                from_client: manager_from_client,
                to_manager: client_to_manager,
            },
        });
        manager.run().await;
    });

    let mut settings_pointer = settings.clone();
    tokio::spawn(async move {
        if let Err(e) = net::listen::main(
            settings_pointer,
            ServerMouthpiece {
                to_ui: to_ui_server,
                from_ui: from_ui_server,
                to_manager: server_to_manager,
            },
        )
        .await
        {
            eprintln!("[x][main()->listen()] {}", e);
        }
    }); // server thread

    settings_pointer = settings.clone();
    tokio::spawn(async move {
        builder::main(
            settings_pointer,
            BuilderMouthpiece {
                to_ui: to_ui_builder,
                from_ui: from_ui_builder,
            },
        )
        .await
    }); // builder task

    // ui runs on the main thread
    if let Err(err) = ui::main(UiMouthpiece {
        to_server: to_server,
        from_server: from_server,

        to_builder: to_builder,
        from_builder: from_builder,

        to_manager: ui_to_manager,
        from_manager: ui_from_manager,
    }) {
        eprintln!("[x] ui error\n    {}", err);
    }

    // ctrl_c().await?;
    Ok(())
}

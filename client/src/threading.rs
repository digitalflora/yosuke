use std::{collections::HashMap, sync::Arc, thread};

use shared::commands::{BaseCommand, BaseResponse, Response};
use smol::{Task, channel::Sender, lock::Mutex};

use crate::handler;

pub struct ActiveCommands {
    tasks: Arc<Mutex<HashMap<u64, Task<()>>>>,
}
impl ActiveCommands {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn spawn(&self, command: BaseCommand, tx: Sender<Vec<u8>>) {
        let id = command.id;
        let tasks = Arc::clone(&self.tasks);

        let parallelism = thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);
        let current_tasks = {
            let lock = self.tasks.lock().await;
            lock.len()
        };
        let max_tasks = if parallelism > 1 { parallelism - 1 } else { 1 };
        if current_tasks >= max_tasks {
            let refusal = bincode::encode_to_vec(
                BaseResponse {
                    id: command.id,
                    response: Response::Error("Client is fat!".to_string()),
                },
                bincode::config::standard(),
            )
            .unwrap();
            let _ = tx.send(refusal).await;
            return;
        }

        let handle = smol::spawn(async move {
            let res = smol::unblock(move || handler::main(command.command)).await;
            let res_data = bincode::encode_to_vec(
                BaseResponse {
                    id: command.id,
                    response: res,
                },
                bincode::config::standard(),
            )
            .unwrap();

            let _ = tx.send(res_data).await;

            let mut lock = tasks.lock().await;
            lock.remove(&id);
            println!("[*] task finished");
        });
        let mut lock = self.tasks.lock().await;
        lock.insert(id, handle);
        println!("[*] spawned task");
    }

    async fn halt(&self) {
        let mut lock = self.tasks.lock().await;
        for (_, handle) in lock.drain() {
            handle.cancel().await;
        }
    }
}

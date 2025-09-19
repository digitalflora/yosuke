use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
};

use enigo::{Enigo, Settings};
use shared::commands::{BaseCommand, BaseResponse, CaptureCommand, CaptureType, Command, Response};
use smol::{Task, channel::Sender, lock::Mutex};

use crate::{handler, input};

pub struct CaptureTaskState {
    id: u64,
    active: Arc<AtomicBool>,
}
pub struct ActiveCommands {
    tasks: Arc<Mutex<HashMap<u64, Task<()>>>>,
    captures: Arc<Mutex<HashMap<CaptureType, CaptureTaskState>>>, // type, command id
    enigo: Enigo,
}
impl ActiveCommands {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(Mutex::new(HashMap::new())),
            captures: Arc::new(Mutex::new(HashMap::new())),
            enigo: Enigo::new(&Settings::default()).unwrap(),
        }
    }

    pub async fn spawn(&mut self, command: BaseCommand, tx: Sender<Vec<u8>>) {
        let id = command.id;
        let tasks = Arc::clone(&self.tasks);

        // stop capturing
        if let Command::Capture(CaptureCommand::Stop, capture_type) = &command.command {
            let mut captures = self.captures.lock().await;
            if let Some(task_state) = captures.get(capture_type) {
                task_state.active.store(false, Ordering::SeqCst);

                let task = {
                    let mut tasks = self.tasks.lock().await;
                    tasks.remove(&task_state.id)
                };

                if let Some(task) = task {
                    task.cancel().await; // wait for task to stop
                    captures.remove(capture_type);
                }
            }
            return;
        }

        // handle input before we try to refuse to reserve a thread
        // design choice: a whole new thread is not needed to handle input bruh
        if input::main(&command, &mut self.enigo) {
            return; // stop running if we handled an input this cycle
        };

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

        // start capturing
        let mut running: Option<Arc<AtomicBool>> = None;
        if let Command::Capture(CaptureCommand::Start(_), capture_type) = &command.command {
            let mut captures = self.captures.lock().await;
            if captures.contains_key(capture_type) {
                let refusal = bincode::encode_to_vec(
                    BaseResponse {
                        id: command.id,
                        response: Response::Error("Capture already started!".to_string()),
                    },
                    bincode::config::standard(),
                )
                .unwrap();
                let _ = tx.send(refusal).await;
                return;
            };

            let capture_state = CaptureTaskState {
                id,
                active: Arc::new(AtomicBool::new(true)),
            };
            running = Some(capture_state.active.clone());
            captures.insert(capture_type.clone(), capture_state);
        }

        // handle capture in here because i don't want to move shit to another thread, that's looong
        let tx_clone = tx.clone();
        let handle = smol::spawn(async move {
            smol::unblock(move || handler::main(command, tx_clone, running)).await;

            let mut tasks = tasks.lock().await;
            tasks.remove(&id);
        });
        let mut lock = self.tasks.lock().await;
        lock.insert(id, handle);
        println!("[*] spawned task");
    }

    /*async fn halt(&self) {
        let mut lock = self.tasks.lock().await;
        for (_, handle) in lock.drain() {
            handle.cancel().await;
        }
    }*/
}

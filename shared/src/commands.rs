use bincode::{Decode, Encode};

#[derive(Encode, Decode)]
pub enum Command {
    ComputerInfo,
}
#[derive(Encode, Decode)]
pub struct BaseCommand {
    pub id: u64,
    pub command: Command,
}

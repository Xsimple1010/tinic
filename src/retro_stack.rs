use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub enum StackCommand {
    LoadGame(String),
    UnloadGame,
    SaveState,
    LoadState,
    Pause,
    Resume,
    UpdateControllers,
    GameQuit,
    Reset,
}

pub struct RetroStack {
    command: Mutex<Vec<StackCommand>>,
}

impl RetroStack {
    pub fn new() -> Arc<RetroStack> {
        Arc::new(Self {
            command: Mutex::new(Vec::new()),
        })
    }

    pub fn push(&self, command: StackCommand) {
        self.command.lock().unwrap().push(command);
    }

    pub fn read(&self) -> Vec<StackCommand> {
        let m_cmd = &mut *self.command.lock().unwrap();
        let cmd = m_cmd.clone().to_vec();

        m_cmd.clear();

        cmd
    }
}

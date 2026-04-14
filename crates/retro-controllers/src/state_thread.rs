use crate::devices_manager::DevicesManager;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{
    thread::{self, sleep},
    time::Duration,
};
use tinic_generics::constants::THREAD_SLEEP_TIME;

#[derive(Debug)]
pub struct EventThread {
    event_thread_can_run: Arc<AtomicBool>,
}

impl EventThread {
    pub fn new() -> Self {
        EventThread {
            event_thread_can_run: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn stop(&self) {
        self.event_thread_can_run.store(false, Ordering::SeqCst);
    }

    pub fn resume(&self, devices: Arc<DevicesManager>) {
        if self.event_thread_can_run.load(Ordering::SeqCst) {
            return;
        }

        self.event_thread_can_run.store(true, Ordering::SeqCst);
        self.create_update_devices_state_thread(devices);
    }

    /// # event listener thread
    ///
    /// Isso é util se quando não há uma *rom* em execução, mas ainda é necessário ouvir os eventos de
    /// input. Por exemplo, a *rom* foi fechada, mas a interface do usuário ainda precisa ser
    /// notificada sobre os eventos de input.
    ///
    /// Aviso: para evitar uso desnecessário de CPU use isso somente quando não hover uma
    /// *rom* em execução!
    fn create_update_devices_state_thread(&self, devices: Arc<DevicesManager>) {
        let event_thread_is_enabled = self.event_thread_can_run.clone();

        thread::spawn(move || {
            while event_thread_is_enabled.load(Ordering::SeqCst) {
                //WITHOUT THIS, WI HAVE A HIGH CPU UTILIZATION!
                sleep(Duration::from_millis(THREAD_SLEEP_TIME));

                if let Err(e) = devices.update_state() {
                    println!("erro na thread de controle: {e:?}");
                }
            }
        });
    }
}

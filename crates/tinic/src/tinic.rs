use crate::channel::ThreadChannel;
use crate::game_thread::game_thread_handle::GameThread;
use crate::thread_stack::main_stack::{SaveImg, SavePath};
use async_std::task;
use generics::erro_handle::ErroHandle;
use generics::retro_paths::RetroPaths;
use libretro_sys::binding_libretro::retro_log_level::RETRO_LOG_ERROR;
use retro_controllers::devices_manager::{Device, DeviceState, DeviceStateListener};
use retro_controllers::RetroController;
use retro_core::option_manager::OptionManager;
use std::sync::{Arc, Mutex, RwLock};

static DEVICE_STATE_LISTENER: RwLock<Option<DeviceStateListener>> = RwLock::new(None);

lazy_static! {
    static ref CHANNEL: Arc<ThreadChannel> = Arc::new(ThreadChannel::new());
}

fn device_state_listener(state: DeviceState, device: Device) {
    if let Some(listener) = DEVICE_STATE_LISTENER.read().unwrap().as_ref() {
        match &state {
            DeviceState::Connected | DeviceState::Disconnected => {
                CHANNEL.connect_device(device.clone());
            }
            _ => {}
        }
        listener(state, device);
    };
}

pub struct Tinic {
    pub controller: Arc<Mutex<RetroController>>,
    game_thread: GameThread,
    pub core_options: Option<Arc<OptionManager>>,
}

impl Tinic {
    pub fn new(listener: Option<DeviceStateListener>) -> Result<Tinic, ErroHandle> {
        match DEVICE_STATE_LISTENER.write() {
            Ok(mut device_listener) => *device_listener = listener,
            Err(e) => {
                return Err(ErroHandle {
                    level: RETRO_LOG_ERROR,
                    message: e.to_string(),
                })
            }
        }

        let controller_ctx = Arc::new(Mutex::new(RetroController::new(Some(
            device_state_listener,
        ))?));

        Ok(Self {
            game_thread: GameThread::new(controller_ctx.clone()),
            core_options: None,
            controller: controller_ctx,
        })
    }

    pub fn load_game(
        &mut self,
        core_path: &str,
        rom_path: &str,
        paths: RetroPaths,
    ) -> Result<bool, ErroHandle> {
        self.game_thread.start(CHANNEL.get_notify())?;

        let (loaded, options) = task::block_on(CHANNEL.load_game(core_path, rom_path, paths));
        self.core_options = options;

        Ok(loaded)
    }

    pub fn pause(&self) {
        CHANNEL.pause_game();
    }

    pub fn resume(&self) {
        CHANNEL.resume_game();
    }

    pub fn save_state(&self, slot: usize) -> Option<(SavePath, SaveImg)> {
        task::block_on(CHANNEL.save_state(slot))
    }

    pub fn load_state(&self, slot: usize) -> bool {
        task::block_on(CHANNEL.load_state(slot))
    }

    pub fn connect_device(device: Device) {
        CHANNEL.connect_device(device);
    }

    pub fn reset(&self) {
        CHANNEL.reset_game();
    }

    pub fn quit(&mut self) {
        self.core_options.take();
        self.game_thread.stop();
    }

    pub fn enable_full_screen(&self) {
        CHANNEL.enable_full_screen();
    }

    pub fn disable_full_screen(&self) {
        CHANNEL.disable_full_screen();
    }
}

use std::sync::Arc;

use crate::devices_manager::{DeviceListener, DeviceRubble, DevicesManager};
use crate::gamepad::retro_gamepad::RetroGamePad;
use crate::state_thread::EventThread;
use generics::erro_handle::ErroHandle;
use libretro_sys::binding_libretro::retro_rumble_effect;
use retro_core::RetroControllerEnvCallbacks;

#[derive(Debug)]
pub struct RetroController {
    event_thread: EventThread,
    manager: Arc<DevicesManager>,
}

impl Drop for RetroController {
    fn drop(&mut self) {
        self.event_thread.stop();
    }
}

impl RetroController {
    pub fn new(listener: Box<dyn DeviceListener>) -> Result<RetroController, ErroHandle> {
        let manager = Arc::new(DevicesManager::new(listener)?);

        let event_thread = EventThread::new();
        event_thread.resume(manager.clone())?;

        Ok(Self {
            event_thread,
            manager,
        })
    }

    #[doc = "retorna uma lista de gamepad disponíveis"]
    pub fn get_list(&self) -> Result<Vec<RetroGamePad>, ErroHandle> {
        Ok(self.manager.get_gamepads())
    }

    pub fn set_max_port(&self, max: usize) -> Result<(), ErroHandle> {
        self.manager.set_max_port(max);
        Ok(())
    }

    #[doc = "Para que o CORE possa 'tomar posse' com existo dos eventos do gamepad é necessário interromper o a thread de eventos"]
    pub fn stop_thread_events(&self) {
        self.event_thread.stop();
    }

    #[doc = "Devolve a 'posse' dos eventos do gamepad dada ao CORE para a thread de eventos. chame isso quando nao houve nenhuma rom em execução"]
    pub fn resume_thread_events(&self) -> Result<(), ErroHandle> {
        self.event_thread.resume(self.manager.clone())
    }

    pub fn apply_rumble(&self, rubble: DeviceRubble) -> Result<(), ErroHandle> {
        self.manager.apply_rumble(rubble);
        Ok(())
    }

    pub fn get_core_cb(&self) -> RetroControllerCb {
        RetroControllerCb {
            manager: self.manager.clone(),
        }
    }
}
pub struct RetroControllerCb {
    manager: Arc<DevicesManager>,
}

impl RetroControllerEnvCallbacks for RetroControllerCb {
    fn input_poll_callback(&self) -> Result<(), ErroHandle> {
        self.manager.update_state()?;
        Ok(())
    }

    fn input_state_callback(
        &self,
        port: i16,
        _device: i16,
        _index: i16,
        id: i16,
    ) -> Result<i16, ErroHandle> {
        Ok(self.manager.get_input_state(port, id))
    }

    fn rumble_callback(
        &self,
        port: std::os::raw::c_uint,
        effect: retro_rumble_effect,
        strength: u16,
    ) -> Result<bool, ErroHandle> {
        Ok(self.manager.apply_rumble(DeviceRubble {
            port: port as usize,
            effect,
            strength,
        }))
    }
}

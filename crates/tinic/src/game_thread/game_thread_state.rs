use crate::channel::ChannelNotify;
use crate::thread_stack::game_stack::GameStackCommand::DeviceConnected;
use crate::thread_stack::main_stack::MainStackCommand::{
    GameLoaded, GameStateSaved, SaveStateLoaded,
};
use generics::erro_handle::ErroHandle;
use generics::retro_paths::RetroPaths;
use libretro_sys::binding_libretro::retro_hw_context_type::RETRO_HW_CONTEXT_OPENGL_CORE;
use libretro_sys::binding_libretro::retro_log_level;
use retro_av::{
    audio_sample_batch_callback, audio_sample_callback, context_destroy, context_reset,
    get_proc_address, video_refresh_callback, EventPump, RetroAv,
};
use retro_controllers::devices_manager::Device;
use retro_controllers::RetroController;
use retro_controllers::{input_poll_callback, input_state_callback, rumble_callback};
use retro_core::graphic_api::GraphicApi;
use retro_core::option_manager::OptionManager;
use retro_core::RetroCore;
use retro_core::RetroCoreIns;
use retro_core::RetroEnvCallbacks;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::{Arc, MutexGuard};

pub struct ThreadState {
    pub channel_notify: ChannelNotify,
    pub pause_request_new_frames: bool,
    pub use_full_screen_mode: bool,
    pub retro_core: Option<RetroCoreIns>,
    pub retro_av: Option<(RetroAv, EventPump)>,
    pub controller_ctx: Arc<Mutex<RetroController>>,
    pub is_running: Arc<Mutex<bool>>,
}

impl ThreadState {
    pub fn is_running(&self) -> bool {
        *self.is_running.lock().unwrap_or_else(|op| {
            let mut can_run = op.into_inner();
            *can_run = false;

            can_run
        })
    }

    fn try_get_retro_core_ctx(&self) -> Result<RetroCoreIns, ErroHandle> {
        match &self.retro_core {
            Some(retro_core) => Ok(retro_core.clone()),
            None => Err(ErroHandle {
                level: retro_log_level::RETRO_LOG_ERROR,
                message: "erro ao tentar recuperar retro_core".to_string(),
            }),
        }
    }

    pub fn try_get_retro_av_ctx(&self) -> Result<&(RetroAv, EventPump), ErroHandle> {
        match &self.retro_av {
            Some(retro_av) => Ok(retro_av),
            None => Err(ErroHandle {
                level: retro_log_level::RETRO_LOG_ERROR,
                message: "erro ao tentar recuperar retro_av".to_string(),
            }),
        }
    }

    fn try_get_controller_ctx(&mut self) -> Result<MutexGuard<'_, RetroController>, ErroHandle> {
        match self.controller_ctx.lock() {
            Ok(guard) => Ok(guard),
            Err(e) => Err(ErroHandle {
                level: retro_log_level::RETRO_LOG_ERROR,
                message: e.to_string(),
            }),
        }
    }

    fn create_retro_contexts(
        &mut self,
        core_path: String,
        rom_path: String,
        paths: RetroPaths,
    ) -> Result<Arc<OptionManager>, ErroHandle> {
        let callbacks = RetroEnvCallbacks {
            input_poll_callback,
            input_state_callback,
            rumble_callback,
            audio_sample_batch_callback,
            audio_sample_callback,
            video_refresh_callback,
            get_proc_address,
            context_destroy,
            context_reset,
        };

        let retro_core = RetroCore::new(
            &core_path,
            paths,
            callbacks,
            GraphicApi::with(RETRO_HW_CONTEXT_OPENGL_CORE),
        )?;
        let av_info = retro_core.load_game(&rom_path)?;
        let retro_av = RetroAv::new(av_info)?;

        let op_manager = retro_core.options.clone();

        self.retro_core.replace(retro_core);
        self.retro_av.replace(retro_av);

        Ok(op_manager)
    }

    pub fn try_render_frame(&mut self) -> Result<(), ErroHandle> {
        if let Some(retro_core) = &self.retro_core {
            if let Some((retro_av, _)) = &mut self.retro_av {
                if !retro_av.sync() || self.pause_request_new_frames {
                    return Ok(());
                }

                // Pede para core gerar novos buffers de video e audio
                retro_core.run()?;
                // Exibe os buffers gerados pelo core
                retro_av.get_new_frame();
            }
        }

        Ok(())
    }

    pub fn load_game(
        &mut self,
        core_path: String,
        rom_path: String,
        paths: RetroPaths,
    ) -> Result<(), ErroHandle> {
        if self.retro_core.is_some() {
            return Ok(());
        }

        match self.create_retro_contexts(core_path, rom_path, paths) {
            Ok(option) => self
                .channel_notify
                .notify_main_stack(GameLoaded(Some(option))),
            Err(e) => {
                self.channel_notify.notify_main_stack(GameLoaded(None));
                return Err(e);
            }
        };

        if let Ok(mut ctr) = self.controller_ctx.lock() {
            ctr.stop_thread_events();

            //Pode ser que essa não seja a primeira vez que um game está sendo
            //executada. Então por garantia o ideal é conectar todos os devices
            //que ja existem agora! E depois os próximos conforme forem chegando.
            for gamepad in ctr.get_list() {
                self.channel_notify
                    .notify_game_stack(DeviceConnected(Device::from_gamepad(&gamepad)))
            }
        }

        Ok(())
    }

    pub fn load_state(&mut self, slot: usize) -> Result<(), ErroHandle> {
        let retro_core = self.try_get_retro_core_ctx()?;

        match retro_core.load_state(slot) {
            Ok(_) => {
                self.channel_notify.notify_main_stack(SaveStateLoaded(true));

                Ok(())
            }
            Err(e) => {
                self.channel_notify
                    .notify_main_stack(SaveStateLoaded(false));

                Err(e)
            }
        }
    }

    pub fn save_state(&mut self, slot: usize) -> Result<(), ErroHandle> {
        let (retro_av, _) = self.try_get_retro_av_ctx()?;
        let retro_core = self.try_get_retro_core_ctx()?;

        match retro_core.save_state(slot) {
            Ok(saved_path) => {
                let mut img_path: PathBuf = PathBuf::new();

                if let Ok(path) = retro_av
                    .video
                    .print_screen(saved_path.parent().unwrap(), &slot.to_string())
                {
                    img_path = path;
                };

                self.channel_notify
                    .notify_main_stack(GameStateSaved(Some((saved_path, img_path))));

                Ok(())
            }

            Err(e) => {
                self.channel_notify.notify_main_stack(GameStateSaved(None));
                Err(e)
            }
        }
    }

    pub fn pause(&mut self) -> Result<(), ErroHandle> {
        self.pause_request_new_frames = true;

        let mut controller = self.try_get_controller_ctx()?;
        controller.resume_thread_events()?;

        Ok(())
    }

    pub fn resume(&mut self) -> Result<(), ErroHandle> {
        self.pause_request_new_frames = false;

        let mut controller = self.try_get_controller_ctx()?;
        controller.stop_thread_events();

        Ok(())
    }

    pub fn reset(&self) -> Result<(), ErroHandle> {
        let retro_core = self.try_get_retro_core_ctx()?;

        retro_core.reset()?;

        Ok(())
    }

    pub fn enable_full_screen(&mut self) -> Result<(), ErroHandle> {
        let (retro_av, _) = self.try_get_retro_av_ctx()?;

        retro_av.video.enable_full_screen();

        Ok(())
    }

    pub fn disable_full_screen(&mut self) -> Result<(), ErroHandle> {
        let (retro_av, _) = self.try_get_retro_av_ctx()?;

        retro_av.video.disable_full_screen();

        Ok(())
    }

    pub fn connect_device(&mut self, device: Device) -> Result<(), ErroHandle> {
        let retro_core = self.try_get_retro_core_ctx()?;

        retro_core.connect_controller(device.retro_port, device.retro_type)?;

        Ok(())
    }
}

impl Drop for ThreadState {
    fn drop(&mut self) {
        self.channel_notify.clear_game_stack();

        //Gracas ao mutex is-running pode ser que algo externo atrapalhe a leitura dos comandos da stack,
        //então so para garantir que essa thread será fechada dando a posse da leitura dos inputs para a
        //thread de inputs novamente, o bom é fazer isso aqui mesmo!
        if let Ok(mut ctr) = self.controller_ctx.lock() {
            let _ = ctr.resume_thread_events();
        }

        if let Some(core) = self.retro_core.take() {
            let _ = core.de_init();
        }

        match self.is_running.lock() {
            Ok(mut is_running) => {
                *is_running = false;
            }
            Err(op) => {
                *op.into_inner() = false;
            }
        }
    }
}

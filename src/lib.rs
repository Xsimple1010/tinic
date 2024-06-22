#[macro_use]
extern crate lazy_static;
extern crate retro_ab;
extern crate retro_ab_av;
extern crate retro_ab_gamepad;

mod game_loop;
mod game_window_handle;
mod retro_stack;
mod stack_handle;

use game_loop::init_game_loop;
use retro_ab::core::{RetroContext, RetroEnvCallbacks};
use retro_ab::retro_sys::retro_rumble_effect;
use retro_ab_av::{audio_sample_batch_callback, audio_sample_callback, video_refresh_callback};
use retro_ab_gamepad::context::{input_poll_callback, input_state_callback, GamepadContext};
use retro_stack::{RetroStack, StackCommand};
use std::sync::{Arc, Mutex};

pub use retro_ab::args_manager;
pub use retro_ab::paths::Paths;
pub use retro_ab::test_tools;
pub use retro_ab_gamepad::retro_gamepad::RetroGamePad;
pub use retro_ab_gamepad::{GamePadState, GamepadStateListener};

static mut GAMEPAD_LISTENER: Option<GamepadStateListener> = None;
lazy_static! {
    static ref STACK: Arc<RetroStack> = RetroStack::new();
}

pub struct Tinic {
    pub controller_ctx: Arc<Mutex<GamepadContext>>,
    pub core_ctx: Option<Arc<RetroContext>>,
}

fn rumble_callback(
    _port: ::std::os::raw::c_uint,
    _effect: retro_rumble_effect,
    _strength: u16,
) -> bool {
    true
}

fn gamepad_state_listener(state: GamePadState, _gamepad: RetroGamePad) {
    unsafe {
        if let Some(listener) = &GAMEPAD_LISTENER {
            match &state {
                GamePadState::Connected | GamePadState::Disconnected => {
                    STACK.push(StackCommand::GamepadConnected(_gamepad.clone()));
                }
                _ => {}
            }
            listener(state, _gamepad);
        };
    }
}

impl Drop for Tinic {
    fn drop(&mut self) {
        STACK.push(StackCommand::Quit);
    }
}

impl Tinic {
    pub fn new(listener: Option<GamepadStateListener>) -> Tinic {
        unsafe {
            GAMEPAD_LISTENER = listener;
        }

        let controller_ctx = Arc::new(Mutex::new(GamepadContext::new(Some(
            gamepad_state_listener,
        ))));

        Self {
            //TODO:o numero máximo de portas deve ser alterado no futuro
            controller_ctx,
            core_ctx: None,
        }
    }

    pub fn load_core(
        &mut self,
        core_path: String,
        rom_path: String,
        paths: Paths,
    ) -> Result<(), String> {
        match self.controller_ctx.clone().lock() {
            Ok(controller) => {
                let gamepads = controller.get_list();
                init_game_loop(gamepads, self.controller_ctx.clone(), STACK.clone());

                STACK.push(StackCommand::LoadGame(
                    core_path.to_owned(),
                    rom_path.to_owned(),
                    paths,
                    RetroEnvCallbacks {
                        audio_sample_batch_callback,
                        audio_sample_callback,
                        input_poll_callback,
                        input_state_callback,
                        video_refresh_callback,
                        rumble_callback,
                    },
                ));
            }
            Err(e) => println!("{:?}", e),
        }

        Ok(())
    }

    pub fn pause(&self) {
        STACK.push(StackCommand::Pause);
    }

    pub fn resume(&self) {
        STACK.push(StackCommand::Resume);
    }

    pub fn save_state(&self) {
        STACK.push(StackCommand::SaveState);
    }

    pub fn load_state(&self) {
        STACK.push(StackCommand::LoadState);
    }

    pub fn reset(&self) {
        STACK.push(StackCommand::Reset);
    }

    pub fn quit(&self) {
        STACK.push(StackCommand::Quit);
    }
}

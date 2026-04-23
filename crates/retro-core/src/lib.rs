extern crate libretro_sys;
extern crate tinic_generics;
extern crate uuid;

mod core_env;
mod managers;
mod retro_core;
mod retro_perf;
mod tools;

pub mod av_info;
pub mod graphic_api;
pub mod pixel;
pub mod system;

pub use core_env::{
    RetroAudioEnvCallbacks, RetroControllerEnvCallbacks, RetroEnvCallbacks, RetroVideoEnvCallbacks,
};
pub use managers::args_manager;
pub use managers::option_manager;
pub use retro_core::{RetroCore, RetroCoreIns};

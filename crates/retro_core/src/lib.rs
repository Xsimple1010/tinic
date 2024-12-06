extern crate libretro_sys;
extern crate uuid;

mod av_info;
mod environment;
mod managers;
mod retro_context;
//arquivo principal!
mod retro_core;
mod retro_perf;
mod tools;

pub mod core;
pub mod graphic_api;
pub mod system;
pub mod test_tools;

pub use managers::args_manager;
pub use managers::option_manager;
pub use retro_core::RetroCore;

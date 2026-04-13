use std::sync::Arc;

use retro_core::av_info::AvInfo;
use tinic_generics::error_handle::TinicResult;

use crate::raw_texture::RawTextureData;

pub enum RetroWindowMode {
    Windowed,
    FullScreen,
}

pub trait RetroWindowContext {
    fn request_redraw(&self);

    fn draw_new_frame(&self, texture: &RawTextureData);

    fn get_proc_address(&self, proc_name: &str) -> *const ();

    fn set_window_mode(&mut self, mode: RetroWindowMode);

    fn toggle_window_model(&mut self);

    fn context_destroy(&mut self) -> TinicResult<()>;

    fn init_context(&mut self) -> TinicResult<()>;

    fn resize(&mut self, width: u32, height: u32);

    fn draw_context_as_initialized(&self) -> bool;

    fn init_frame_buffer(&mut self, av_info: &Arc<AvInfo>) -> TinicResult<()>;

    fn prepare_for_core(&self);
}

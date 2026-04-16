use retro_core::av_info::AvInfo;
use std::sync::Arc;
use tinic_generics::error_handle::TinicResult;

pub enum RetroWindowMode {
    Windowed,
    FullScreen,
}

pub trait RetroWindowContext: 'static {
    fn request_redraw(&self);

    fn draw_new_frame(&self) -> TinicResult<()>;

    fn set_window_mode(&mut self, mode: RetroWindowMode);

    fn toggle_window_model(&mut self);

    fn destroy(&mut self) -> TinicResult<()>;

    fn init_context(&mut self) -> TinicResult<()>;

    fn resize(&mut self, width: u32, height: u32);

    fn draw_context_as_initialized(&self) -> bool;

    fn init_frame_buffer(&mut self, av_info: &Arc<AvInfo>) -> TinicResult<()>;

    fn prepare_for_core(&self);
}

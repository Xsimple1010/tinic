use crate::{print_scree::PrintScree, retro_gl::window::RetroGlWindow};
use generics::{
    erro_handle::ErroHandle,
    types::{ArcTMuxte, TMutex},
};
use libretro_sys::binding_libretro::retro_hw_context_type::{
    RETRO_HW_CONTEXT_NONE, RETRO_HW_CONTEXT_OPENGL, RETRO_HW_CONTEXT_OPENGL_CORE,
};
use retro_core::{
    av_info::{AvInfo, Geometry},
    RetroVideoEnvCallbacks,
};
use std::{
    cell::UnsafeCell,
    ffi::{c_uint, c_void},
    path::{Path, PathBuf},
    ptr::null,
    sync::Arc,
};
use winit::event_loop::ActiveEventLoop;

pub struct RawTextureData {
    pub data: UnsafeCell<*const c_void>,
    pub width: c_uint,
    pub height: c_uint,
    pub pitch: usize,
}

impl RawTextureData {
    pub fn new() -> Self {
        Self {
            data: UnsafeCell::new(null()),
            pitch: 0,
            height: 0,
            width: 0,
        }
    }
}

pub trait RetroVideoAPi {
    fn request_redraw(&self);

    fn draw_new_frame(&self, texture: &RawTextureData, geo: &Geometry);

    #[doc = "define um novo tamanho para a janela.
        ```
        resize((width, height))
        ```
    "]

    fn get_proc_address(&self, proc_name: &str) -> *const ();

    fn full_screen(&mut self);

    fn context_destroy(&mut self);

    fn context_reset(&mut self);
}

pub struct RetroVideo {
    window_ctx: ArcTMuxte<Option<Box<dyn RetroVideoAPi>>>,
    texture: ArcTMuxte<RawTextureData>,
}

impl RetroVideo {
    pub fn new() -> Self {
        Self {
            window_ctx: TMutex::new(None),
            texture: TMutex::new(RawTextureData::new()),
        }
    }

    pub fn init(
        &mut self,
        av_info: &Arc<AvInfo>,
        event_loop: &ActiveEventLoop,
    ) -> Result<(), ErroHandle> {
        match &av_info.video.graphic_api.context_type {
            RETRO_HW_CONTEXT_OPENGL_CORE | RETRO_HW_CONTEXT_OPENGL | RETRO_HW_CONTEXT_NONE => {
                self.window_ctx
                    .try_load()?
                    .replace(Box::new(RetroGlWindow::new(event_loop, av_info)));
                Ok(())
            }
            // RETRO_HW_CONTEXT_VULKAN => {}
            _ => Err(ErroHandle {
                message: "suporte para a api selecionada não está disponível".to_owned(),
            }),
        }
    }

    pub fn destroy_window(&mut self) {
        self.window_ctx.store(None);
        self.texture.store(RawTextureData::new());
    }

    pub fn request_redraw(&self) -> Result<(), ErroHandle> {
        if let Some(win) = &*self.window_ctx.try_load()? {
            win.request_redraw();
        }

        Ok(())
    }

    pub fn draw_new_frame(&self, av_info: &Arc<AvInfo>) -> Result<(), ErroHandle> {
        let texture = &*self.texture.try_load()?;

        if let Some(win) = &*self.window_ctx.try_load()? {
            win.draw_new_frame(texture, &av_info.video.geometry);
        }

        Ok(())
    }

    pub fn print_screen(&self, out_path: &Path, av_info: &Arc<AvInfo>) -> Result<(), ErroHandle> {
        PrintScree::take(
            &*self.texture.try_load()?,
            av_info,
            &mut PathBuf::from(out_path),
        )
    }

    pub fn full_screen(&self) -> Result<(), ErroHandle> {
        Ok(())
    }

    pub fn get_core_cb(&self) -> RetroVideoCb {
        RetroVideoCb {
            texture: self.texture.clone(),
            window_ctx: self.window_ctx.clone(),
        }
    }
}

pub struct RetroVideoCb {
    texture: ArcTMuxte<RawTextureData>,
    window_ctx: ArcTMuxte<Option<Box<dyn RetroVideoAPi>>>,
}

impl RetroVideoEnvCallbacks for RetroVideoCb {
    fn video_refresh_callback(
        &self,
        data: *const c_void,
        width: u32,
        height: u32,
        pitch: usize,
    ) -> Result<(), ErroHandle> {
        let mut texture = self.texture.try_load()?;
        let tex_data = texture.data.get_mut();

        *tex_data = data;
        texture.width = width;
        texture.height = height;
        texture.pitch = pitch;

        Ok(())
    }

    fn get_proc_address(&self, proc_name: &str) -> Result<*const (), ErroHandle> {
        if let Some(win) = &mut *self.window_ctx.try_load()? {
            win.get_proc_address(proc_name);
        }

        Ok(null())
    }

    fn context_destroy(&self) -> Result<(), ErroHandle> {
        if let Some(win) = &mut *self.window_ctx.try_load()? {
            win.context_destroy();
        }
        Ok(())
    }

    fn context_reset(&self) -> Result<(), ErroHandle> {
        if let Some(win) = &mut *self.window_ctx.try_load()? {
            win.context_reset();
        }
        Ok(())
    }
}

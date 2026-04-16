use std::sync::Arc;

use retro_core::RetroVideoEnvCallbacks;
use tinic_generics::{error_handle::TinicResult, types::ArcTMutex};

use crate::{raw_texture::RawTextureData, retro_gl::proc_resolver::GlProcResolver};

pub struct RetroVideoCb {
    texture: ArcTMutex<RawTextureData>,
    proc_resolve: Arc<GlProcResolver>,
}

impl RetroVideoCb {
    pub fn new(texture: ArcTMutex<RawTextureData>, proc_resolve: Arc<GlProcResolver>) -> Self {
        Self {
            texture,
            proc_resolve,
        }
    }
}

impl RetroVideoEnvCallbacks for RetroVideoCb {
    fn video_refresh_callback(
        &self,
        data: Vec<u8>,
        width: u32,
        height: u32,
        pitch: usize,
        is_hw: bool,
    ) -> TinicResult<()> {
        let mut texture = self.texture.try_load()?;

        texture.data = data;
        texture.width = width;
        texture.height = height;
        texture.pitch = pitch;
        texture.is_hw = is_hw;

        Ok(())
    }

    fn get_proc_address(&self, proc_name: &str) -> *const () {
        self.proc_resolve.get_proc(proc_name)
    }
}

use super::gl::gl::{self, types::GLuint};
use retro_core::pixel::PixelFormat;
use std::mem::size_of;
use tinic_generics::error_handle::ErrorHandle;

pub struct Pixel {
    pub format: GLuint,
    pub typ: GLuint,
    pub bpm: i32,
}

impl Pixel {
    pub fn new(retro_pixel: &PixelFormat) -> Result<Pixel, ErrorHandle> {
        match retro_pixel {
            PixelFormat::Xrgb8888 => Ok(Pixel {
                format: gl::UNSIGNED_INT_8_8_8_8_REV,
                typ: gl::BGRA,
                bpm: size_of::<u32>() as i32,
            }),
            PixelFormat::Rgb1555 => Ok(Pixel {
                format: gl::UNSIGNED_SHORT_5_5_5_1,
                typ: gl::BGRA,
                bpm: size_of::<u16>() as i32,
            }),
            PixelFormat::Rgb565 => Ok(Pixel {
                format: gl::UNSIGNED_SHORT_5_6_5,
                typ: gl::RGB,
                bpm: size_of::<u16>() as i32,
            }),
            _ => Err(ErrorHandle {
                message: "Formato de pixel desconhecido".to_string(),
            }),
        }
    }
}

use libretro_sys::binding_libretro::retro_pixel_format;

/// Identifies the pixel format used by a libretro core to deliver video frames.
///
/// This is the public-facing equivalent of [`retro_pixel_format`] from the
/// libretro C API. Use [`From`] / [`Into`] to convert between the two.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[repr(u32)]
pub enum PixelFormat {
    /// 0RGB1555, native endian.
    ///
    /// The 0 bit must be set to 0.
    /// This pixel format is default for compatibility concerns only.
    /// If a 15/16-bit pixel format is desired, consider using [`PixelFormat::Rgb565`].
    Rgb1555 = 0,

    /// XRGB8888, native endian.
    ///
    /// X bits are ignored.
    Xrgb8888 = 1,

    /// RGB565, native endian.
    ///
    /// This is the recommended format when a 15/16-bit format is desired, as it
    /// is typically available on a wide range of low-power devices and is
    /// natively supported in APIs like OpenGL ES.
    Rgb565 = 2,

    Unknown = i32::MAX as u32,
}

impl From<retro_pixel_format> for PixelFormat {
    fn from(v: retro_pixel_format) -> Self {
        // SAFETY: ambas são #[repr(u32)] com discriminantes idênticos.
        unsafe { std::mem::transmute(v) }
    }
}

impl From<PixelFormat> for retro_pixel_format {
    fn from(v: PixelFormat) -> Self {
        // SAFETY: ambas são #[repr(u32)] com discriminantes idênticos.
        unsafe { std::mem::transmute(v) }
    }
}

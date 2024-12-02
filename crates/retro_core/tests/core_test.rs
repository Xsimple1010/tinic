use generics::erro_handle::ErroHandle;
use retro_core::core::{retro_language, retro_pixel_format};

mod common;

#[test]
fn core_implement_tests() -> Result<(), ErroHandle> {
    let ctx = common::core::setup()?;

    assert_eq!(
        *ctx.core().language.lock().unwrap(),
        retro_language::RETRO_LANGUAGE_ENGLISH
    );

    assert_eq!(
        *ctx.core().av_info.video.pixel_format.lock().unwrap(),
        retro_pixel_format::RETRO_PIXEL_FORMAT_UNKNOWN
    );

    return Ok(());
}

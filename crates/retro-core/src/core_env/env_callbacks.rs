#[cfg(feature = "hw")]
use std::{ffi::c_char, mem};
use std::{
    ffi::{c_uint, c_void},
    ptr::addr_of,
};

#[cfg(feature = "hw")]
use libretro_sys::binding_libretro::retro_proc_address_t;
use libretro_sys::binding_libretro::retro_rumble_effect;

use crate::core_env::CORE_CONTEXT;

pub unsafe extern "C" fn rumble_callback(
    port: c_uint,
    effect: retro_rumble_effect,
    strength: u16,
) -> bool {
    let retro_core = unsafe {
        match &*addr_of!(CORE_CONTEXT) {
            Some(core_ctx) => core_ctx,
            None => return false,
        }
    };

    let res = retro_core
        .callbacks
        .controller
        .rumble_callback(port, effect, strength);

    match res {
        Ok(v) => v,
        Err(e) => {
            println!("{:?}", e);
            let _ = retro_core.de_init();
            false
        }
    }
}

pub unsafe extern "C" fn input_poll_callback() {
    let retro_core = unsafe {
        match &*addr_of!(CORE_CONTEXT) {
            Some(core_ctx) => core_ctx,
            None => return,
        }
    };

    if let Err(e) = retro_core.callbacks.controller.input_poll_callback() {
        println!("{:?}", e);
        let _ = retro_core.de_init();
    }
}

pub unsafe extern "C" fn input_state_callback(
    port: c_uint,
    device: c_uint,
    index: c_uint,
    id: c_uint,
) -> i16 {
    let retro_core = unsafe {
        match &*addr_of!(CORE_CONTEXT) {
            Some(core_ctx) => core_ctx,
            None => return 0,
        }
    };

    let res = retro_core.callbacks.controller.input_state_callback(
        port as i16,
        device as i16,
        index as i16,
        id as i16,
    );

    match res {
        Ok(v) => v,
        Err(e) => {
            println!("{:?}", e);
            let _ = retro_core.de_init();
            0
        }
    }
}

pub unsafe extern "C" fn audio_sample_callback(left: i16, right: i16) {
    unsafe {
        if let Some(core_ctx) = &*addr_of!(CORE_CONTEXT)
            && let Err(e) = core_ctx.callbacks.audio.audio_sample_callback(
                left,
                right,
                core_ctx.av_info.clone(),
            )
        {
            println!("{:?}", e);
            let _ = core_ctx.de_init();
        }
    }
}

pub unsafe extern "C" fn audio_sample_batch_callback(data: *const i16, frames: usize) -> usize {
    unsafe {
        if let Some(core_ctx) = &*addr_of!(CORE_CONTEXT) {
            let res = core_ctx.callbacks.audio.audio_sample_batch_callback(
                data,
                frames,
                core_ctx.av_info.clone(),
            );

            match res {
                Ok(frames) => frames,
                Err(e) => {
                    println!("{:?}", e);
                    let _ = core_ctx.de_init();
                    0
                }
            }
        } else {
            0
        }
    }
}

pub unsafe extern "C" fn video_refresh_callback(
    data: *const c_void,
    width: u32,
    height: u32,
    pitch: usize,
) {
    unsafe {
        const HW_FLAG: *const c_void = usize::MAX as *const c_void;

        let is_hw = data == HW_FLAG;

        let buffer = if is_hw {
            Vec::new()
        } else {
            if data.is_null() || width == 0 || height == 0 || pitch == 0 {
                return;
            }

            let size = match pitch.checked_mul(height as usize) {
                Some(s) => s,
                None => return,
            };

            std::slice::from_raw_parts(data as *const u8, size).to_vec()
        };

        let core_ctx = &*addr_of!(CORE_CONTEXT);

        if let Some(core_ctx) = core_ctx.as_ref() {
            let res = core_ctx
                .callbacks
                .video
                .video_refresh_callback(buffer, width, height, pitch, is_hw);

            if let Err(e) = res {
                println!("{e:?}");
                let _ = core_ctx.de_init();
            }
        }
    }
}

#[cfg(feature = "hw")]
pub unsafe extern "C" fn get_current_frame_buffer() -> usize {
    unsafe {
        match &*addr_of!(CORE_CONTEXT) {
            Some(core_ctx) => core_ctx
                .av_info
                .video
                .graphic_api
                .fbo
                .read()
                .unwrap()
                .unwrap(),
            None => 0,
        }
    }
}

//TODO: ainda preciso testar  se isso esta funcionando
#[cfg(feature = "hw")]
pub unsafe extern "C" fn get_proc_address(sym: *const c_char) -> retro_proc_address_t {
    use crate::tools::ffi_tools::get_str_from_ptr;

    unsafe {
        match &*addr_of!(CORE_CONTEXT) {
            Some(core_ctx) => {
                let fc_name = get_str_from_ptr(sym);

                let proc_address = core_ctx.callbacks.video.get_proc_address(&fc_name);
                if proc_address.is_null() {
                    return None;
                }

                let function: unsafe extern "C" fn() = mem::transmute(proc_address);

                Some(function)
            }
            None => None,
        }
    }
}

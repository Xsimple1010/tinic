#[cfg(feature = "hw")]
use crate::libretro_sys::{
    binding_libretro::{
        retro_hw_context_type, retro_hw_render_callback, retro_proc_address_t,
        RETRO_ENVIRONMENT_EXPERIMENTAL, RETRO_ENVIRONMENT_GET_PREFERRED_HW_RENDER,
        RETRO_ENVIRONMENT_SET_HW_RENDER,
    },
    binding_log_interface,
};
use crate::{
    libretro_sys::binding_libretro::{
        retro_game_geometry, retro_pixel_format, RETRO_ENVIRONMENT_GET_AUDIO_VIDEO_ENABLE,
        RETRO_ENVIRONMENT_SET_GEOMETRY, RETRO_ENVIRONMENT_SET_PIXEL_FORMAT,
    },
    RetroCoreIns,
};
#[cfg(feature = "hw")]
use std::{ffi::c_char, mem};
use std::{
    ffi::{c_uint, c_void},
    ptr::addr_of,
};

use super::environment::CORE_CONTEXT;

pub unsafe extern "C" fn audio_sample_callback(left: i16, right: i16) {
    if let Some(core_ctx) = &*addr_of!(CORE_CONTEXT) {
        if let Err(e) = core_ctx.callbacks.audio.audio_sample_callback(left, right) {
            println!("{:?}", e);
            let _ = core_ctx.de_init();
        }
    }
}

pub unsafe extern "C" fn audio_sample_batch_callback(data: *const i16, frames: usize) -> usize {
    if let Some(core_ctx) = &*addr_of!(CORE_CONTEXT) {
        let res = core_ctx
            .callbacks
            .audio
            .audio_sample_batch_callback(data, frames);

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

pub unsafe extern "C" fn video_refresh_callback(
    data: *const c_void,
    width: c_uint,
    height: c_uint,
    pitch: usize,
) {
    if let Some(core_ctx) = &*addr_of!(CORE_CONTEXT) {
        if let Err(e) = core_ctx
            .callbacks
            .video
            .video_refresh_callback(data, width, height, pitch)
        {
            println!("{:?}", e);
            let _ = core_ctx.de_init();
        }
    }
}

#[cfg(feature = "hw")]
unsafe extern "C" fn get_current_frame_buffer() -> usize {
    println!("get_current_frame_buffer");
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

//TODO: ainda preciso testar  se isso esta funcionando
#[cfg(feature = "hw")]
unsafe extern "C" fn get_proc_address(sym: *const c_char) -> retro_proc_address_t {
    use crate::tools::ffi_tools::get_str_from_ptr;

    println!("get_proc_address");
    match &*addr_of!(CORE_CONTEXT) {
        Some(core_ctx) => {
            let fc_name = get_str_from_ptr(sym);

            let res = core_ctx.callbacks.video.get_proc_address(&fc_name);

            match res {
                Ok(proc_address) => {
                    if proc_address.is_null() {
                        return None;
                    }

                    let function: unsafe extern "C" fn() = unsafe { mem::transmute(proc_address) };

                    Some(function)
                }
                Err(e) => {
                    println!("{:?}", e);
                    let _ = core_ctx.de_init();
                    None
                }
            }
        }
        None => None,
    }
}

#[cfg(feature = "hw")]
unsafe extern "C" fn context_reset() {
    println!("context_reset");
    if let Some(core_ctx) = &*addr_of!(CORE_CONTEXT) {
        if let Err(e) = core_ctx.callbacks.video.context_reset() {
            println!("{:?}", e);
            let _ = core_ctx.de_init();
        }
    }
}

#[cfg(feature = "hw")]
unsafe extern "C" fn context_destroy() {
    println!("context_destroy");

    if let Some(core_ctx) = &*addr_of!(CORE_CONTEXT) {
        if let Err(e) = core_ctx.callbacks.video.context_destroy() {
            println!("{:?}", e);
            let _ = core_ctx.de_init();
        }
    }
}

pub unsafe fn env_cb_av(core_ctx: &RetroCoreIns, cmd: c_uint, data: *mut c_void) -> bool {
    match cmd {
        RETRO_ENVIRONMENT_SET_GEOMETRY => {
            #[cfg(feature = "core_ev_logs")]
            println!("RETRO_ENVIRONMENT_SET_GEOMETRY -> ok");

            let raw_geometry_ptr = data as *mut retro_game_geometry;

            if raw_geometry_ptr.is_null() {
                return false;
            }

            let _ = core_ctx.av_info.try_set_new_geometry(raw_geometry_ptr);

            true
        }
        RETRO_ENVIRONMENT_SET_PIXEL_FORMAT => {
            #[cfg(feature = "core_ev_logs")]
            println!("RETRO_ENVIRONMENT_SET_PIXEL_FORMAT -> ok");

            *core_ctx.av_info.video.pixel_format.write().unwrap() =
                *(data as *mut retro_pixel_format);

            true
        }
        RETRO_ENVIRONMENT_GET_AUDIO_VIDEO_ENABLE => {
            #[cfg(feature = "core_ev_logs")]
            println!("RETRO_ENVIRONMENT_GET_AUDIO_VIDEO_ENABLE -> ok");

            *(data as *mut u32) = 1 << 0 | 1 << 1;

            true
        }
        #[cfg(feature = "hw")]
        RETRO_ENVIRONMENT_GET_PREFERRED_HW_RENDER => {
            #[cfg(feature = "core_ev_logs")]
            println!("RETRO_ENVIRONMENT_GET_PREFERRED_HW_RENDER");

            *(data as *mut retro_hw_context_type) = core_ctx.av_info.video.graphic_api.context_type;

            true
        }
        #[cfg(feature = "hw")]
        RETRO_ENVIRONMENT_SET_HW_RENDER | RETRO_ENVIRONMENT_EXPERIMENTAL => {
            #[cfg(feature = "core_ev_logs")]
            println!("RETRO_ENVIRONMENT_SET_HW_RENDER");

            if data.is_null() {
                return false;
            }

            binding_log_interface::set_hw_callback(
                data,
                Some(context_reset),
                Some(get_current_frame_buffer),
                Some(context_destroy),
                Some(get_proc_address),
            );

            core_ctx
                .av_info
                .video
                .graphic_api
                .try_update_from_raw(data as *mut retro_hw_render_callback)
        }

        _ => false,
    }
}

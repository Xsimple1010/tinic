use crate::{
    RetroCoreIns,
    core_env::env_callbacks::rumble_callback,
    libretro_sys::binding_libretro::{
        RETRO_ENVIRONMENT_GET_INPUT_BITMASKS, RETRO_ENVIRONMENT_GET_RUMBLE_INTERFACE,
        RETRO_ENVIRONMENT_SET_CONTROLLER_INFO, RETRO_ENVIRONMENT_SET_INPUT_DESCRIPTORS,
        retro_controller_info, retro_rumble_interface,
    },
    tinic_generics::constants::MAX_CORE_CONTROLLER_INFO_TYPES,
    tools::validation::InputValidator,
};
use std::{ffi::c_uint, os::raw::c_void};
use tinic_generics::error_handle::ErrorHandle;

pub unsafe fn env_cb_gamepad_io(
    core_ctx: &RetroCoreIns,
    cmd: c_uint,
    data: *mut c_void,
) -> Result<bool, ErrorHandle> {
    match cmd {
        RETRO_ENVIRONMENT_GET_INPUT_BITMASKS => {
            #[cfg(feature = "core_ev_logs")]
            println!("RETRO_ENVIRONMENT_GET_INPUT_BITMASKS -> ok");
            Ok(true)
        }
        RETRO_ENVIRONMENT_SET_CONTROLLER_INFO => {
            #[cfg(feature = "core_ev_logs")]
            println!("RETRO_ENVIRONMENT_SET_CONTROLLER_INFO -> ok");

            InputValidator::validate_non_null_mut_ptr(
                data,
                "data in RETRO_ENVIRONMENT_SET_CONTROLLER_INFO",
            )?;

            let raw_ctr_infos =
                unsafe { *(data as *mut [retro_controller_info; MAX_CORE_CONTROLLER_INFO_TYPES]) };

            let _ = core_ctx.system.get_ports(raw_ctr_infos);

            Ok(true)
        }
        RETRO_ENVIRONMENT_SET_INPUT_DESCRIPTORS => {
            #[cfg(feature = "core_ev_logs")]
            println!("RETRO_ENVIRONMENT_SET_INPUT_DESCRIPTORS -> ok");
            Ok(false)
        }
        RETRO_ENVIRONMENT_GET_RUMBLE_INTERFACE => {
            #[cfg(feature = "core_ev_logs")]
            println!("RETRO_ENVIRONMENT_GET_RUMBLE_INTERFACE -> ok");

            InputValidator::validate_non_null_mut_ptr(
                data,
                "data in RETRO_ENVIRONMENT_GET_RUMBLE_INTERFACE",
            )?;

            let rumble_raw = unsafe { &mut *(data as *mut retro_rumble_interface) };
            rumble_raw.set_rumble_state = Some(rumble_callback);

            Ok(true)
        }

        _ => Ok(false),
    }
}

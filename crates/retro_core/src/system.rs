use crate::tools::ffi_tools::get_str_from_ptr;
use generics::constants::{
    MAX_CORE_CONTROLLER_INFO_TYPES, MAX_CORE_SUBSYSTEM_INFO, MAX_CORE_SUBSYSTEM_ROM_INFO,
};
use libretro_sys::binding_libretro::{
    retro_controller_description, retro_controller_info, retro_subsystem_info,
    retro_subsystem_memory_info, retro_subsystem_rom_info, retro_system_info, LibretroRaw,
};
use std::sync::{atomic::AtomicU8, Arc, RwLock};

#[derive(Default, Debug)]
pub struct SysInfo {
    pub library_name: Arc<String>,
    pub library_version: Arc<String>,
    pub valid_extensions: Arc<String>,
    pub need_full_path: Arc<bool>,
    pub block_extract: Arc<bool>,
}

#[derive(Default, Debug)]
pub struct MemoryInfo {
    pub extension: Arc<String>,
    pub type_: Arc<u32>,
}

#[derive(Default, Debug)]
pub struct ControllerDescription {
    pub desc: Arc<String>,
    pub id: Arc<u32>,
}

#[derive(Default, Debug)]
pub struct SubSystemRomInfo {
    pub desc: Arc<String>,
    pub valid_extensions: Arc<String>,
    pub need_full_path: Arc<bool>,
    pub block_extract: Arc<bool>,
    pub required: Arc<bool>,
    pub memory: MemoryInfo,
    pub num_memory: Arc<u32>,
}

#[derive(Default, Debug)]
pub struct SubSystemInfo {
    pub id: Arc<u32>,
    pub desc: Arc<String>,
    pub ident: Arc<String>,
    pub roms: RwLock<Vec<SubSystemRomInfo>>,
}

#[derive(Debug)]
pub struct System {
    pub info: SysInfo,
    pub ports: RwLock<Vec<ControllerDescription>>,
    pub subsystem: RwLock<Vec<SubSystemInfo>>,
    pub performance_level: AtomicU8,
}

impl System {
    pub fn new(raw: &LibretroRaw) -> Self {
        unsafe {
            let sys_info = &mut retro_system_info {
                block_extract: false,
                need_fullpath: false,
                library_name: "".as_ptr() as *const i8,
                library_version: "".as_ptr() as *const i8,
                valid_extensions: "".as_ptr() as *const i8,
            };

            raw.retro_get_system_info(sys_info);

            System {
                ports: RwLock::new(Vec::new()),
                subsystem: RwLock::new(Vec::new()),
                performance_level: AtomicU8::new(0),
                info: SysInfo {
                    library_name: Arc::new(get_str_from_ptr(sys_info.library_name)),
                    library_version: Arc::new(get_str_from_ptr(sys_info.library_version)),
                    valid_extensions: Arc::new(get_str_from_ptr(sys_info.valid_extensions)),
                    need_full_path: Arc::new(sys_info.need_fullpath),
                    block_extract: Arc::new(sys_info.block_extract),
                },
            }
        }
    }

    pub fn get_subsystem(&self, raw_subsystem: [retro_subsystem_info; MAX_CORE_SUBSYSTEM_INFO]) {
        self.subsystem.write().unwrap().clear();

        for raw_sys in raw_subsystem {
            if raw_sys.ident.is_null() {
                break;
            }

            let mut roms: Vec<SubSystemRomInfo> = Vec::new();

            let raw_roms = unsafe {
                *(raw_sys.roms as *mut [retro_subsystem_rom_info; MAX_CORE_SUBSYSTEM_ROM_INFO])
            };

            for rom in raw_roms.iter().take(raw_sys.num_roms as usize) {
                let memory = unsafe { *(rom.memory as *mut retro_subsystem_memory_info) };

                roms.push(SubSystemRomInfo {
                    desc: Arc::new(get_str_from_ptr(rom.desc)),
                    valid_extensions: Arc::new(get_str_from_ptr(rom.valid_extensions)),
                    need_full_path: Arc::new(rom.need_fullpath),
                    block_extract: Arc::new(rom.block_extract),
                    required: Arc::new(rom.required),
                    num_memory: Arc::new(rom.num_memory),
                    memory: MemoryInfo {
                        extension: Arc::new(get_str_from_ptr(memory.extension)),
                        type_: Arc::new(memory.type_),
                    },
                });
            }

            let subsystem = SubSystemInfo {
                id: Arc::new(raw_sys.id),
                desc: Arc::new(get_str_from_ptr(raw_sys.desc)),
                ident: Arc::new(get_str_from_ptr(raw_sys.ident)),
                roms: RwLock::new(roms),
            };

            self.subsystem.write().unwrap().push(subsystem);
        }
    }

    pub fn get_ports(
        &self,
        raw_ctr_infos: [retro_controller_info; MAX_CORE_CONTROLLER_INFO_TYPES],
    ) {
        self.ports.write().unwrap().clear();

        for raw_ctr_info in raw_ctr_infos {
            if raw_ctr_info.types.is_null() {
                break;
            }

            let raw_ctr_types = unsafe {
                *(raw_ctr_info.types
                    as *mut [retro_controller_description; MAX_CORE_CONTROLLER_INFO_TYPES])
            };

            for ctr_type in raw_ctr_types.iter().take(raw_ctr_info.num_types as usize) {
                if ctr_type.desc.is_null() {
                    return;
                }

                let controller_description = ControllerDescription {
                    desc: Arc::new(get_str_from_ptr(ctr_type.desc)),
                    id: Arc::new(ctr_type.id),
                };

                self.ports.write().unwrap().push(controller_description);
            }
        }
    }
}

#[cfg(test)]
mod test_system {
    use crate::{system::System, test_tools};

    #[test]
    fn test_get_sys_info() {
        let core = test_tools::core::get_core_wrapper();

        let sys = System::new(&core.raw);

        assert_eq!(*sys.info.library_name, "Snes9x".to_owned());

        assert_eq!(*sys.info.library_version, "1.62.3 46f8a6b".to_owned());

        assert_eq!(
            *sys.info.valid_extensions,
            "smc|sfc|swc|fig|bs|st".to_owned()
        );

        assert_eq!(*sys.info.block_extract, false);

        assert_eq!(*sys.info.need_full_path, false);
    }
}

#[derive(Debug, Default)]
pub struct CoreInfo {
    // Informações de Software
    pub display_name: String,
    pub authors: String,
    pub supported_extensions: String,
    pub core_name: String,
    pub license: String,
    pub permissions: String,
    pub display_version: String,
    pub categories: String,
    pub description: String,

    // Informações de Hardware
    pub manufacturer: String,
    pub system_name: String,
    pub system_id: String,

    // // Recursos do Libretro
    pub save_state: bool,
    pub save_state_features: String,
    pub cheats: bool,
    pub input_descriptors: bool,
    pub memory_descriptors: bool,
    pub libretro_saves: bool,
    pub core_options: bool,
    pub core_options_version: String,
    pub supports_no_game: bool,
    pub database: String,
    pub hw_render: bool,
    pub needs_full_path: bool,
    pub disk_control: bool,
    pub load_subsystem: bool,
    pub required_hw_api: String,
    pub is_experimental: bool,
    pub needs_fullpath: bool,
}

impl CoreInfo {
    fn get_boolean_value(&self, value: &String) -> bool {
        value == "true"
    }

    pub fn set_value(&mut self, key: &str, value: String) {
        match key {
            // # Software Information
            "display_name" => self.display_name = value,
            "authors" => self.authors = value,
            "supported_extensions" => self.supported_extensions = value,
            "corename" => self.core_name = value,
            "license" => self.license = value,
            "permissions" => self.permissions = value,
            "display_version" => self.display_version = value,
            "categories" => self.categories = value,
            "description" => self.description = value,

            // # Hardware Information
            "manufacturer" => self.manufacturer = value,
            "systemname" => self.system_name = value,
            "systemid" => self.system_id = value,

            // # Libretro Features
            "savestate" => self.save_state = self.get_boolean_value(&value),
            "savestate_features" => self.save_state_features = value,
            "cheats" => self.cheats = self.get_boolean_value(&value),
            "input_descriptors" => self.input_descriptors = self.get_boolean_value(&value),
            "memory_descriptors" => self.memory_descriptors = self.get_boolean_value(&value),
            "libretro_saves" => self.libretro_saves = self.get_boolean_value(&value),
            "core_options" => self.core_options = self.get_boolean_value(&value),
            "core_options_version" => self.core_options_version = value,
            "load_subsystem" => self.load_subsystem = self.get_boolean_value(&value),
            "supports_no_game" => self.supports_no_game = self.get_boolean_value(&value),
            "database" => self.database = value,
            "hw_render" => self.hw_render = self.get_boolean_value(&value),
            "needs_fullpath" => self.needs_fullpath = self.get_boolean_value(&value),
            "disk_control" => self.disk_control = self.get_boolean_value(&value),
            _ => {}
        }
    }
}

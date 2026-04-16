use std::ffi::c_uint;

#[derive(Clone)]
pub struct RawTextureData {
    pub data: Vec<u8>,
    pub width: c_uint,
    pub height: c_uint,
    pub pitch: usize,
    pub is_hw: bool,
}

impl RawTextureData {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            pitch: 0,
            height: 0,
            width: 0,
            is_hw: false,
        }
    }
}

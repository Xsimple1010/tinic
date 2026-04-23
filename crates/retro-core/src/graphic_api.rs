use libretro_sys::binding_libretro::{
    retro_hw_context_reset_t, retro_hw_context_type, retro_hw_render_callback,
};
use std::sync::Arc;
use std::sync::atomic::AtomicU8;
use std::sync::{
    RwLock,
    atomic::{AtomicBool, Ordering},
};
use tinic_generics::error_handle::TinicResult;

/// Identifies the graphics API requested by a libretro core.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[repr(u32)]
pub enum HwContextType {
    None = 0,
    /// OpenGL 2.x. Driver can choose to use latest compatibility context.
    OpenGl = 1,
    /// OpenGL ES 2.0.
    OpenGlEs2 = 2,
    /// Modern desktop core GL context. Use `version_major`/`version_minor`
    /// fields to set GL version.
    OpenGlCore = 3,
    /// OpenGL ES 3.0.
    OpenGlEs3 = 4,
    /// OpenGL ES 3.1+. Set `version_major`/`version_minor`. For GLES2 and
    /// GLES3, use the corresponding variants directly.
    OpenGlEsVersion = 5,
    /// Vulkan. See `RETRO_ENVIRONMENT_GET_HW_RENDER_INTERFACE`.
    Vulkan = 6,
    /// Direct3D 11. See `RETRO_ENVIRONMENT_GET_HW_RENDER_INTERFACE`.
    D3D11 = 7,
    /// Direct3D 10. See `RETRO_ENVIRONMENT_GET_HW_RENDER_INTERFACE`.
    D3D10 = 8,
    /// Direct3D 12. See `RETRO_ENVIRONMENT_GET_HW_RENDER_INTERFACE`.
    D3D12 = 9,
    /// Direct3D 9. See `RETRO_ENVIRONMENT_GET_HW_RENDER_INTERFACE`.
    D3D9 = 10,
    #[doc(hidden)]
    Dummy = i32::MAX as u32,
}

impl From<retro_hw_context_type> for HwContextType {
    fn from(v: retro_hw_context_type) -> Self {
        unsafe { std::mem::transmute(v) }
    }
}

impl From<HwContextType> for retro_hw_context_type {
    fn from(v: HwContextType) -> Self {
        unsafe { std::mem::transmute(v) }
    }
}

/// Holds the hardware render state negotiated between a libretro core and the
/// frontend.
///
/// Populated via [`GraphicApi::try_update_from_raw`] when the core sets the
/// `RETRO_ENVIRONMENT_SET_HW_RENDER` environment key. All fields are
/// internally synchronized so the struct can be shared across threads.
#[derive(Debug)]
pub struct GraphicApi {
    /// Which graphics API the core requested.
    pub context_type: RwLock<HwContextType>,

    /// Opaque handle to the frontend-allocated framebuffer object, if any.
    ///
    /// TODO: Obsolete — the frontend should not be providing pre-allocated
    /// framebuffers.
    pub fbo: RwLock<Option<usize>>,

    /// Whether render buffers should have a depth component attached.
    ///
    /// TODO: Obsolete.
    pub depth: AtomicBool,

    /// Whether render buffers should have a stencil component attached.
    ///
    /// TODO: Obsolete.
    pub stencil: AtomicBool,

    /// Whether to use the conventional bottom-left origin.
    ///
    /// If `false`, the standard libretro top-left origin semantics are used.
    pub bottom_left_origin: AtomicBool,

    /// Major version number for core GL context or GLES 3.1+.
    pub major: AtomicU8,

    /// Minor version number for core GL context or GLES 3.1+.
    pub minor: AtomicU8,

    /// If `true`, the frontend will avoid resetting the context in scenarios
    /// such as toggling fullscreen.
    pub cache_context: AtomicBool,

    /// Whether to create a debug context.
    pub debug_context: AtomicBool,

    context_reset: Arc<RwLock<Option<retro_hw_context_reset_t>>>,
    context_destroy: Arc<RwLock<Option<retro_hw_context_reset_t>>>,
}

impl Default for GraphicApi {
    fn default() -> Self {
        GraphicApi {
            context_type: RwLock::new(HwContextType::OpenGl),
            fbo: RwLock::new(None),
            depth: AtomicBool::new(false),
            stencil: AtomicBool::new(false),
            bottom_left_origin: AtomicBool::new(false),
            major: AtomicU8::new(0),
            minor: AtomicU8::new(0),
            cache_context: AtomicBool::new(false),
            debug_context: AtomicBool::new(false),
            context_reset: Arc::new(RwLock::new(None)),
            context_destroy: Arc::new(RwLock::new(None)),
        }
    }
}

impl GraphicApi {
    /// Creates a [`GraphicApi`] configured for OpenGL Core Profile.
    pub fn with_opengl_core() -> Self {
        Self {
            context_type: RwLock::new(HwContextType::OpenGlCore),
            ..Default::default()
        }
    }

    /// Creates a [`GraphicApi`] configured for OpenGL compatibility.
    pub fn with_opengl() -> Self {
        Self {
            context_type: RwLock::new(HwContextType::OpenGl),
            ..Default::default()
        }
    }

    pub fn with_vulkan() -> Self {
        Self {
            context_type: RwLock::new(HwContextType::Vulkan),
            ..Default::default()
        }
    }

    /// Invokes the core's context-reset callback, if one has been registered.
    pub fn try_reset_ctx(&self) -> TinicResult<()> {
        let context_reset_fn = self.context_reset.read()?.clone();

        if let Some(Some(f)) = context_reset_fn {
            unsafe { f() }
        }

        Ok(())
    }

    /// Invokes the core's context-destroy callback, if one has been registered.
    pub fn try_destroy_ctx(&self) -> TinicResult<()> {
        let context_destroy = self.context_destroy.read()?.clone();

        if let Some(Some(f)) = context_destroy {
            unsafe { f() }
        }

        Ok(())
    }

    /// Populates this [`GraphicApi`] from a raw [`retro_hw_render_callback`]
    /// provided by the core.
    ///
    /// Stores the context-reset and context-destroy callbacks so the frontend
    /// can call them at the appropriate time.
    pub(crate) fn try_update_from_raw(
        &self,
        hw_cb: &retro_hw_render_callback,
    ) -> TinicResult<bool> {
        self.context_reset.write()?.replace(hw_cb.context_reset);
        self.context_destroy.write()?.replace(hw_cb.context_destroy);

        *self.context_type.write()? = HwContextType::from(hw_cb.context_type);

        self.depth.store(hw_cb.depth, Ordering::SeqCst);
        self.stencil.store(hw_cb.stencil, Ordering::SeqCst);
        self.bottom_left_origin
            .store(hw_cb.bottom_left_origin, Ordering::SeqCst);
        self.minor
            .store(hw_cb.version_minor as u8, Ordering::SeqCst);
        self.major
            .store(hw_cb.version_major as u8, Ordering::SeqCst);
        self.cache_context
            .store(hw_cb.cache_context, Ordering::SeqCst);
        self.debug_context
            .store(hw_cb.debug_context, Ordering::SeqCst);

        Ok(true)
    }

    /// Resets all fields to their default values and releases stored callbacks.
    pub fn clear(&self) -> TinicResult<()> {
        self.context_reset.write()?.take();
        self.context_destroy.write()?.take();
        self.fbo.write()?.take();

        *self.context_type.write()? = HwContextType::OpenGl;

        self.depth.store(false, Ordering::SeqCst);
        self.stencil.store(false, Ordering::SeqCst);
        self.bottom_left_origin.store(false, Ordering::SeqCst);
        self.major.store(0, Ordering::SeqCst);
        self.minor.store(0, Ordering::SeqCst);
        self.cache_context.store(false, Ordering::SeqCst);
        self.debug_context.store(false, Ordering::SeqCst);

        Ok(())
    }
}

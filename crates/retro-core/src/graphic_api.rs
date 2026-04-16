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

#[derive(Debug)]
pub struct GraphicApi {
    #[doc = " Which API to use. Set by libretro core."]
    pub context_type: RwLock<retro_hw_context_type>,

    #[doc = " Set by frontend.\n TODO: This is rather obsolete. The frontend should not\n be providing pre allocated framebuffers."]
    pub fbo: RwLock<Option<usize>>,

    #[doc = " Set if render buffers should have depth component attached.\n TODO: Obsolete."]
    pub depth: AtomicBool,

    #[doc = " Set if stencil buffers should be attached.\n TODO: Obsolete."]
    pub stencil: AtomicBool,

    #[doc = " Use conventional bottom-left origin convention. If false,
    standard libretro top-left origin semantics are used.
    TODO: Move to GL specific interface."]
    pub bottom_left_origin: AtomicBool,

    #[doc = " Major version number for core GL context or GLES 3.1+."]
    pub major: AtomicU8,

    #[doc = " Minor version number for core GL context or GLES 3.1+."]
    pub minor: AtomicU8,

    #[doc = " If this is true, the frontend will go very far to avoid\n resetting context in scenarios like toggling full_screen, etc. TODO: Obsolete? Maybe frontend should just always assume this ..."]
    pub cache_context: AtomicBool,

    #[doc = " Creates a debug context."]
    pub debug_context: AtomicBool,

    context_reset: Arc<RwLock<Option<retro_hw_context_reset_t>>>,
    context_destroy: Arc<RwLock<Option<retro_hw_context_reset_t>>>,
}

impl Default for GraphicApi {
    fn default() -> Self {
        GraphicApi {
            context_type: RwLock::new(retro_hw_context_type::RETRO_HW_CONTEXT_OPENGL),
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
    pub fn with_opengl_core() -> Self {
        Self {
            context_type: RwLock::new(retro_hw_context_type::RETRO_HW_CONTEXT_OPENGL_CORE),
            ..Default::default()
        }
    }

    pub fn with_opengl() -> Self {
        Self {
            context_type: RwLock::new(retro_hw_context_type::RETRO_HW_CONTEXT_OPENGL),
            ..Default::default()
        }
    }

    pub fn try_reset_ctx(&self) -> TinicResult<()> {
        let context_reset_fn = self.context_reset.read()?.clone();

        if let Some(Some(f)) = context_reset_fn {
            unsafe { f() }
        } else {
        }

        Ok(())
    }

    pub fn try_destroy_ctx(&self) -> TinicResult<()> {
        let context_destroy = self.context_destroy.read()?.clone();

        if let Some(Some(context_destroy)) = context_destroy {
            unsafe {
                context_destroy();
            }
        }

        Ok(())
    }

    pub fn try_update_from_raw(&self, hw_cb: &retro_hw_render_callback) -> TinicResult<bool> {
        // Salva os callbacks do CORE para o frontend chamar depois
        self.context_reset.write()?.replace(hw_cb.context_reset);
        self.context_destroy.write()?.replace(hw_cb.context_destroy);

        // Atualiza context_type com o que o core pediu
        *self.context_type.write().unwrap() = hw_cb.context_type;

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

    pub fn clear(&self) -> TinicResult<()> {
        self.context_reset.write()?.take();
        self.context_destroy.write()?.take();

        self.fbo.write()?.take();

        *self.context_type.write()? = retro_hw_context_type::RETRO_HW_CONTEXT_OPENGL;

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

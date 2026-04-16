use std::ptr::null;
use std::sync::Arc;

use tinic_generics::types::TMutex;

#[derive(Clone)]
pub struct GlProcResolver {
    loader: Arc<TMutex<Option<Arc<dyn Fn(&str) -> *const () + Send + Sync>>>>,
}

impl GlProcResolver {
    pub fn new() -> Self {
        Self {
            loader: TMutex::new(None),
        }
    }

    pub fn set_loader<F>(&self, f: F)
    where
        F: Fn(&str) -> *const () + Send + Sync + 'static,
    {
        *self
            .loader
            .load_or_spawn_err("Erro ao definir função gl")
            .unwrap() = Some(Arc::new(f));
    }

    pub fn get_proc(&self, name: &str) -> *const () {
        if let Some(loader) = &*self
            .loader
            .load_or_spawn_err("Erro ao carregar função gl")
            .unwrap()
        {
            loader(name)
        } else {
            null()
        }
    }
}

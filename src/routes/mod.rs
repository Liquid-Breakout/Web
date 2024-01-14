use std::borrow::Borrow;

use liquid_breakout_backend::Backend;

pub mod structs;
mod whitelist;

pub struct Routes {
    backend: Backend
}
impl Routes {
    pub fn new(backend: Backend) -> Self {
        Self { backend: backend }
    }
}
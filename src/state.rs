use std::sync::{Arc, atomic::AtomicBool};

use crate::{auth::Verifier, config::Config};

/// Shared by all handlers.
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    /// `/readyz` answers 200 while this is true.
    pub ready: Arc<AtomicBool>,
    /// Present when `AUTH_AUDIENCE` is configured.
    pub verifier: Option<Arc<Verifier>>,
}

impl AppState {
    pub fn new(config: Config, verifier: Option<Verifier>) -> Self {
        Self {
            config: Arc::new(config),
            ready: Arc::new(AtomicBool::new(true)),
            verifier: verifier.map(Arc::new),
        }
    }
}

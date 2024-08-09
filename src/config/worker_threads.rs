use crate::config::BLOCKING_THREADS;
use crate::config::WORKER_THREADS;

use once_cell::sync::Lazy;
use std::sync::RwLock;

// Create a Lazy static constant with RwLock
pub(crate) static CURRENT_WORKER_THREADS: Lazy<RwLock<usize>> =
    Lazy::new(|| RwLock::new(WORKER_THREADS));
pub(crate) static CURRENT_BLOCKING_THREADS: Lazy<RwLock<usize>> =
    Lazy::new(|| RwLock::new(BLOCKING_THREADS));

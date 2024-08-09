use crate::config::worker_threads::CURRENT_BLOCKING_THREADS;
use crate::config::worker_threads::CURRENT_WORKER_THREADS;

use std::sync::RwLock;

use crate::config::{BLOCKING_THREADS, WORKER_THREADS};
use crate::modules::logger::init_logger;
use crate::modules::process::run_directory_processing;
use crate::modules::runtime::build_runtime;
use crate::modules::utils::{format_duration, get_number_of_cpu_cores};
use tracing::{debug, error, info};

pub fn initialize_app() {
    let _guard = init_logger();

    info!("Application started...");

}

pub fn set_threads_count() -> (usize, usize) {
    let cpu_cores = get_number_of_cpu_cores();
    let worker_threads = cpu_cores - (cpu_cores as f64 * 0.1) as usize;
    let blocking_threads = worker_threads * 2;

    (worker_threads, blocking_threads)
}

pub fn run_app() {

    let (worker_threads, blocking_threads) = set_threads_count();
    debug!(
        "Running with {} worker threads and {} blocking threads",
        worker_threads, blocking_threads
    );

    debug!("Setting up runtime:");

    // // Configure Tokio runtime with optimized settings for high-performance system
    let runtime = build_runtime(worker_threads, blocking_threads);

    debug!(
        "Running with {} worker threads and {} blocking threads",
        worker_threads, blocking_threads
    );

    // Run the async function using the configured runtime
    runtime.block_on(async {
        let time_used = run_directory_processing().await;
        info!("Time used: {:?}", format_duration(time_used));
    });

    info!("Application finished.");
}

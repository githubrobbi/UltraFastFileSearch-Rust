use tokio::runtime::Builder;

pub(crate) fn build_runtime(
    worker_threads: usize,
    blocking_threads: usize,
) -> tokio::runtime::Runtime {
    Builder::new_multi_thread()
        .worker_threads(worker_threads)
        .max_blocking_threads(blocking_threads)
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime")
}

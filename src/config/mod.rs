pub mod app_configs;
pub mod constants;
pub mod worker_threads;

pub(crate) use app_configs::config;

pub(crate) use constants::BLOCKING_THREADS;
pub(crate) use constants::LOG_DATE_FORMAT;
pub(crate) use constants::MAX_CONCURRENT_READS;
pub(crate) use constants::MAX_DIRS;
pub(crate) use constants::MAX_DIRS_ALL;
pub(crate) use constants::MAX_FILES;
pub(crate) use constants::MAX_FILES_ALL;
pub(crate) use constants::MAX_TEMP_FILES;
pub(crate) use constants::MAX_TEMP_FILES_HDD_BATCH;
pub(crate) use constants::WORKER_THREADS;

pub(crate) use worker_threads::CURRENT_WORKER_THREADS;

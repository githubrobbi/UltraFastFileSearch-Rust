// ------------------------------------------------------------------------------
// Filename: logger_impl
// Path: ./src/logger_impl
// Original Author: Robert S.A. Nio
// Contributor:
// Date: 2024-07-28
// ------------------------------------------------------------------------------
// Description: This module initializes and configures the logging system for the
//              Ultra Fast File Search Tool (UFFS). It sets up both terminal and
//              file-based logging with environment-based log levels and ensures
//              that log files are created and managed properly.
// ------------------------------------------------------------------------------
// Copyright © 2024 SKY, LLC. All Rights Reserved.
//
// This software is the confidential and proprietary information of SKY, LLC..
// ("Confidential Information"). You shall not disclose such Confidential
// Information and shall use it only in accordance with the terms of the license
// agreement you entered into with SKY, LLC..
// ------------------------------------------------------------------------------
// For more information on standards and best practices, refer to
// <CORPORATE DEVELOPMENT GUIDELINES LINK OR RESOURCE>
// ------------------------------------------------------------------------------

use dirs_next::home_dir;
use std::io::stdout;
use std::path::PathBuf;
use std::{env, fs};
use tracing::{debug, info};
use tracing_appender::non_blocking::{NonBlocking, WorkerGuard};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::registry::Registry;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::{fmt, Layer};

// Function to get the logging level from the environment variable or default to LevelFilter::Info
fn get_log_level() -> EnvFilter {
    EnvFilter::new(env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()))
}

fn log_location_and_name() -> (PathBuf, String) {
    // Get the home directory
    let home_dir = home_dir().expect("Failed to find home directory");

    // Specify the output directory for logs
    let log_output_dir = home_dir.join("bin").join("rust");

    // Create the output directory if it doesn't exist
    fs::create_dir_all(&log_output_dir).expect("Failed to create 'home_dir/bin/rust'");

    // Create the log file name
    // let log_file_name = format!("{}_uffs_log.txt", current_date);
    let log_file_name = "uffs_log_".to_string();

    // Return the path of the log file
    (log_output_dir, log_file_name)
}

pub fn init_logger() -> WorkerGuard {
    let (log_directory_path, log_file_name) = log_location_and_name();

    // Create a rolling file appender
    let file_appender =
        RollingFileAppender::new(Rotation::DAILY, log_directory_path, log_file_name);
    let (non_blocking, guard) = NonBlocking::new(file_appender);

    // Create environment filters for log levels
    let terminal_filter = get_log_level();
    let file_filter = get_log_level();

    // Create a format for the timer
    let timer = UtcTime::rfc_3339();

    // Create the terminal layer
    let terminal_layer = fmt::layer()
        .with_writer(stdout)
        .with_timer(timer.clone())
        .with_ansi(true)
        .with_filter(terminal_filter); // Enable ANSI colors

    // Create the file layer
    let file_layer = fmt::layer()
        .with_writer(non_blocking)
        .with_timer(timer)
        .with_ansi(false)
        .with_filter(file_filter); // Disable ANSI colors

    // Combine the layers and initialize the subscriber
    let subscriber = Registry::default().with(terminal_layer).with(file_layer);

    tracing::subscriber::set_global_default(subscriber)
        .expect("Unable to set global tracing subscriber");

    debug!("Logger initialized and running in UTC.");

    guard
}

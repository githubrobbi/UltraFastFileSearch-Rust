// ------------------------------------------------------------------------------
// Filename: build.rs
// Path: ./build.rs
// Original Author: Robert S.A. Nio
// Contributor:
// Date: 2024-07-28
// ------------------------------------------------------------------------------
// Description: This build script sets build-time environment variables and
//              performs pre-build tasks for the Ultra Fast File Search Tool (UFFS).
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

use chrono::Local;
use dirs_next::home_dir;
use simplelog::*;
use std::env;
#[cfg(target_family = "unix")]
use std::fs::Permissions;
use std::fs::{self, File};
#[cfg(target_family = "unix")]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
#[cfg(target_family = "unix")]
use std::process::Command;
use time::macros::format_description;
use time::UtcOffset;
use toml::Value;

const OUT_DIR: &str = "bin";

/// Function to find the Cargo.toml file location by checking the environment variable
/// and walking up the directory tree if necessary
/// Function to find the Cargo.toml file location by checking the environment variable
/// and falling back to the PROJECT_DIR constant if necessary
/// USER_HOME/rust/uffs/Cargo.toml
fn find_cargo_toml() -> std::io::Result<PathBuf> {
    // The project directory
    let project_dir = get_path().unwrap();

    // Check if the CARGO_MANIFEST_DIR environment variable is set
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let manifest_path = PathBuf::from(manifest_dir).join("Cargo.toml");
        if manifest_path.exists() {
            return Ok(manifest_path);
        } else {
            log::error!(
                "CARGO_MANIFEST_DIR is set but Cargo.toml not found at {}",
                manifest_path.display()
            );
        }
    } else {
        log::error!("CARGO_MANIFEST_DIR environment variable is not set.");
    }

    let manifest_path = home_dir()
        .ok_or(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not determine home directory",
        ))?
        .join(project_dir)
        .join("Cargo.toml");
    if manifest_path.exists() {
        return Ok(manifest_path);
    } else {
        log::error!("Cargo.toml not found at {}", manifest_path.display());
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "Cargo.toml not found",
    ))
}

/// Reads and parses a TOML file.
///
/// # Arguments
///
/// * `path` - The path to the TOML file.
///
/// # Returns
///
/// * `Result<Option<Value>, std::io::Error>` - The parsed TOML value or `None` if the file doesn't exist or can't be parsed.
fn read_toml_file(path: &Path) -> std::io::Result<Option<Value>> {
    if path.exists() {
        let content = fs::read_to_string(path)?;
        Ok(toml::from_str(&content).ok())
    } else {
        Ok(None)
    }
}

#[cfg(target_family = "unix")]
fn set_unix_permissions(path: &Path) -> std::io::Result<()> {
    let permissions = Permissions::from_mode(0o755);
    fs::set_permissions(path, permissions)
}

fn get_path() -> std::io::Result<PathBuf> {
    #[cfg(target_os = "windows")]
    let path = Ok(PathBuf::from("C:\\".to_string()));

    #[cfg(target_family = "unix")]
    let path = get_unix_path();

    path
}

#[cfg(target_family = "unix")]
fn get_unix_path() -> std::io::Result<PathBuf> {
    // Construct the full path
    let path: PathBuf = home_dir()
        .ok_or(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not determine home directory",
        ))?
        .join("GitHub/uffs");

    Ok(path)
}

// Function to get the build directory from the global configuration
fn get_build_directory(global_config: &Value) -> Option<PathBuf> {
    global_config
        .get("build")
        .and_then(|build| build.get("target-dir"))
        .and_then(|target_dir| target_dir.as_str())
        .map(|dir| dir.into())
}

// Function to ensure the build directory exists under the user's home directory
fn ensure_build_directory_exists(global_config: &Value) -> std::io::Result<PathBuf> {
    // Get the user's home directory
    let home_dir = home_dir().ok_or(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "Could not determine home directory",
    ))?;

    // Get the build directory from the global configuration
    let build_dir = get_build_directory(global_config).ok_or(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "Could not determine BIN directory from config",
    ))?;

    // Construct the full path under the home directory
    let full_bin_path = home_dir.join(build_dir);

    // Create the directory if it doesn't exist
    if !full_bin_path.exists() {
        fs::create_dir_all(&full_bin_path)?;
    }

    Ok(full_bin_path)
}

// Function to get the logging level from the environment variable or default to `LevelFilter::Info`
fn get_log_level() -> LevelFilter {
    match env::var("LOG_LEVEL") {
        Ok(level) => match level.to_lowercase().as_str() {
            "error" => LevelFilter::Error,
            "warn" => LevelFilter::Warn,
            "info" => LevelFilter::Info,
            "debug" => LevelFilter::Debug,
            "trace" => LevelFilter::Trace,
            _ => LevelFilter::Info,
        },
        Err(_) => LevelFilter::Info,
    }
}

// Define a constant for the time offset
const TIME_OFFSET: UtcOffset = UtcOffset::UTC;
// Custom time format using the `time` crate, including a UTC indicator
const TIME_FORMAT: &[FormatItem<'static>] =
    format_description!("[year]-[month]-[day] [hour]:[minute]:[second] 'UTC'");

fn main() -> std::io::Result<()> {
    let current_log_level = get_log_level();

    // Ensure the script runs every time
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-env-changed=FORCE_REBUILD");
    println!("cargo:rerun-if-env-changed=PROFILE");

    // Get the build profile
    let profile = env::var("PROFILE")
        .unwrap_or_else(|_| "debug".to_string())
        .to_uppercase();

    // Specify the output directory for BUILD.RS
    let debug_output_dir = home_dir()
        .ok_or(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Failed to find home directory",
        ))?
        .join("bin")
        .join("rust");

    // Create the output directory if it doesn't exist
    fs::create_dir_all(&debug_output_dir).unwrap_or_else(|_| {
        panic!(
            "Could not create build.rs output DIRECTORY for profile: {}",
            profile.clone()
        )
    });

    // Get the current date
    let current_date = Local::now().format("%Y-%m-%d").to_string();

    let debug_output_file = format!("{}_{}_build.rs_output.txt", current_date, profile);

    // Specify the output file name within the output directory
    let debug_output_file_location = debug_output_dir.join(debug_output_file);

    // Ensure the file is fresh by deleting it if it exists
    if debug_output_file_location.exists() {
        fs::remove_file(&debug_output_file_location)?;
    }

    // Create and write to the debug output file
    let debug_file = File::create(&debug_output_file_location).unwrap_or_else(|_| {
        panic!(
            "Could not create build.rs output FILE for profile: {}",
            profile.clone()
        )
    });

    // Initialize the logger with custom configuration
    CombinedLogger::init(vec![
        TermLogger::new(
            current_log_level,
            ConfigBuilder::new()
                .set_time_format_custom(TIME_FORMAT)
                .set_time_offset(TIME_OFFSET)
                .build(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            current_log_level,
            ConfigBuilder::new()
                .set_time_format_custom(TIME_FORMAT)
                .set_time_offset(TIME_OFFSET)
                .build(),
            debug_file,
        ),
    ])
    .unwrap();

    // Example log message
    log::info!("The build.rs script is just STARTED!");
    log::info!("Build profile: {:?}", profile);
    log::info!("Loglevel: {}", get_log_level());

    // Path to the global Cargo configuration file
    let global_config_path = home_dir()
        .ok_or(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Failed to find home directory",
        ))?
        .join(".cargo")
        .join("config.toml");

    log::info!("Global config path: {:?}", global_config_path);

    // Read the global configuration
    let global_config = match read_toml_file(&global_config_path) {
        Ok(Some(config)) => config,
        Ok(None) => {
            log::error!(
                "Global configuration file not found at {:?}",
                global_config_path
            );

            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!(
                    "Global configuration file not found at {:?}",
                    global_config_path
                ),
            ));
        }
        Err(e) => {
            log::error!("Failed to read global configuration file: {:?}", e);

            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to read global configuration file: {:?}", e),
            ));
        }
    };

    if current_log_level != LevelFilter::Info {
        log::warn!("Global config: {:?}", global_config);
    }

    // Get the build directory from the global configuration
    let global_build_dir = ensure_build_directory_exists(&global_config)?;

    log::info!("Global BUILD dir: {:?}", global_build_dir);

    let bin_dir = global_build_dir
        .parent()
        .map(|p| p.to_path_buf())
        .ok_or(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Parent directory not found for the build directory",
        ))?;

    log::info!("Global BIN dir: {:?}", bin_dir);

    // Determine the path to the Cargo.toml file
    let manifest_path = find_cargo_toml()?;

    log::info!("Manifest path: {:?}", manifest_path);

    // Read the contents of the Cargo.toml file
    let manifest_content = fs::read_to_string(manifest_path)?;

    if current_log_level != LevelFilter::Info {
        log::warn!("Manifest content: {:?}", manifest_content);
    }

    // Parse the Cargo.toml file
    let manifest: Value = toml::from_str(&manifest_content)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    if current_log_level != LevelFilter::Info {
        log::warn!("Parsed manifest: {:?}", manifest);
    }

    // Get the package name
    let base_package_name = manifest["package"]["name"]
        .as_str()
        .ok_or(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Package name not found in Cargo.toml",
        ))?;

    // Attempt to extract the binary name from the bin section
    let bin_name = manifest["bin"].get(0).and_then(|bin| bin["name"].as_str());

    // Use bin_name if it exists, otherwise use base_package_name
    let package_name = bin_name.unwrap_or(base_package_name);

    log::info!("Package name: {:?}", package_name);

    // Conditionally add .exe extension to package_name on Windows
    let mut binary_name = package_name.to_string();

    #[cfg(target_os = "windows")]
    {
        if !binary_name.ends_with(".exe") {
            binary_name.push_str(".exe");
        }
    }

    log::info!("TARGET dir: {:?}", bin_dir);

    // Path to the final binary
    let default_out_dir = home_dir()
        .ok_or(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not determine home directory",
        ))?
        .join(OUT_DIR);

    // Check if the OUT_DIR environment variable is set and points to an existing directory
    let out_dir_path = if let Ok(out_dir) = env::var("OUT_DIR") {
        let out_dir_path = PathBuf::from(out_dir);
        if out_dir_path.exists() {
            log::info!("OUT_DIR is set and found at {}", out_dir_path.display());
            out_dir_path
        } else {
            log::info!(
                "OUT_DIR is set but directory not found at {}",
                out_dir_path.display()
            );
            log::info!(
                "Using default output directory: {}",
                default_out_dir.display()
            );
            default_out_dir
        }
    } else {
        log::info!("OUT_DIR environment variable is not set.");
        // Fall back to the default directory if OUT_DIR is not set
        log::info!(
            "Using default output directory: {}",
            default_out_dir.display()
        );
        default_out_dir
    };

    let binary_path = out_dir_path
        .ancestors()
        .nth(3)
        .unwrap()
        .join(binary_name.clone());

    log::info!("Binary path: {:?}", binary_path);

    // Path to the target location
    let target_location = bin_dir.join(package_name);

    log::info!("Target location: {:?}", target_location);

    // Generate the appropriate script based on the target OS
    #[cfg(target_family = "unix")]
    {
        let script_path = global_build_dir.join("copy_binary.sh");
        let script_content = format!(
            r#"#!/bin/bash

BINARY="{}"
BINARY_PATH="{}"
TARGET_LOCATION="{}"
PROFILE="{}"

if [ -e "$BINARY_PATH" ]; then
    cp "$BINARY_PATH" "$TARGET_LOCATION"
    if [ $? -eq 0 ]; then
        echo -e "\nSuccessfully copied the $PROFILE version of '$BINARY' to: \t\t\t$TARGET_LOCATION"
        # Set extended attribute to mark the build type for the binary
        xattr -w com.$BINARY.buildtype "$PROFILE" "$TARGET_LOCATION"
        if [ $? -eq 0 ]; then
            echo -e "Successfully set extended attribute for build type on '$BINARY': \t$PROFILE"
        else
            echo -e "Failed to set extended attribute for build type on '$BINARY': $PROFILE" >&2
        fi
    else
        echo -e "\nFailed to copy the $PROFILE version of '$BINARY' to: \t$TARGET_LOCATION\n" >&2
    fi
else
    echo -e "\nThe $PROFILE version of '$BINARY' does not exist: \t$BINARY_PATH\n" >&2
fi
"#,
            binary_name,
            binary_path.display(),
            target_location.display(),
            profile
        );

        fs::write(&script_path, script_content)?;

        log::info!("Successfully created shell script at {:?}", script_path);

        // Set the execution rights
        set_unix_permissions(&script_path)?;

        log::info!(
            "Successfully set execution rights for shell script at {:?}",
            script_path
        );

        let attribute_name = format!("com.{}.buildtype", package_name);
        // Set extended attribute to mark the build type for the shell script
        let xattr_result = Command::new("xattr")
            .arg("-w")
            .arg(&attribute_name)
            .arg(&profile)
            .arg(&script_path)
            .output();

        match xattr_result {
            Ok(output) if output.status.success() => {
                log::info!(
                    "Successfully set extended attribute for build type on shell script: {}",
                    profile
                );
            }
            Ok(output) => {
                let error_message = format!(
                    "Failed to set extended attribute for build type on shell script: {}: {:?}",
                    profile, output
                );
                log::error!("{}", error_message);
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    error_message,
                ));
            }
            Err(e) => {
                log::error!("Failed to execute xattr command for shell script: {:?}", e);
                return Err(e);
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let script_path = global_build_dir.join("copy_binary.bat");
        let script_content = format!(
            r#"@echo off

set BINARY={}
set BINARY_PATH={}
set TARGET_LOCATION={}
set PROFILE={}

if exist "%BINARY_PATH%" (
    copy "%BINARY_PATH%" "%TARGET_LOCATION%"
    if %errorlevel% == 0 (
        echo.
        echo Successfully copied the %PROFILE% version of '%BINARY%' to:%TAB%%TAB%%TARGET_LOCATION%
        rem Note: Windows does not have an equivalent to xattr. You may need to use an alternative method.
    ) else (
        echo Failed to copy the %PROFILE% version of '%BINARY%' to: %TARGET_LOCATION% >&2
    )
) else (
    echo The %PROFILE% version of '%BINARY%' does not exist: %BINARY_PATH% >&2
)
"#,
            binary_name,
            binary_path.display(),
            format_args!("{}.exe", target_location.display()),
            profile
        );

        fs::write(&script_path, script_content)?;

        log::info!("Successfully created shell script at {:?}", script_path);
    }

    Ok(())
}

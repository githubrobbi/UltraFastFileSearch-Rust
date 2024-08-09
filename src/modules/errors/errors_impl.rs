use std::io;
use thiserror::Error;
use miette::{Diagnostic, SourceSpan};

#[derive(Error, Debug, Diagnostic)]
pub(crate) enum UFFSError {
    #[error("IO error: {0}")]
    #[diagnostic(code(uff::io_error), help("Check if the file path is correct and you have the necessary permissions."))]
    Io(#[from] io::Error),

    #[error("Drive information is empty.")]
    #[diagnostic(code(uff::empty_drive_info), help("Ensure that the drive is properly connected and contains data."))]
    EmptyDriveInfo,

    #[error("Drive letter not found.")]
    #[diagnostic(code(uff::drive_letter_not_found), help("Verify that the drive letter is correct and accessible."))]
    DriveLetterNotFound,

    #[error("Failed to read directory entries.")]
    #[diagnostic(code(uff::directory_read_error), help("Check the directory path and your access permissions."))]
    DirectoryReadError,

    #[error("Configuration error: {0}")]
    #[diagnostic(code(uff::config_error))]
    ConfigError(String),

    #[error("Custom error with data: {message}, data: {data}")]
    #[diagnostic(code(uff::custom_error))]
    CustomError { message: String, data: usize },
}

//! Error types for the ALX library.

use std::path::PathBuf;
use thiserror::Error;

/// Result type alias for ALX operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during ALX operations.
#[derive(Debug, Error)]
pub enum Error {
    /// I/O error during file operations.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// The ISO file is invalid or not a supported game.
    #[error("Invalid ISO: {0}")]
    InvalidIso(String),

    /// The game version is not supported.
    #[error("Unsupported game version: {0}")]
    UnsupportedVersion(String),

    /// A required file was not found in the ISO.
    #[error("File not found in ISO: {path}")]
    FileNotFound { path: PathBuf },

    /// Binary data parsing error.
    #[error("Parse error at offset {offset:#x}: {message}")]
    ParseError { offset: usize, message: String },

    /// String encoding error.
    #[error("String encoding error: {0}")]
    EncodingError(String),

    /// CSV parsing/writing error.
    #[error("CSV error: {0}")]
    CsvError(#[from] ::csv::Error),

    /// Data validation error.
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// gc_fst ISO operation error.
    #[error("ISO operation error: {0}")]
    IsoOperationError(String),
}

impl From<gc_fst::ReadISOFilesError> for Error {
    fn from(e: gc_fst::ReadISOFilesError) -> Self {
        Error::IsoOperationError(format!("{:?}", e))
    }
}

impl From<gc_fst::OperateISOError> for Error {
    fn from(e: gc_fst::OperateISOError) -> Self {
        Error::IsoOperationError(format!("{:?}", e))
    }
}


//! Custom error types used in this crate.

use std::error::Error;
use std::fmt::{self, Display};

#[derive(Debug, Clone, Copy)]
pub enum FreesasaErrorKind {
    Unknown,
    Memory,
    Io,
    Parameter,
    Structure,
    Selection,
    Ffi,
}

impl Display for FreesasaErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = match self {
            FreesasaErrorKind::Unknown => "Unknown",
            FreesasaErrorKind::Memory => "Memory",
            FreesasaErrorKind::Io => "I/O",
            FreesasaErrorKind::Parameter => "Parameter",
            FreesasaErrorKind::Structure => "Structure",
            FreesasaErrorKind::Selection => "Selection",
            FreesasaErrorKind::Ffi => "FFI",
        };
        write!(f, "{}", kind)
    }
}

/// Error type for the `freesasa` crate.
///
#[derive(Debug)]
pub struct FreesasaError {
    message: String,
    kind: FreesasaErrorKind,
    code: Option<i32>,
}

impl FreesasaError {
    /// Create a new `FreesasaError`.
    ///
    /// # Arguments
    ///
    /// * `message` - A string slice that holds the error message.
    /// * `code` - An optional integer that holds the error code.
    ///
    pub fn new(
        message: &str,
        kind: FreesasaErrorKind,
        code: Option<i32>,
    ) -> FreesasaError {
        FreesasaError {
            message: message.to_string(),
            kind,
            code,
        }
    }

    /// Get the error code.
    ///
    /// # Returns
    ///
    /// An optional integer that holds the error code.
    ///
    pub fn code(&self) -> Option<i32> {
        self.code
    }
}

impl fmt::Display for FreesasaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut message = self.message.clone();

        message = format!("{} ({})", message, self.kind);

        if let Some(code) = &self.code {
            message = format!("{} ({})", message, code);
        }
        write!(f, "{}", message)
    }
}

impl Error for FreesasaError {
    fn description(&self) -> &str {
        &self.message
    }
}

// From implementations for error types from the standard library.
impl From<std::io::Error> for FreesasaError {
    fn from(error: std::io::Error) -> Self {
        FreesasaError::new(
            &error.to_string(),
            FreesasaErrorKind::Io,
            None,
        )
    }
}

impl From<std::ffi::NulError> for FreesasaError {
    fn from(error: std::ffi::NulError) -> Self {
        FreesasaError::new(
            &error.to_string(),
            FreesasaErrorKind::Ffi,
            None,
        )
    }
}

impl From<std::string::FromUtf8Error> for FreesasaError {
    fn from(error: std::string::FromUtf8Error) -> Self {
        FreesasaError::new(
            &error.to_string(),
            FreesasaErrorKind::Ffi,
            None,
        )
    }
}

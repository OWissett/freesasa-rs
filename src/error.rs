//! Custom error types used in this crate.

use std::error::Error;
use std::fmt;

/// Error type for the `freesasa` crate.
///
#[derive(Debug)]
pub struct FreesasaError {
    message: String,
    kind: Option<String>,
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
        kind: Option<String>,
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
        if let Some(kind) = &self.kind {
            message = format!("{} ({})", message, kind);
        }
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
            Some("io".to_owned()),
            None,
        )
    }
}

impl From<std::ffi::NulError> for FreesasaError {
    fn from(error: std::ffi::NulError) -> Self {
        FreesasaError::new(
            &error.to_string(),
            Some("ffi".to_owned()),
            None,
        )
    }
}

impl From<std::string::FromUtf8Error> for FreesasaError {
    fn from(error: std::string::FromUtf8Error) -> Self {
        FreesasaError::new(
            &error.to_string(),
            Some("string".to_owned()),
            None,
        )
    }
}

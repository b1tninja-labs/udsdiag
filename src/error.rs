//! The crate error type.
//!
//! The Python original raised `ValueError` for an out-of-range value or a malformed PDU. The
//! Rust port returns [`Error`] from the fallible functions instead; it carries a human-readable
//! message and nothing else.

use core::fmt;

/// An invalid argument or a malformed PDU: the Rust counterpart of the Python `ValueError`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    message: String,
}

impl Error {
    /// Build an error with the given message.
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for Error {}

/// A `Result` whose error is this crate's [`Error`].
pub type Result<T> = core::result::Result<T, Error>;

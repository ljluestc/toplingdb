//! Status and error handling module
//!
//! This module provides error types and status codes for ToplingDB operations.

use std::fmt;

/// Status codes used throughout ToplingDB
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StatusCode {
    /// Operation completed successfully
    Ok,
    /// Key not found
    NotFound,
    /// Operation failed due to corruption
    Corruption,
    /// Operation not supported
    NotSupported,
    /// Invalid argument provided
    InvalidArgument,
    /// I/O error occurred
    IoError,
    /// Merge operation required but no merge operator provided
    MergeInProgress,
    /// Operation incomplete
    Incomplete,
    /// Database is shutting down
    ShutdownInProgress,
    /// Timed out waiting for operation
    TimedOut,
    /// Operation was aborted
    Aborted,
    /// Database is busy
    Busy,
    /// Operation expired
    Expired,
    /// Try again later
    TryAgain,
    /// Compaction too large
    CompactionTooLarge,
    /// Column family dropped
    ColumnFamilyDropped,
    /// Maximum sequence number reached
    MaxSequenceNumberReached,
}

impl fmt::Display for StatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ok => write!(f, "OK"),
            Self::NotFound => write!(f, "NotFound"),
            Self::Corruption => write!(f, "Corruption"),
            Self::NotSupported => write!(f, "Not implemented"),
            Self::InvalidArgument => write!(f, "Invalid argument"),
            Self::IoError => write!(f, "IO error"),
            Self::MergeInProgress => write!(f, "Merge in progress"),
            Self::Incomplete => write!(f, "Result incomplete"),
            Self::ShutdownInProgress => write!(f, "Shutdown in progress"),
            Self::TimedOut => write!(f, "Operation timed out"),
            Self::Aborted => write!(f, "Operation aborted"),
            Self::Busy => write!(f, "Resource busy"),
            Self::Expired => write!(f, "Operation expired"),
            Self::TryAgain => write!(f, "Try again"),
            Self::CompactionTooLarge => write!(f, "Compaction too large"),
            Self::ColumnFamilyDropped => write!(f, "Column family dropped"),
            Self::MaxSequenceNumberReached => write!(f, "Maximum sequence number reached"),
        }
    }
}

/// Status represents the result of an operation
#[derive(Debug, Clone)]
pub struct Status {
    code: StatusCode,
    subcode: u8,
    severity: Severity,
    message: Option<String>,
    state: Option<String>,
}

/// Severity levels for status messages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// No error
    NoError = 0,
    /// Soft error that can be recovered from
    SoftError = 1,
    /// Hard error that requires intervention
    HardError = 2,
    /// Fatal error that causes termination
    FatalError = 3,
    /// Unrecoverable error
    UnrecoverableError = 4,
}

impl Status {
    /// Create a new OK status
    pub fn ok() -> Self {
        Self {
            code: StatusCode::Ok,
            subcode: 0,
            severity: Severity::NoError,
            message: None,
            state: None,
        }
    }

    /// Create a new status with the given code
    pub fn new(code: StatusCode, message: Option<String>) -> Self {
        Self {
            code,
            subcode: 0,
            severity: match code {
                StatusCode::Ok => Severity::NoError,
                StatusCode::NotFound | StatusCode::Incomplete => Severity::SoftError,
                StatusCode::Corruption | StatusCode::IoError => Severity::HardError,
                _ => Severity::SoftError,
            },
            message,
            state: None,
        }
    }

    /// Create a NotFound status
    pub fn not_found(message: Option<String>) -> Self {
        Self::new(StatusCode::NotFound, message)
    }

    /// Create a Corruption status
    pub fn corruption(message: Option<String>) -> Self {
        Self::new(StatusCode::Corruption, message)
    }

    /// Create an InvalidArgument status
    pub fn invalid_argument(message: Option<String>) -> Self {
        Self::new(StatusCode::InvalidArgument, message)
    }

    /// Create an IoError status
    pub fn io_error(message: Option<String>) -> Self {
        Self::new(StatusCode::IoError, message)
    }

    /// Create a NotSupported status
    pub fn not_supported(message: Option<String>) -> Self {
        Self::new(StatusCode::NotSupported, message)
    }

    /// Check if status is OK
    pub fn is_ok(&self) -> bool {
        self.code == StatusCode::Ok
    }

    /// Check if status is NotFound
    pub fn is_not_found(&self) -> bool {
        self.code == StatusCode::NotFound
    }

    /// Check if status is Corruption
    pub fn is_corruption(&self) -> bool {
        self.code == StatusCode::Corruption
    }

    /// Check if status is InvalidArgument
    pub fn is_invalid_argument(&self) -> bool {
        self.code == StatusCode::InvalidArgument
    }

    /// Check if status is IoError
    pub fn is_io_error(&self) -> bool {
        self.code == StatusCode::IoError
    }

    /// Get the status code
    pub fn code(&self) -> StatusCode {
        self.code
    }

    /// Get the subcode
    pub fn subcode(&self) -> u8 {
        self.subcode
    }

    /// Get the severity
    pub fn severity(&self) -> Severity {
        self.severity
    }

    /// Get the message
    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    /// Get the state
    pub fn state(&self) -> Option<&str> {
        self.state.as_deref()
    }

    /// Convert to string representation
    pub fn to_string(&self) -> String {
        let mut result = self.code.to_string();
        if let Some(msg) = &self.message {
            result.push_str(": ");
            result.push_str(msg);
        }
        result
    }
}

impl Default for Status {
    fn default() -> Self {
        Self::ok()
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl std::error::Error for Status {}

impl From<std::io::Error> for Status {
    fn from(err: std::io::Error) -> Self {
        Self::io_error(Some(err.to_string()))
    }
}

impl From<Status> for std::io::Error {
    fn from(status: Status) -> Self {
        let kind = match status.code {
            StatusCode::NotFound => std::io::ErrorKind::NotFound,
            StatusCode::InvalidArgument => std::io::ErrorKind::InvalidInput,
            StatusCode::IoError => std::io::ErrorKind::Other,
            StatusCode::TimedOut => std::io::ErrorKind::TimedOut,
            StatusCode::Aborted => std::io::ErrorKind::Interrupted,
            _ => std::io::ErrorKind::Other,
        };
        std::io::Error::new(kind, status.to_string())
    }
}

/// Convenient alias for Status as an Error type
pub type Error = Status;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_ok() {
        let status = Status::ok();
        assert!(status.is_ok());
        assert_eq!(status.code(), StatusCode::Ok);
        assert_eq!(status.severity(), Severity::NoError);
    }

    #[test]
    fn test_status_not_found() {
        let status = Status::not_found(Some("Key not found".to_string()));
        assert!(status.is_not_found());
        assert_eq!(status.code(), StatusCode::NotFound);
        assert_eq!(status.message(), Some("Key not found"));
    }

    #[test]
    fn test_status_display() {
        let status = Status::corruption(Some("Data corrupted".to_string()));
        assert_eq!(status.to_string(), "Corruption: Data corrupted");
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let status = Status::from(io_err);
        assert!(status.is_io_error());

        let status = Status::not_found(Some("Key missing".to_string()));
        let io_err = std::io::Error::from(status);
        assert_eq!(io_err.kind(), std::io::ErrorKind::NotFound);
    }
}
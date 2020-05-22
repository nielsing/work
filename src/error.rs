use std::error;
use std::fmt;
use std::io;

/// An error that can occur in this crate.
///
/// There are two main reasons for a error in this crate.
/// Either the user did something silly like a wrong input or something went wrong regardin the
/// log file.
///
/// These errors are meant to "flow upwards" and eventually printed to the terminal. If a function
/// returns an AppError, it most likely returns all the way back to `main()`.
#[derive(Clone, Debug)]
pub struct AppError {
    kind: ErrorKind,
}

impl AppError {
    pub(crate) fn new(kind: ErrorKind) -> AppError {
        AppError { kind }
    }

    /// Return the `kind` of this error
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }
}

#[derive(Clone, Debug)]
pub enum ErrorKind {
    User(String),
    System(String),
    LogFile(String),
}

impl error::Error for AppError {
    fn description(&self) -> &str {
        match self.kind {
            ErrorKind::User(_) => "user error",
            ErrorKind::System(_) => "system error",
            ErrorKind::LogFile(_) => "log file error",
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            ErrorKind::User(ref s) => write!(f, "{}", s),
            ErrorKind::System(ref s) => write!(f, "{}", s),
            ErrorKind::LogFile(ref s) => write!(f, "{}", s),
        }
    }
}

impl From<io::Error> for AppError {
    fn from(error: io::Error) -> Self {
        match error.kind() {
            io::ErrorKind::NotFound => {
                AppError::new(ErrorKind::LogFile("Work log does not exist!".to_string()))
            }
            io::ErrorKind::PermissionDenied => AppError::new(ErrorKind::LogFile(
                "Invalid permissions for work log!".to_string(),
            )),
            _ => AppError::new(ErrorKind::LogFile(
                "Unable to write/read to/from work log!".to_string(),
            )),
        }
    }
}

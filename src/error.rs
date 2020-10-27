use winapi::shared::windef::HWND;

use std::{error, fmt};

pub type ExtError = Box<dyn error::Error + Send + Sync + 'static>;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    WindowFound((HWND, u32)),
    NoWindowFound((HWND, u32)),
    Unimplemented,
    SysError(String),
    Error(ExtError)
}

impl error::Error for Error {}

unsafe impl Send for Error {}
unsafe impl Sync for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::WindowFound(ref key) => {
                write!(f, "window already exists in handler and over written: ({:?}, {})", key.0, key.1)
            }
            Error::NoWindowFound(ref key) => {
                write!(f, "no window found for provided keys in handler: ({:?}, {})", key.0, key.1)
            },
            Error::Unimplemented => {
                write!(f, "unimplemented")
            },
            Error::SysError(ref e) => write!(f, "{}", e),
            Error::Error(ref e) => write!(f, "Error: {}", e),
        }
    }
}

impl From<ExtError> for Error {
    fn from(err: ExtError) -> Error {
        Error::Error(err)
    }
}

impl From<String> for Error {
    fn from(err: String) -> Error {
        Error::SysError(err)
    }
}

impl From<u32> for Error {
    fn from(err: u32) -> Error {
        Error::SysError(format!("System error: {}", err))
    }
}
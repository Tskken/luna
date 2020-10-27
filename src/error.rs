use winapi::shared::windef::HWND;

use std::{error, fmt};

pub type ExtError = Box<dyn error::Error + Send + Sync + 'static>;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    CallbackFound((HWND, u32)),
    NoCallbackFound((HWND, u32)),
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
            Error::CallbackFound(ref key) => {
                write!(f, "callback already exists and over written: ({:?}, {})", key.0, key.1)
            }
            Error::NoCallbackFound(ref key) => {
                write!(f, "no callback found for provided keys: ({:?}, {})", key.0, key.1)
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
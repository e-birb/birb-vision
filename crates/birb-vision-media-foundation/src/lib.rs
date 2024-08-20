#![cfg(windows)]

use std::error::Error;

use windows::core::{HRESULT, Error as WinError};


mod ctx;
mod device;
mod media_type;

pub use ctx::MediaFoundationContext;
pub use device::{
    MFDeviceInfo,
    MFDevice,
};
pub use media_type::*;

#[derive(Debug)]
pub enum MFError {
    WinError(WinError),
    Other(Box<dyn Error>),
}

impl MFError {
    pub fn from_hresult(hr: HRESULT) -> MFResult<()> {
        if hr.is_ok() {
            Ok(())
        } else {
            Err(MFError::WinError(WinError::from_win32()))
        }
    }
}

impl From<WinError> for MFError {
    fn from(err: WinError) -> Self {
        MFError::WinError(err)
    }
}

impl From<Box<dyn Error>> for MFError {
    fn from(err: Box<dyn Error>) -> Self {
        MFError::Other(err)
    }
}

impl std::fmt::Display for MFError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MFError::WinError(err) => write!(f, "{}", err),
            MFError::Other(err) => write!(f, "{}", err),
        }
    }
}

impl Error for MFError {}

pub type MFResult<T> = Result<T, MFError>;
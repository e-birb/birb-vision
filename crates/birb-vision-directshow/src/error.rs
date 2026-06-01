use std::error::Error;

use windows::core::Error as WinError;

#[derive(Debug)]
pub enum DSError {
    WinError(WinError),
    Other(Box<dyn Error + Send + Sync + 'static>),
    Message(String),
}

impl DSError {
    pub fn msg(msg: impl Into<String>) -> Self {
        DSError::Message(msg.into())
    }
}

impl From<WinError> for DSError {
    fn from(err: WinError) -> Self {
        DSError::WinError(err)
    }
}

impl From<Box<dyn Error + Send + Sync + 'static>> for DSError {
    fn from(err: Box<dyn Error + Send + Sync + 'static>) -> Self {
        DSError::Other(err)
    }
}

impl From<String> for DSError {
    fn from(msg: String) -> Self {
        DSError::Message(msg)
    }
}

impl std::fmt::Display for DSError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DSError::WinError(err) => write!(f, "{}", err),
            DSError::Other(err) => write!(f, "{}", err),
            DSError::Message(msg) => write!(f, "{}", msg),
        }
    }
}

impl Error for DSError {}

pub type DSResult<T> = Result<T, DSError>;

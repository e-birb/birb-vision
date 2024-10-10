#[derive(thiserror::Error, Debug)]
pub enum DeviceError {
    #[error("Device is not accessible in the requested mode")]
    NotAccessible,

    #[error("Invalid parameter")]
    InvalidParameter,

    #[error("Operation is not supported")]
    Unsupported,

    #[error("Functionality not implemented")]
    NotImplemented,

    #[error("Buffer overflow")]
    BufferOverflow,

    #[error("Call order error, this function cannot be called at this time")]
    CallOrderError,

    #[error("No data available")]
    NoDataAvailable,

    #[error("Timeout")]
    Timeout,

    #[error("Version mismatch")]
    VersionMismatch,

    #[error("Library load error")]
    LibraryLoadError,

    #[error("Input/output error: {0}")]
    IO(#[from] std::io::Error),

    #[error("Invalid Node ID")]
    InvalidNodeId,

    #[error("Unsupported Format")]
    UnsupportedFormat,

    //#[error("Error: {0}")]
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type DeviceResult<T = ()> = Result<T, DeviceError>;
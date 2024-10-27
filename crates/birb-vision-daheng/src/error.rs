use std::{error::Error, ffi::CStr, fmt::Display};

use daheng_sys::{v1, SDK};


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GxErrorKind {
    Unspecified,
    TLLibNotFound,
    DeviceNotFound,
    DeviceOffline,
    InvalidParameter,
    InvalidHandle,
    InvalidCall,
    InvalidAccess,
    NeedMoreBuffer,
    InvalidType,
    OutOfRange,
    NotImplemented,
    InterfaceNotInitialized,
    Timeout,
    Unknown(i32),
}

impl GxErrorKind {
    pub fn from_v1_code(code: i32) -> Option<Self> {
        use GxErrorKind::*;

        let k = match code {
            v1::GX_STATUS_LIST_GX_STATUS_SUCCESS => return None,
            v1::GX_STATUS_LIST_GX_STATUS_ERROR => Unspecified,
            v1::GX_STATUS_LIST_GX_STATUS_NOT_FOUND_TL => TLLibNotFound,
            v1::GX_STATUS_LIST_GX_STATUS_NOT_FOUND_DEVICE => DeviceNotFound,
            v1::GX_STATUS_LIST_GX_STATUS_OFFLINE => DeviceOffline,
            v1::GX_STATUS_LIST_GX_STATUS_INVALID_PARAMETER => InvalidParameter,
            v1::GX_STATUS_LIST_GX_STATUS_INVALID_HANDLE => InvalidHandle,
            v1::GX_STATUS_LIST_GX_STATUS_INVALID_CALL => InvalidCall,
            v1::GX_STATUS_LIST_GX_STATUS_INVALID_ACCESS => InvalidAccess,
            v1::GX_STATUS_LIST_GX_STATUS_NEED_MORE_BUFFER => NeedMoreBuffer,
            v1::GX_STATUS_LIST_GX_STATUS_ERROR_TYPE => InvalidType,
            v1::GX_STATUS_LIST_GX_STATUS_OUT_OF_RANGE => OutOfRange,
            v1::GX_STATUS_LIST_GX_STATUS_NOT_IMPLEMENTED => NotImplemented,
            v1::GX_STATUS_LIST_GX_STATUS_NOT_INIT_API => InterfaceNotInitialized,
            v1::GX_STATUS_LIST_GX_STATUS_TIMEOUT => Timeout,
            _ => Unknown(code),
        };

        Some(k)
    }

    pub fn from_v2_code(code: i32) -> Option<Self> {
        Self::from_v1_code(code)
    }
}

impl Display for GxErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use GxErrorKind::*;
        match self {
            Unspecified => write!(f, "unspecified error"),
            TLLibNotFound => write!(f, "TL library not found"),
            DeviceNotFound => write!(f, "device not found"),
            DeviceOffline => write!(f, "device offline"),
            InvalidParameter => write!(f, "invalid parameter"),
            InvalidHandle => write!(f, "invalid handle"),
            InvalidCall => write!(f, "invalid call"),
            InvalidAccess => write!(f, "invalid access"),
            NeedMoreBuffer => write!(f, "need more buffer"),
            InvalidType => write!(f, "invalid type"),
            OutOfRange => write!(f, "out of range"),
            NotImplemented => write!(f, "not implemented"),
            InterfaceNotInitialized => write!(f, "interface not initialized"),
            Timeout => write!(f, "timeout"),
            Unknown(code) => write!(f, "unknown error code {code:x}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GxError {
    pub kind: GxErrorKind,
    pub message: String,
}

impl GxError {
    pub fn result(sdk: &SDK, code: i32) -> Result<(), Self> {
        let Some(kind) = (match &sdk {
            SDK::V1(_) => GxErrorKind::from_v1_code(code),
            SDK::V2(_) => GxErrorKind::from_v2_code(code),
        }) else {
            return Ok(());
        };

        let mut last_code = 0;
        let mut message_len = 0;

        if let Some(e) = unsafe { match &sdk {
            SDK::V1(api) => GxErrorKind::from_v1_code(api.GXGetLastError(
                &mut last_code,
                std::ptr::null_mut(),
                &mut message_len,
            )),
            SDK::V2(api) => GxErrorKind::from_v2_code(api.GXGetLastError(
                &mut last_code,
                std::ptr::null_mut(),
                &mut message_len,
            )),
        } } {
            log::error!("failed to get last error message length: {e:?}");
            return Err(GxError {
                kind,
                message: String::new(),
            })
        }

        if last_code != code {
            // TODO maybe this could happen in a multi-thread environment, fins a way to make
            // the api only accessible with a mutex
            let last_kind = match &sdk {
                SDK::V1(_) => GxErrorKind::from_v1_code(last_code),
                SDK::V2(_) => GxErrorKind::from_v2_code(last_code),
            };
            log::error!("the error code given to DahengError::result ({code}, {kind:?}) does not corresponds to the last error code stored in the Daheng library during error message length query ({last_code}, {last_kind:?})");
            return Err(GxError {
                kind,
                message: String::new(),
            });
        }

        let mut buffer: Vec::<u8> = vec![0; message_len];

        if let Some(e) = GxErrorKind::from_v1_code(unsafe { match &sdk {
            SDK::V1(api) => api.GXGetLastError(
                &mut last_code,
                buffer.as_mut_ptr() as *mut i8,
                &mut message_len,
            ),
            SDK::V2(api) => api.GXGetLastError(
                &mut last_code,
                buffer.as_mut_ptr() as *mut i8,
                &mut message_len,
            ),
        } }) {
            log::error!("failed to get last error message: {e:?}");
            return Err(GxError {
                kind,
                message: String::new(),
            })
        }

        if last_code != code {
            // TODO maybe this could happen in a multi-thread environment, fins a way to make
            // the api only accessible with a mutex
            let last_kind = match &sdk {
                SDK::V1(_) => GxErrorKind::from_v1_code(last_code),
                SDK::V2(_) => GxErrorKind::from_v2_code(last_code),
            };
            log::error!("the error code given to DahengError::result ({code}, {kind:?}) does not corresponds to the last error code stored in the Daheng library during error message read ({last_code}, {last_kind:?})");
            return Err(GxError {
                kind,
                message: String::new(),
            });
        }


        let message = &buffer[0..message_len];

        let message = match CStr::from_bytes_until_nul(message) {
            Ok(message) => message,
            Err(err) => {
                log::error!("failed to decode the error message: {err}");
                return Err(GxError {
                    kind,
                    message: String::new(),
                });
            }
        };

        return Err(GxError {
            kind,
            message: message.to_string_lossy().into_owned(),
        });
    }
}

impl Display for GxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.kind, self.message)
    }
}

impl Error for GxError {}

#[derive(thiserror::Error, Debug)]
pub enum DahengError {
    #[error("{0}")]
    GxError(#[from] GxError),
    #[error("{0}")]
    Other(#[from] anyhow::Error),
}
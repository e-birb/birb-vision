
use std::error::Error;

use icube_sdk_sys::{v1, v2};

#[allow(non_camel_case_types)]
//#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Debug)]
pub enum iCubeError {
    /// Generic error
    Error,

    IF_NOT_OPEN,
    WRONG_PARAM,
    OUT_OF_MEMORY,
    ALREADY_DONE,
    WRONG_CLOCK_VAL,
    COM_LIB_INIT,
    NOT_IF_STARTED,
    WRONG_ROI_ID,
    IF_NOT_ENABLED,
    COLOR_CAM_ONLY,
    DRIVER_VERSION,
    D3D_INIT,
    BAD_POINTER,
    ERROR_FILE_SIZE,
    RECONNECTION_ACTIVE,
    USB_REQUEST_FAIL,
    RESOURCE_IN_USE,
    DEVICE_GONE,
    DLL_MISMATCH,
    WRONG_FW_VERSION,
    NO_RGB_CALLBACK,
    NO_USB30_CAMERA,
    ERR_FIX_RELATION,
    CRC_CONFIG_DATA,
    CONFIG_DATA,
    ERR_START_PNP,
    INVALID_CAM_TYPE,
    NOT_IF_STREAMING,
    USB_STARTUP,

    /// Unknown error
    ///
    /// This variant is used when the error code is not recognized.
    Unknown(u8),

    Unimplemented,
    Other(Box<dyn Error>),
}

impl iCubeError {
    pub fn result_from_code_v2(code: i32) -> Result<(), Self> {
        use v2::*;

        if code == IC_SUCCESS {
            return Ok(());
        }

        let e = match code {
            IC_ERROR => Self::Error,
            IC_IF_NOT_OPEN => Self::IF_NOT_OPEN,
            IC_WRONG_PARAM => Self::WRONG_PARAM,
            IC_OUT_OF_MEMORY => Self::OUT_OF_MEMORY,
            IC_ALREADY_DONE => Self::ALREADY_DONE,
            IC_WRONG_CLOCK_VAL => Self::WRONG_CLOCK_VAL,
            IC_COM_LIB_INIT => Self::COM_LIB_INIT,
            IC_NOT_IF_STARTED => Self::NOT_IF_STARTED,
            IC_WRONG_ROI_ID => Self::WRONG_ROI_ID,
            IC_IF_NOT_ENABLED => Self::IF_NOT_ENABLED,
            IC_COLOR_CAM_ONLY => Self::COLOR_CAM_ONLY,
            IC_DRIVER_VERSION => Self::DRIVER_VERSION,
            IC_D3D_INIT => Self::D3D_INIT,
            IC_BAD_POINTER => Self::BAD_POINTER,
            IC_ERROR_FILE_SIZE => Self::ERROR_FILE_SIZE,
            IC_RECONNECTION_ACTIVE => Self::RECONNECTION_ACTIVE,
            IC_USB_REQUEST_FAIL => Self::USB_REQUEST_FAIL,
            IC_RESOURCE_IN_USE => Self::RESOURCE_IN_USE,
            IC_DEVICE_GONE => Self::DEVICE_GONE,
            IC_DLL_MISMATCH => Self::DLL_MISMATCH,
            IC_WRONG_FW_VERSION => Self::WRONG_FW_VERSION,
            IC_NO_RGB_CALLBACK => Self::NO_RGB_CALLBACK,
            IC_NO_USB30_CAMERA => Self::NO_USB30_CAMERA,
            IC_ERR_FIX_RELATION => Self::ERR_FIX_RELATION,
            IC_CRC_CONFIG_DATA => Self::CRC_CONFIG_DATA,
            IC_CONFIG_DATA => Self::CONFIG_DATA,
            IC_ERR_START_PNP => Self::ERR_START_PNP,
            IC_INVALID_CAM_TYPE => Self::INVALID_CAM_TYPE,
            IC_NOT_IF_STREAMING => Self::NOT_IF_STREAMING,
            IC_USB_STARTUP => Self::USB_STARTUP,
            _ => Self::Unknown(code as u8),
        };

        Err(e)
    }

    pub fn result_from_code_v1(code: i32) -> Result<(), Self> {
        use v1::*;

        if code == IC_SUCCESS {
            return Ok(());
        }

        let e = match code {
            IC_ERROR => Self::Error,
            IC_IF_NOT_OPEN => Self::IF_NOT_OPEN,
            IC_WRONG_PARAM => Self::WRONG_PARAM,
            IC_OUT_OF_MEMORY => Self::OUT_OF_MEMORY,
            IC_ALREADY_DONE => Self::ALREADY_DONE,
            IC_WRONG_CLOCK_VAL => Self::WRONG_CLOCK_VAL,
            IC_COM_LIB_INIT => Self::COM_LIB_INIT,
            IC_NOT_IF_STARTED => Self::NOT_IF_STARTED,
            _ => Self::Unknown(code as u8),
        };

        Err(e)
    }
}

impl std::fmt::Display for iCubeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
         // TODO check, written by copilot
        let msg = match self {
            Self::Error => "Generic error",
            Self::IF_NOT_OPEN => "Interface not open",
            Self::WRONG_PARAM => "Wrong parameter",
            Self::OUT_OF_MEMORY => "Out of memory",
            Self::ALREADY_DONE => "Already done",
            Self::WRONG_CLOCK_VAL => "Wrong clock value",
            Self::COM_LIB_INIT => "COM library initialization",
            Self::NOT_IF_STARTED => "Interface not started",
            Self::WRONG_ROI_ID => "Wrong ROI ID",
            Self::IF_NOT_ENABLED => "Interface not enabled",
            Self::COLOR_CAM_ONLY => "Color camera only",
            Self::DRIVER_VERSION => "Driver version",
            Self::D3D_INIT => "Direct3D initialization",
            Self::BAD_POINTER => "Bad pointer",
            Self::ERROR_FILE_SIZE => "Error file size",
            Self::RECONNECTION_ACTIVE => "Reconnection active",
            Self::USB_REQUEST_FAIL => "USB request fail",
            Self::RESOURCE_IN_USE => "Resource in use",
            Self::DEVICE_GONE => "Device gone",
            Self::DLL_MISMATCH => "DLL mismatch",
            Self::WRONG_FW_VERSION => "Wrong firmware version",
            Self::NO_RGB_CALLBACK => "No RGB callback",
            Self::NO_USB30_CAMERA => "No USB 3.0 camera",
            Self::ERR_FIX_RELATION => "Error fix relation",
            Self::CRC_CONFIG_DATA => "CRC config data",
            Self::CONFIG_DATA => "Config data",
            Self::ERR_START_PNP => "Error start plug and play",
            Self::INVALID_CAM_TYPE => "Invalid camera type",
            Self::NOT_IF_STREAMING => "Interface not streaming",
            Self::USB_STARTUP => "USB startup",
            Self::Unknown(code) => return write!(f, "Unknown error code: {}", code),
            Self::Unimplemented => "Unimplemented",
            Self::Other(e) => return write!(f, "Other error: {}", e),
        };

        write!(f, "{}", msg)
    }
}

impl Error for iCubeError {}

pub(crate) trait IntoICubeResult {
    fn v1_result(self) -> Result<(), iCubeError>;
    fn v2_result(self) -> Result<(), iCubeError>;
}

impl IntoICubeResult for i32 {
    fn v1_result(self) -> Result<(), iCubeError> {
        iCubeError::result_from_code_v1(self)
    }

    fn v2_result(self) -> Result<(), iCubeError> {
        iCubeError::result_from_code_v2(self)
    }
}

impl From<Box<dyn Error>> for iCubeError {
    fn from(e: Box<dyn Error>) -> Self {
        Self::Other(e)
    }
}
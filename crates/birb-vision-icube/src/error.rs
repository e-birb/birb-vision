
use std::error::Error;

use icube_sdk_sys::ffi;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(i32)]
pub enum iCubeError {
    /// Generic error
    Error = ffi::IC_ERROR as _,

    IF_NOT_OPEN = ffi::IC_IF_NOT_OPEN,
    WRONG_PARAM = ffi::IC_WRONG_PARAM,
    OUT_OF_MEMORY = ffi::IC_OUT_OF_MEMORY,
    ALREADY_DONE = ffi::IC_ALREADY_DONE,
    WRONG_CLOCK_VAL = ffi::IC_WRONG_CLOCK_VAL,
    COM_LIB_INIT = ffi::IC_COM_LIB_INIT,
    NOT_IF_STARTED = ffi::IC_NOT_IF_STARTED,
    WRONG_ROI_ID = ffi::IC_WRONG_ROI_ID,
    IF_NOT_ENABLED = ffi::IC_IF_NOT_ENABLED,
    COLOR_CAM_ONLY = ffi::IC_COLOR_CAM_ONLY,
    DRIVER_VERSION = ffi::IC_DRIVER_VERSION,
    D3D_INIT = ffi::IC_D3D_INIT,
    BAD_POINTER = ffi::IC_BAD_POINTER,
    ERROR_FILE_SIZE = ffi::IC_ERROR_FILE_SIZE,
    RECONNECTION_ACTIVE = ffi::IC_RECONNECTION_ACTIVE,
    USB_REQUEST_FAIL = ffi::IC_USB_REQUEST_FAIL,
    RESOURCE_IN_USE = ffi::IC_RESOURCE_IN_USE,
    DEVICE_GONE = ffi::IC_DEVICE_GONE,
    DLL_MISMATCH = ffi::IC_DLL_MISMATCH,
    WRONG_FW_VERSION = ffi::IC_WRONG_FW_VERSION,
    NO_RGB_CALLBACK = ffi::IC_NO_RGB_CALLBACK,
    NO_USB30_CAMERA = ffi::IC_NO_USB30_CAMERA,
    ERR_FIX_RELATION = ffi::IC_ERR_FIX_RELATION,
    CRC_CONFIG_DATA = ffi::IC_CRC_CONFIG_DATA,
    CONFIG_DATA = ffi::IC_CONFIG_DATA,
    ERR_START_PNP = ffi::IC_ERR_START_PNP,
    INVALID_CAM_TYPE = ffi::IC_INVALID_CAM_TYPE,
    NOT_IF_STREAMING = ffi::IC_NOT_IF_STREAMING,
    USB_STARTUP = ffi::IC_USB_STARTUP,

    /// Unknown error
    ///
    /// This variant is used when the error code is not recognized.
    Unknown(u8) = 0x00FFFFFF - 1, // TODO check this, maybe use a better choice
}

impl iCubeError {
    pub fn from_code(code: i32) -> Self {
        const IC_ERROR: i32 = ffi::IC_ERROR as _;

        match code {
            IC_ERROR => Self::Error,
            ffi::IC_IF_NOT_OPEN => Self::IF_NOT_OPEN,
            ffi::IC_WRONG_PARAM => Self::WRONG_PARAM,
            ffi::IC_OUT_OF_MEMORY => Self::OUT_OF_MEMORY,
            ffi::IC_ALREADY_DONE => Self::ALREADY_DONE,
            ffi::IC_WRONG_CLOCK_VAL => Self::WRONG_CLOCK_VAL,
            ffi::IC_COM_LIB_INIT => Self::COM_LIB_INIT,
            ffi::IC_NOT_IF_STARTED => Self::NOT_IF_STARTED,
            ffi::IC_WRONG_ROI_ID => Self::WRONG_ROI_ID,
            ffi::IC_IF_NOT_ENABLED => Self::IF_NOT_ENABLED,
            ffi::IC_COLOR_CAM_ONLY => Self::COLOR_CAM_ONLY,
            ffi::IC_DRIVER_VERSION => Self::DRIVER_VERSION,
            ffi::IC_D3D_INIT => Self::D3D_INIT,
            ffi::IC_BAD_POINTER => Self::BAD_POINTER,
            ffi::IC_ERROR_FILE_SIZE => Self::ERROR_FILE_SIZE,
            ffi::IC_RECONNECTION_ACTIVE => Self::RECONNECTION_ACTIVE,
            ffi::IC_USB_REQUEST_FAIL => Self::USB_REQUEST_FAIL,
            ffi::IC_RESOURCE_IN_USE => Self::RESOURCE_IN_USE,
            ffi::IC_DEVICE_GONE => Self::DEVICE_GONE,
            ffi::IC_DLL_MISMATCH => Self::DLL_MISMATCH,
            ffi::IC_WRONG_FW_VERSION => Self::WRONG_FW_VERSION,
            ffi::IC_NO_RGB_CALLBACK => Self::NO_RGB_CALLBACK,
            ffi::IC_NO_USB30_CAMERA => Self::NO_USB30_CAMERA,
            ffi::IC_ERR_FIX_RELATION => Self::ERR_FIX_RELATION,
            ffi::IC_CRC_CONFIG_DATA => Self::CRC_CONFIG_DATA,
            ffi::IC_CONFIG_DATA => Self::CONFIG_DATA,
            ffi::IC_ERR_START_PNP => Self::ERR_START_PNP,
            ffi::IC_INVALID_CAM_TYPE => Self::INVALID_CAM_TYPE,
            ffi::IC_NOT_IF_STREAMING => Self::NOT_IF_STREAMING,
            ffi::IC_USB_STARTUP => Self::USB_STARTUP,
            _ => Self::Unknown(code as u8),
        }
    }

    pub fn sdk_code(&self) -> i32 {
        match self {
            Self::Error => ffi::IC_ERROR as _,
            Self::IF_NOT_OPEN => ffi::IC_IF_NOT_OPEN,
            Self::WRONG_PARAM => ffi::IC_WRONG_PARAM,
            Self::OUT_OF_MEMORY => ffi::IC_OUT_OF_MEMORY,
            Self::ALREADY_DONE => ffi::IC_ALREADY_DONE,
            Self::WRONG_CLOCK_VAL => ffi::IC_WRONG_CLOCK_VAL,
            Self::COM_LIB_INIT => ffi::IC_COM_LIB_INIT,
            Self::NOT_IF_STARTED => ffi::IC_NOT_IF_STARTED,
            Self::WRONG_ROI_ID => ffi::IC_WRONG_ROI_ID,
            Self::IF_NOT_ENABLED => ffi::IC_IF_NOT_ENABLED,
            Self::COLOR_CAM_ONLY => ffi::IC_COLOR_CAM_ONLY,
            Self::DRIVER_VERSION => ffi::IC_DRIVER_VERSION,
            Self::D3D_INIT => ffi::IC_D3D_INIT,
            Self::BAD_POINTER => ffi::IC_BAD_POINTER,
            Self::ERROR_FILE_SIZE => ffi::IC_ERROR_FILE_SIZE,
            Self::RECONNECTION_ACTIVE => ffi::IC_RECONNECTION_ACTIVE,
            Self::USB_REQUEST_FAIL => ffi::IC_USB_REQUEST_FAIL,
            Self::RESOURCE_IN_USE => ffi::IC_RESOURCE_IN_USE,
            Self::DEVICE_GONE => ffi::IC_DEVICE_GONE,
            Self::DLL_MISMATCH => ffi::IC_DLL_MISMATCH,
            Self::WRONG_FW_VERSION => ffi::IC_WRONG_FW_VERSION,
            Self::NO_RGB_CALLBACK => ffi::IC_NO_RGB_CALLBACK,
            Self::NO_USB30_CAMERA => ffi::IC_NO_USB30_CAMERA,
            Self::ERR_FIX_RELATION => ffi::IC_ERR_FIX_RELATION,
            Self::CRC_CONFIG_DATA => ffi::IC_CRC_CONFIG_DATA,
            Self::CONFIG_DATA => ffi::IC_CONFIG_DATA,
            Self::ERR_START_PNP => ffi::IC_ERR_START_PNP,
            Self::INVALID_CAM_TYPE => ffi::IC_INVALID_CAM_TYPE,
            Self::NOT_IF_STREAMING => ffi::IC_NOT_IF_STREAMING,
            Self::USB_STARTUP => ffi::IC_USB_STARTUP,
            Self::Unknown(code) => *code as i32,
        }
    }

    pub fn result_from_code(code: i32) -> Result<(), Self> {
        if code == ffi::IC_SUCCESS as _ {
            Ok(())
        } else {
            Err(Self::from_code(code))
        }
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
        };

        write!(f, "{}", msg)
    }
}

impl Error for iCubeError {}
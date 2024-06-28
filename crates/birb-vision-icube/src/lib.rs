use std::{marker::PhantomData, ops::{Deref, RangeInclusive}, sync::Mutex};

use icube_sdk_sys::ffi;

mod error;
mod ctx;

pub use error::*;
pub use ctx::*;

macro_rules! sdk {
    ($name:ident) => {
        icube_sdk_sys::ffi::$name
            .expect(format!("iCube function {} not found", stringify!($name)).as_str())
    };
}

macro_rules! ic_try {
    ($name:ident($($args:expr),*$(,)?)) => {
        {
            let result_code = unsafe { sdk!($name)($($args),*) };
            iCubeError::result_from_code(result_code)
        }
    };
}

pub(crate) use sdk;
pub(crate) use ic_try;

#[allow(non_camel_case_types)]
pub struct iCubeDevice {
    handle: DeviceHandle,

    callback: Box<OptionalCallbackWrapper>,

    /// This is a marker to ensure that the device is not [`Send`] nor [`Sync`].
    _marker: PhantomData<*const ()>,
}

impl iCubeDevice {
    /// TODO ???
    ///
    /// This method corresponds to the [`ICubeSDK_Open`] function.
    ///
    /// [`ICubeSDK_Open`]: icube_sdk_sys::ffi::ICubeSDK_Open
    pub fn get_size(&self) -> Result<(i32, i32), iCubeError> {
        let mut width = 0;
        let mut height = 0;

        ic_try!(ICubeSDK_GetSize(*self.handle, &mut width, &mut height)).map(|_| (width, height))
    }

    pub fn set_callback(&self, callback: Box<Callback>) {
        *self.callback.lock().unwrap() = Some(callback);
    }

    pub fn start_video_stream(&self, preview: bool, callback: bool) -> Result<(), iCubeError> {
        ic_try!(ICubeSDK_Start(
            *self.handle,
            std::ptr::null_mut(),
            preview as _,
            callback as _,
        ))
    }

    pub fn is_video_streaming(&self) -> bool {
        let status = unsafe { sdk!(ICubeSDK_IsStarted)(*self.handle) };
        match status as _ {
            ffi::ON => true,
            ffi::OFF => false,
            _ => panic!("unexpected return value from ICubeSDK_IsStarted: {}", status),
        }
    }

    pub fn get_frame_size(&self) -> Result<(i32, i32), iCubeError> {
        let mut width = 0;
        let mut height = 0;
        ic_try!(ICubeSDK_GetSize(*self.handle, &mut width, &mut height))?;
        Ok((width, height))
    }

    pub fn get_name(&self) -> Result<String, iCubeError> {
        let mut name = [0i8; ffi::NETCAM_NAME_LENGTH as usize];
        ic_try!(ICubeSDK_GetName(*self.handle, name.as_mut_ptr())).map(|_| {
            let name_len = name.iter().position(|&c| c == 0).unwrap_or(name.len());
            name[..name_len].iter().map(|&c| c as u8 as char).collect::<String>()
        })
    }

    pub fn get_broken_frames(&self) -> Result<i32, iCubeError> {
        let mut count = 0;
        ic_try!(ICubeSDK_GetBrokenFrames(*self.handle, &mut count))?;
        Ok(count)
    }

    pub fn get_good_frames(&self) -> Result<i32, iCubeError> {
        let mut count = 0;
        ic_try!(ICubeSDK_GetGoodFrames(*self.handle, &mut count))?;
        Ok(count)
    }

    pub fn set_display_mode(
        &self,
        mode: DisplayMode,
        properties: DisplayProperty,
    ) -> Result<(), iCubeError> {
        ic_try!(ICubeSDK_SetDisplayMode(
            *self.handle,
            mode as _,
            properties.into_sdk_struct(),
        ))
    }

    pub fn get_version(&self) -> Result<String, iCubeError> {
        let mut version = [0i8; ffi::NETCAM_VERSION_LENGTH as usize];
        unsafe {
            sdk!(ICubeSDK_GetVersion)(*self.handle, version.as_mut_ptr());
        }

        let version_len = version.iter().position(|&c| c == 0).unwrap_or(version.len());
        Ok(version[..version_len].iter().map(|&c| c as u8 as char).collect::<String>())
    }

    pub fn get_firmware_version(&self) -> Result<String, iCubeError> {
        let mut version = [0i8; ffi::NETCAM_VERSION_LENGTH as usize];
        ic_try!(ICubeSDK_GetFWVersion(*self.handle, version.as_mut_ptr()))?;
        let version_len = version.iter().position(|&c| c == 0).unwrap_or(version.len());
        Ok(version[..version_len].iter().map(|&c| c as u8 as char).collect::<String>())
    }

    pub fn get_serial_number(&self) -> Result<String, iCubeError> {
        let mut serial = [0i8; ffi::NETCAM_SERIAL_LENGTH as usize];
        ic_try!(ICubeSDK_GetSerialNum(*self.handle, serial.as_mut_ptr()))?;
        let serial_len = serial.iter().position(|&c| c == 0).unwrap_or(serial.len());
        Ok(serial[..serial_len].iter().map(|&c| c as u8 as char).collect::<String>())
    }

    pub fn get_fpga_version(&self) -> Result<String, iCubeError> {
        let mut version = [0i8; ffi::NETCAM_VERSION_LENGTH as usize];
        ic_try!(ICubeSDK_GetFPGAVersion(*self.handle, version.as_mut_ptr()))?;
        let version_len = version.iter().position(|&c| c == 0).unwrap_or(version.len());
        Ok(version[..version_len].iter().map(|&c| c as u8 as char).collect::<String>())
    }

    pub fn set_roi_property(&self, roi: RoiProperty) -> Result<(), iCubeError> {
        ic_try!(ICubeSDK_SetResolution(*self.handle, &mut RoiProperty::to_sdk_struct(&roi)))
    }

    pub fn get_resolution(&self) -> Result<RoiProperty, iCubeError> {
        let mut roi = ffi::ROI_PROPERTY {
            bEnabled: 0,
            nXPos: 0,
            nYPos: 0,
            nXRes: 0,
            nYRes: 0,
        };
        ic_try!(ICubeSDK_GetResolution(*self.handle, &mut roi))?;
        Ok(RoiProperty::from_sdk_struct(&roi))
    }

    pub fn get_resolution_range(&self) -> Result<RoiRange, iCubeError> {
        let mut roi_range = ffi::ROI_RANGE_PROPERTY {
            nXMin: 0,
            nXMax: 0,
            nYMin: 0,
            nYMax: 0,
        };
        ic_try!(ICubeSDK_GetResolutionRange(*self.handle, &mut roi_range))?;
        Ok(RoiRange {
            width_range: roi_range.nXMin..=roi_range.nXMax,
            height_range: roi_range.nYMin..=roi_range.nYMax,
        })
    }

    // TODO incomprehensible!
    //pub fn set_resolution_param(&self, roi: &RoiProperty) -> Result<(), iCubeError> {
    //    ic_try!(ICubeSDK_SetResolutionParam(*self.handle, &mut RoiProperty::to_sdk_struct(roi)))
    //}

    pub fn set_mode(&self, resolution_mode: &ResolutionMode) -> Result<(), iCubeError> {
        ic_try!(ICubeSDK_SetMode(*self.handle, resolution_mode.into_sdk_value()))
    }

    pub fn get_mode(&self) -> Result<ResolutionMode, iCubeError> {
        let mut mode = 0;
        ic_try!(ICubeSDK_GetMode(*self.handle, &mut mode))?;
        Ok(ResolutionMode::from_sdk_value(mode))
    }

    pub fn get_mode_list(&self) -> Result<Vec<ResolutionMode>, iCubeError> {
        let mut modes = vec![0; 255];
        let mut count = 0;
        ic_try!(ICubeSDK_GetModeList(*self.handle, modes.as_mut_ptr(), &mut count))?;
        modes.truncate(count as usize);
        Ok(modes.into_iter().map(ResolutionMode::from_sdk_value).collect())
    }

    pub fn set_bin_skip(&self, mode: BinOrSkipMode, param: BinSkipParameter) -> Result<(), iCubeError> {
        ic_try!(ICubeSDK_SetBinSkip(*self.handle, param.into_sdk_value(), mode.into_sdk_value()))
    }

    pub fn get_bin_skip(&self, mode: BinOrSkipMode) -> Result<BinSkipParameter, iCubeError> {
        let mut param = 0;
        ic_try!(ICubeSDK_GetBinSkip(*self.handle, &mut param, mode.into_sdk_value()))?;
        Ok(BinSkipParameter::from_sdk_value(param))
    }

    pub fn get_bin_skip_list(&self, mode: BinOrSkipMode) -> Result<Vec<BinSkipParameter>, iCubeError> {
        let mut params = vec![0; 255];
        let mut count = 0;
        ic_try!(ICubeSDK_GetBinSkipList(*self.handle, mode.into_sdk_value(), params.as_mut_ptr(), &mut count))?;
        params.truncate(count as usize);
        Ok(params.into_iter().map(BinSkipParameter::from_sdk_value).collect())
    }

    pub fn save_to_file(&self, path: &str) -> Result<(), iCubeError> {
        let path = std::ffi::CString::new(path).expect("Failed to convert path to CString");
        ic_try!(ICubeSDK_SaveToFile(*self.handle, path.as_ptr()))
    }

    // TODO
    //pub fn save_avi(&self, path: &str) -> Result<(), iCubeError> {
    //    let path = std::ffi::CString::new(path).expect("Failed to convert path to CString");
    //    ic_try!(ICubeSDK_SaveAvi(*self.handle, path.as_ptr()))
    //}

    pub fn set_trigger(&self, mode: TriggerMode) -> Result<(), iCubeError> {
        ic_try!(ICubeSDK_SetTrigger(*self.handle, mode.into_sdk_value()))
    }

    pub fn get_trigger(&self) -> Result<TriggerMode, iCubeError> {
        let mut mode = 0;
        ic_try!(ICubeSDK_GetTrigger(*self.handle, &mut mode))?;
        Ok(TriggerMode::from_sdk_value(mode))
    }

    pub fn set_cam_parameter(&self, id: ParameterID, value: u32) -> Result<(), iCubeError> {
        ic_try!(ICubeSDK_SetCamParameter(*self.handle, id as u32 as _, value))
    }

    pub fn get_cam_parameter(&self, id: ParameterID) -> Result<u32, iCubeError> {
        let mut value = 0;
        ic_try!(ICubeSDK_GetCamParameter(*self.handle, id as u32 as _, &mut value))?;
        Ok(value)
    }

    pub fn get_cam_parameter_range(&self, id: ParameterID) -> Result<ParameterProperty, iCubeError> {
        let mut prop = unsafe { std::mem::zeroed() };
        ic_try!(ICubeSDK_GetCamParameterRange(*self.handle, id as u32 as _, &mut prop))?;
        Ok(ParameterProperty::from_sdk_struct(&prop))
    }

    pub fn set_exposure(&self, value: f32) -> Result<(), iCubeError> {
        ic_try!(ICubeSDK_SetExposure(*self.handle, value))
    }

    pub fn get_exposure(&self) -> Result<f32, iCubeError> {
        let mut value = 0.0;
        ic_try!(ICubeSDK_GetExposure(*self.handle, &mut value))?;
        Ok(value)
    }

    pub fn get_exposure_range(&self) -> Result<RangeInclusive<f32>, iCubeError> {
        let mut prop = unsafe { std::mem::zeroed() };
        ic_try!(ICubeSDK_GetExposureRange(*self.handle, &mut prop))?;
        Ok(prop.nMin..=prop.nMax)
    }

    pub fn get_param_auto_supported(&self, id: ParameterID) -> Result<bool, iCubeError> {
        let mut auto = 0;
        ic_try!(ICubeSDK_GetParamAuto(*self.handle, id as u32 as _, &mut auto))?;
        Ok(auto != 0)
    }

    pub fn set_param_auto(&self, id: ParameterID, value: bool) -> Result<(), iCubeError> {
        ic_try!(ICubeSDK_SetParamAuto(*self.handle, id as u32 as _, value as _))
    }

    // TODO missing?
    //pub fn set_param_default(&self, id: ParameterID) -> Result<(), iCubeError> {
    //    ic_try!(ICubeSDK_SetParamDef(*self.handle, id as u32 as _))
    //}

    pub fn set_param_one_push(&self, id: ParameterID) -> Result<(), iCubeError> {
        ic_try!(ICubeSDK_SetParamOnePush(*self.handle, id as u32 as _))
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct DeviceHandle {
    index: i32,
}

impl Drop for DeviceHandle {
    fn drop(&mut self) {
        unsafe { sdk!(ICubeSDK_Close)(self.index); };
    }
}

impl Deref for DeviceHandle {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.index
    }
}

pub type Callback = dyn Fn(CallbackEventType) + Send + Sync;
type OptionalCallbackWrapper = Mutex<Option<Box<Callback>>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(non_camel_case_types)]
pub enum CallbackEventType<'a> {
    NEW_FRAME(&'a [u8]),
    DEV_DISCONNECTED,
    DEV_RECONNECTED,
    USB_TRANSFER_FAILED,
    Unknown(i32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(non_camel_case_types)]
#[repr(u32)]
pub enum DisplayMode {
    DISPLAY_NORMAL = ffi::DISPLAY_NORMAL,
    DISPLAY_FIT_TO_WINDOW = ffi::DISPLAY_FIT_TO_WINDOW,
    DISPLAY_RECT = ffi::DISPLAY_RECT,
}

pub struct DisplayProperty {
    pub top: i32,
    pub bottom: i32,
    pub left: i32,
    pub right: i32,
}

impl DisplayProperty {
    pub fn into_sdk_struct(&self) -> ffi::DISP_PROPERTY {
        ffi::DISP_PROPERTY {
            top: self.top,
            bottom: self.bottom,
            left: self.left,
            right: self.right,
        }
    }
}

pub struct RoiProperty {
    pub enabled: bool,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl RoiProperty {
    fn to_sdk_struct(&self) -> ffi::ROI_PROPERTY {
        ffi::ROI_PROPERTY {
            bEnabled: self.enabled as _,
            nXPos: self.x,
            nYPos: self.y,
            nXRes: self.width,
            nYRes: self.height,
        }
    }

    fn from_sdk_struct(sdk_struct: &ffi::ROI_PROPERTY) -> Self {
        Self {
            enabled: sdk_struct.bEnabled != 0,
            x: sdk_struct.nXPos,
            y: sdk_struct.nYPos,
            width: sdk_struct.nXRes,
            height: sdk_struct.nYRes,
        }
    }
}

pub struct RoiRange {
    pub width_range: RangeInclusive<i32>,
    pub height_range: RangeInclusive<i32>,
}

impl RoiRange {
    pub fn to_sdk_struct(&self) -> ffi::ROI_RANGE_PROPERTY {
        ffi::ROI_RANGE_PROPERTY {
            nXMin: *self.width_range.start(),
            nXMax: *self.width_range.end(),
            nYMin: *self.height_range.start(),
            nYMax: *self.height_range.end(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(non_camel_case_types)]
#[repr(u32)]
pub enum ResolutionMode {
    MODE_320x240,
    MODE_640x480,
    MODE_752x480,
    MODE_800x600,
    MODE_1024x768,
    MODE_1280x1024,
    MODE_1600x1200,
    MODE_2048x1536,
    MODE_2592x1944,
    MODE_3840x2748,
    MODE_1920x1200,
    Unknown(i32),
}

impl ResolutionMode {
    pub fn into_sdk_value(self) -> i32 {
        match self {
            Self::MODE_320x240 => ffi::MODE_320x240 as _,
            Self::MODE_640x480 => ffi::MODE_640x480 as _,
            Self::MODE_752x480 => ffi::MODE_752x480 as _,
            Self::MODE_800x600 => ffi::MODE_800x600 as _,
            Self::MODE_1024x768 => ffi::MODE_1024x768 as _,
            Self::MODE_1280x1024 => ffi::MODE_1280x1024 as _,
            Self::MODE_1600x1200 => ffi::MODE_1600x1200 as _,
            Self::MODE_2048x1536 => ffi::MODE_2048x1536 as _,
            Self::MODE_2592x1944 => ffi::MODE_2592x1944 as _,
            Self::MODE_3840x2748 => ffi::MODE_3840x2748 as _,
            Self::MODE_1920x1200 => ffi::MODE_1920x1200 as _,
            Self::Unknown(v) => v,
        }
    }

    pub fn from_sdk_value(value: i32) -> Self {
        match value as _ {
            ffi::MODE_320x240 => Self::MODE_320x240,
            ffi::MODE_640x480 => Self::MODE_640x480,
            ffi::MODE_752x480 => Self::MODE_752x480,
            ffi::MODE_800x600 => Self::MODE_800x600,
            ffi::MODE_1024x768 => Self::MODE_1024x768,
            ffi::MODE_1280x1024 => Self::MODE_1280x1024,
            ffi::MODE_1600x1200 => Self::MODE_1600x1200,
            ffi::MODE_2048x1536 => Self::MODE_2048x1536,
            ffi::MODE_2592x1944 => Self::MODE_2592x1944,
            ffi::MODE_3840x2748 => Self::MODE_3840x2748,
            ffi::MODE_1920x1200 => Self::MODE_1920x1200,
            _ => Self::Unknown(value),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(non_camel_case_types)]
#[repr(u32)]
pub enum BinSkipParameter {
    BIN_SKIP_OFF = ffi::BIN_SKIP_OFF,
    BIN_SKIP_2ND_PIXEL = ffi::BIN_SKIP_2ND_PIXEL,
    BIN_SKIP_4TH_PIXEL = ffi::BIN_SKIP_4TH_PIXEL,
    Unknown(i32),
}

impl BinSkipParameter {
    pub fn into_sdk_value(self) -> i32 {
        match self {
            Self::BIN_SKIP_OFF => ffi::BIN_SKIP_OFF as _,
            Self::BIN_SKIP_2ND_PIXEL => ffi::BIN_SKIP_2ND_PIXEL as _,
            Self::BIN_SKIP_4TH_PIXEL => ffi::BIN_SKIP_4TH_PIXEL as _,
            Self::Unknown(v) => v,
        }
    }

    pub fn from_sdk_value(value: i32) -> Self {
        match value as _ {
            ffi::BIN_SKIP_OFF => Self::BIN_SKIP_OFF,
            ffi::BIN_SKIP_2ND_PIXEL => Self::BIN_SKIP_2ND_PIXEL,
            ffi::BIN_SKIP_4TH_PIXEL => Self::BIN_SKIP_4TH_PIXEL,
            _ => Self::Unknown(value),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(non_camel_case_types)]
#[repr(u32)]
pub enum BinOrSkipMode {
    Bin = ffi::MODE_BIN,
    Skip = ffi::MODE_SKIP,
}

impl BinOrSkipMode {
    pub fn into_sdk_value(self) -> i32 {
        match self {
            Self::Bin => ffi::MODE_BIN as _,
            Self::Skip => ffi::MODE_SKIP as _,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(non_camel_case_types)]
#[repr(u32)]
pub enum TriggerMode {
    TRIG_SW_START = ffi::TRIG_SW_START,
    TRIG_SW_DO = ffi::TRIG_SW_DO,
    TRIG_HW_START = ffi::TRIG_HW_START,
    TRIG_STOP = ffi::TRIG_STOP,
    TRIG_SW_START_2 = ffi::TRIG_SW_START_2,
    TRIG_HW_START_2 = ffi::TRIG_HW_START_2,
    Unknown(i32),
}

impl TriggerMode {
    pub fn into_sdk_value(self) -> i32 {
        match self {
            Self::TRIG_SW_START => ffi::TRIG_SW_START as _,
            Self::TRIG_SW_DO => ffi::TRIG_SW_DO as _,
            Self::TRIG_HW_START => ffi::TRIG_HW_START as _,
            Self::TRIG_STOP => ffi::TRIG_STOP as _,
            Self::TRIG_SW_START_2 => ffi::TRIG_SW_START_2 as _,
            Self::TRIG_HW_START_2 => ffi::TRIG_HW_START_2 as _,
            Self::Unknown(v) => v,
        }
    }

    pub fn from_sdk_value(value: i32) -> Self {
        match value as _ {
            ffi::TRIG_SW_START => Self::TRIG_SW_START,
            ffi::TRIG_SW_DO => Self::TRIG_SW_DO,
            ffi::TRIG_HW_START => Self::TRIG_HW_START,
            ffi::TRIG_STOP => Self::TRIG_STOP,
            ffi::TRIG_SW_START_2 => Self::TRIG_SW_START_2,
            ffi::TRIG_HW_START_2 => Self::TRIG_HW_START_2,
            _ => Self::Unknown(value),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(non_camel_case_types)]
#[repr(u32)]
pub enum ParameterID {
    BRIGHTNESS = ffi::REG_BRIGHTNESS,
    CONTRAST = ffi::REG_CONTRAST,
    GAMMA = ffi::REG_GAMMA,
    FLIPPED_V = ffi::REG_FLIPPED_V,
    FLIPPED_H = ffi::REG_FLIPPED_H,
    WHITE_BALANCE = ffi::REG_WHITE_BALANCE,
    EXPOSURE_TIME = ffi::REG_EXPOSURE_TIME,
    EXPOSURE_TARGET = ffi::REG_EXPOSURE_TARGET,
    RED = ffi::REG_RED,
    GREEN = ffi::REG_GREEN,
    BLUE = ffi::REG_BLUE,
    BLACKLEVEL = ffi::REG_BLACKLEVEL,
    GAIN = ffi::REG_GAIN,
    COLOR = ffi::REG_COLOR,
    PLL = ffi::REG_PLL,
    STROBE_LENGHT = ffi::REG_STROBE_LENGHT,
    STROBE_DELAY = ffi::REG_STROBE_DELAY,
    TRIGGER_DELAY = ffi::REG_TRIGGER_DELAY,
    SATURATION = ffi::REG_SATURATION,
    COLOR_ENHANCE = ffi::REG_COLOR_ENHANCE,
    TRIGGER_INVERT = ffi::REG_TRIGGER_INVERT,
    RECONNECTIONS = ffi::REG_RECONNECTIONS,
    MEASURE_FIELD_AE = ffi::REG_MEASURE_FIELD_AE,
    AVI_STATE = ffi::REG_AVI_STATE,
    FOCUS = ffi::REG_FOCUS,
    SHUTTER = ffi::REG_SHUTTER,
    ROI_ID = ffi::REG_ROI_ID,
    ROI_CYCLE = ffi::REG_ROI_CYCLE,
    DEFECT_COR = ffi::REG_DEFECT_COR,
    BAD_FRAME = ffi::REG_BAD_FRAME,
    GOOD_FRAME = ffi::REG_GOOD_FRAME,
    SW_TRIG_MODE = ffi::REG_SW_TRIG_MODE,
    ROI_FPGA_ONE_FRAME = ffi::REG_ROI_FPGA_ONE_FRAME,
    CALLBACK_BR_FRAMES = ffi::REG_CALLBACK_BR_FRAMES,
    FGPA_VBLANKING = ffi::REG_FGPA_VBLANKING,
    FGPA_HBLANKING = ffi::REG_FGPA_HBLANKING,
    FGPA_CLK_DIVIDER = ffi::REG_FGPA_CLK_DIVIDER,
    FGPA_ON_BOARD = ffi::REG_FGPA_ON_BOARD,
    SET_GPIO = ffi::REG_SET_GPIO,
    GET_GPIO = ffi::REG_GET_GPIO,
    SET_GPIO_MODE = ffi::REG_SET_GPIO_MODE,
    MASK_ROI_ID = ffi::REG_MASK_ROI_ID,
    RED_OFFSET = ffi::REG_RED_OFFSET,
    GREEN_OFFSET = ffi::REG_GREEN_OFFSET,
    BLUE_OFFSET = ffi::REG_BLUE_OFFSET,
    HUE = ffi::REG_HUE,
    COLOR_CORRECTION = ffi::REG_COLOR_CORRECTION,
    GAMMA_ENABLE = ffi::REG_GAMMA_ENABLE,
    GAMMA_MODE = ffi::REG_GAMMA_MODE,
    INVERT_PIXEL = ffi::REG_INVERT_PIXEL,
    TNR = ffi::REG_TNR,
    BAYER_CONVERSION = ffi::REG_BAYER_CONVERSION,
    COLOR_PROCESSING = ffi::REG_COLOR_PROCESSING,
    USB_SPEED = ffi::REG_USB_SPEED,
    DEVICE_SPEED = ffi::REG_DEVICE_SPEED,
    DATA_TRANSMISSION = ffi::REG_DATA_TRANSMISSION,
    SIGNIFICANT_BITS = ffi::REG_SIGNIFICANT_BITS,
    GRAPHIC_MODE = ffi::REG_GRAPHIC_MODE,
    EDGE_ENHANCEMENT = ffi::REG_EDGE_ENHANCEMENT,
    EDGE_ENHANCEMENT_GAIN = ffi::REG_EDGE_ENHANCEMENT_GAIN,
    SENSOR_STROBE = ffi::REG_SENSOR_STROBE,
    TRIG_TIMEOUT = ffi::REG_TRIG_TIMEOUT,
    PIPE_TIMEOUT_MODE = ffi::REG_PIPE_TIMEOUT_MODE,
    RESET_CAMERA = ffi::REG_RESET_CAMERA,
    DISCONNECTIONS = ffi::REG_DISCONNECTIONS,
    XACT_RECOVER_MODE = ffi::REG_XACT_RECOVER_MODE,
    RESET_TO_DEFAULT = ffi::REG_RESET_TO_DEFAULT,
    TRANSFER_BUF_CNT = ffi::REG_TRANSFER_BUF_CNT,
    SW_TRIG_WD_MODE = ffi::REG_SW_TRIG_WD_MODE,
    AQU_FRAMERATE = ffi::REG_AQU_FRAMERATE,
    SENSOR_STROBE_DELAY = ffi::REG_SENSOR_STROBE_DELAY,
    SENSOR_OVERLAPPED = ffi::REG_SENSOR_OVERLAPPED,
    TRIG_DELAY_DIVIDER = ffi::REG_TRIG_DELAY_DIVIDER,
}

/*
typedef struct {
	BOOL    bEnabled;
	BOOL    bAuto;
	BOOL    bOnePush;
	UINT    nDef;
	UINT    nMin;
	ULONG   nMax;	
}
*/

pub struct ParameterProperty {
    pub enabled: bool,
    pub auto: bool,
    pub one_push: bool,
    pub default: u32,
    pub min: u32,
    pub max: u32,
}

impl ParameterProperty {
    fn from_sdk_struct(sdk_struct: &ffi::PARAM_PROPERTY) -> Self {
        Self {
            enabled: sdk_struct.bEnabled != 0,
            auto: sdk_struct.bAuto != 0,
            one_push: sdk_struct.bOnePush != 0,
            default: sdk_struct.nDef,
            min: sdk_struct.nMin,
            max: sdk_struct.nMax,
        }
    }
}
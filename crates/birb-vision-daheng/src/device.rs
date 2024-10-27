use std::{ffi::{c_void, CStr, CString}, fmt::Debug};

use daheng_sys::{v1, v2};

use crate::{ctx::try_common, Ctx, DahengError, GxError};


pub struct Device {
    cx: Ctx,
    handle: *mut c_void,
}

impl Device {
    pub fn open(info: &DeviceInfo) -> Result<Self, DahengError> {
        let mut handle = std::ptr::null_mut();

        let cx = Ctx::new()?;

        GxError::result(cx.sdk(), match cx.sdk() {
            daheng_sys::SDK::V1(api) => unsafe {
                let content: CString = CString::new(info.serial_number()).unwrap();
                let mut open_param = v1::GX_OPEN_PARAM {
                    pszContent: content.as_ptr() as *mut i8,
                    openMode: v1::GX_OPEN_MODE_GX_OPEN_SN as i32,
                    accessMode: v1::GX_ACCESS_MODE_GX_ACCESS_EXCLUSIVE as i32,
                };
                api.GXOpenDevice(&mut open_param, &mut handle)
            },
            daheng_sys::SDK::V2(api) => unsafe {
                let content: CString = CString::new(info.serial_number()).unwrap();
                let mut open_param = v2::GX_OPEN_PARAM {
                    pszContent: content.as_ptr() as *mut i8,
                    openMode: v1::GX_OPEN_MODE_GX_OPEN_SN as i32,
                    accessMode: v1::GX_ACCESS_MODE_GX_ACCESS_EXCLUSIVE as i32,
                };
                api.GXOpenDevice(&mut open_param, &mut handle)
            },
        })?;

        Ok(Self {
            cx,
            handle,
        })
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        if let Err(e) = try_common!(self.cx.sdk() => GXCloseDevice(self.handle)) {
            log::error!("Failed to close Daheng device: {e}");
        }
    }
}

pub enum DeviceInfo {
    V1(v1::GX_DEVICE_BASE_INFO),
    V2(v2::GX_DEVICE_BASE_INFO),
}

impl DeviceInfo {
    pub fn vendor_name(&self) -> String {
        // TODO handle errors and maybe do not use to_string_lossy?
        match self {
            DeviceInfo::V1(info) => CStr::from_bytes_until_nul(&info.szVendorName.map(|v| v as u8)).unwrap().to_string_lossy().to_string(),
            DeviceInfo::V2(info) => CStr::from_bytes_until_nul(&info.szVendorName.map(|v| v as u8)).unwrap().to_string_lossy().to_string(),
        }
    }
    pub fn model_name(&self) -> String {
        match self {
            DeviceInfo::V1(info) => CStr::from_bytes_until_nul(&info.szModelName.map(|v| v as u8)).unwrap().to_string_lossy().to_string(),
            DeviceInfo::V2(info) => CStr::from_bytes_until_nul(&info.szModelName.map(|v| v as u8)).unwrap().to_string_lossy().to_string(),
        }
    }
    pub fn serial_number(&self) -> String {
        match self {
            DeviceInfo::V1(info) => CStr::from_bytes_until_nul(&info.szSN.map(|v| v as u8)).unwrap().to_string_lossy().to_string(),
            DeviceInfo::V2(info) => CStr::from_bytes_until_nul(&info.szSN.map(|v| v as u8)).unwrap().to_string_lossy().to_string(),
        }
    }
    pub fn display_name(&self) -> String {
        match self {
            DeviceInfo::V1(info) => CStr::from_bytes_until_nul(&info.szDisplayName.map(|v| v as u8)).unwrap().to_string_lossy().to_string(),
            DeviceInfo::V2(info) => CStr::from_bytes_until_nul(&info.szDisplayName.map(|v| v as u8)).unwrap().to_string_lossy().to_string(),
        }
    }
    pub fn device_id(&self) -> String {
        match self {
            DeviceInfo::V1(info) => CStr::from_bytes_until_nul(&info.szDeviceID.map(|v| v as u8)).unwrap().to_string_lossy().to_string(),
            DeviceInfo::V2(info) => CStr::from_bytes_until_nul(&info.szDeviceID.map(|v| v as u8)).unwrap().to_string_lossy().to_string(),
        }
    }
    pub fn user_id(&self) -> String {
        match self {
            DeviceInfo::V1(info) => CStr::from_bytes_until_nul(&info.szUserID.map(|v| v as u8)).unwrap().to_string_lossy().to_string(),
            DeviceInfo::V2(info) => CStr::from_bytes_until_nul(&info.szUserID.map(|v| v as u8)).unwrap().to_string_lossy().to_string(),
        }
    }

    // TODO supported_access_status,
    // TODO device_class
}

impl Debug for DeviceInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeviceInfo")
            .field("vendor_name", &self.vendor_name())
            .field("model_name", &self.model_name())
            .field("serial_number", &self.serial_number())
            .field("display_name", &self.display_name())
            .field("device_id", &self.device_id())
            .field("user_id", &self.user_id())
            // TODO supported_access_status,
            // TODO device_class
            .finish()
    }
}
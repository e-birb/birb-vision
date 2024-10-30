use std::{ffi::CStr, fmt::Debug};

use daheng_sys::{v1, v2};

#[derive(Clone, Copy)]
pub enum DeviceInfo {
    V1(v1::GX_DEVICE_BASE_INFO),
    V2(v2::GX_DEVICE_BASE_INFO),
}

impl DeviceInfo {
    pub fn vendor_name(&self) -> &CStr {
        match self {
            DeviceInfo::V1(info) => convert_string_array(&info.szVendorName),
            DeviceInfo::V2(info) => convert_string_array(&info.szVendorName),
        }
    }
    pub fn model_name(&self) -> &CStr {
        match self {
            DeviceInfo::V1(info) => convert_string_array(&info.szModelName),
            DeviceInfo::V2(info) => convert_string_array(&info.szModelName),
        }
    }
    pub fn serial_number(&self) -> &CStr {
        match self {
            DeviceInfo::V1(info) => convert_string_array(&info.szSN),
            DeviceInfo::V2(info) => convert_string_array(&info.szSN),
        }
    }
    pub fn display_name(&self) -> &CStr {
        match self {
            DeviceInfo::V1(info) => convert_string_array(&info.szDisplayName),
            DeviceInfo::V2(info) => convert_string_array(&info.szDisplayName),
        }
    }
    pub fn device_id(&self) -> &CStr {
        match self {
            DeviceInfo::V1(info) => convert_string_array(&info.szDeviceID),
            DeviceInfo::V2(info) => convert_string_array(&info.szDeviceID),
        }
    }
    pub fn user_id(&self) -> &CStr {
        match self {
            DeviceInfo::V1(info) => convert_string_array(&info.szUserID),
            DeviceInfo::V2(info) => convert_string_array(&info.szUserID),
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

fn convert_string_array(data: &[i8]) -> &CStr {
    let data = unsafe { std::slice::from_raw_parts(data.as_ptr() as *const u8, data.len()) };
    CStr::from_bytes_with_nul(data).unwrap()
}
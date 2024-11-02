use std::{ffi::c_void, sync::Mutex};

use birb_vision_core::{context::DeviceInfoEntry, CameraDevice, DeviceError};
use callbacks::DeviceCallbacks;
use daheng_sys::{v1, v2, SDK};

use crate::{ctx::try_common, Ctx, DahengError, GxError};

mod callbacks;
mod info;

pub use info::DeviceInfo;

pub struct Device {
    cx: Ctx,
    info: DeviceInfo,
    handle: *mut c_void,
    callbacks: Box<Mutex<DeviceCallbacks>>, // TODO maybe pin it?
}

impl Device {
    pub fn open(info: DeviceInfo) -> Result<Self, DahengError> {
        let mut handle = std::ptr::null_mut();

        // note: creating callbacks is a potential point of failure so we build
        // it before opening the device so that after opening the device
        // unwinding will close the device
        let callbacks = DeviceCallbacks::new();

        let cx = Ctx::new()?;

        GxError::result(cx.sdk(), match cx.sdk() {
            daheng_sys::SDK::V1(api) => unsafe {
                let mut open_param = v1::GX_OPEN_PARAM {
                    pszContent: info.serial_number().as_ptr() as *mut i8,
                    openMode: v1::GX_OPEN_MODE_GX_OPEN_SN as i32,
                    accessMode: v1::GX_ACCESS_MODE_GX_ACCESS_EXCLUSIVE as i32,
                };
                api.GXOpenDevice(&mut open_param, &mut handle)
            },
            daheng_sys::SDK::V2(api) => unsafe {
                let mut open_param = v2::GX_OPEN_PARAM {
                    pszContent: info.serial_number().as_ptr() as *mut i8,
                    openMode: v1::GX_OPEN_MODE_GX_OPEN_SN as i32,
                    accessMode: v1::GX_ACCESS_MODE_GX_ACCESS_EXCLUSIVE as i32,
                };
                api.GXOpenDevice(&mut open_param, &mut handle)
            },
        })?;

        let device = Self {
            cx,
            info: info.clone(),
            handle,
            callbacks,
        };

        DeviceCallbacks::setup(&device)?;

        Ok(device)
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        if let Err(e) = try_common!(self.cx.sdk() => GXCloseDevice(self.handle)) {
            log::error!("Failed to close Daheng device: {e}");
        }
    }
}

impl CameraDevice for Device {
    fn get_device_info(&self) -> birb_vision_core::DeviceResult<birb_vision_core::context::DeviceInfo> {
        let mut info = birb_vision_core::context::DeviceInfo::new();
        info.display_name = self.info.display_name().to_string_lossy().into_owned();
        info.other.insert("vendor_name".into(), DeviceInfoEntry::new("Vendor Name", self.info.vendor_name().to_string_lossy()));
        info.other.insert("model_name".into(), DeviceInfoEntry::new("Model Name", self.info.model_name().to_string_lossy()));
        info.other.insert("serial_number".into(), DeviceInfoEntry::new("Serial Number", self.info.serial_number().to_string_lossy()));
        info.other.insert("device_id".into(), DeviceInfoEntry::new("Device ID", self.info.device_id().to_string_lossy()));
        info.other.insert("user_id".into(), DeviceInfoEntry::new("User ID", self.info.vendor_name().to_string_lossy()));
        // TODO ...
        Ok(info)
    }

    fn start_grabbing(&self) -> birb_vision_core::DeviceResult {
        GxError::result(self.cx.sdk(), match self.cx.sdk() {
            SDK::V1(v1) => unsafe { v1.GXStreamOn(self.handle) },
            SDK::V2(v2) => unsafe { v2.GXSendCommand(self.handle, v2::GX_FEATURE_ID_GX_COMMAND_ACQUISITION_START as i32) },
        })?;
        Ok(())
    }

    fn stop_grabbing(&self) -> birb_vision_core::DeviceResult {
        GxError::result(self.cx.sdk(), match self.cx.sdk() {
            SDK::V1(v1) => unsafe { v1.GXStreamOff(self.handle) },
            SDK::V2(v2) => unsafe { v2.GXSendCommand(self.handle, v2::GX_FEATURE_ID_GX_COMMAND_ACQUISITION_STOP as i32) },
        })?;
        Ok(())
    }

    fn set_stream_callback(&self, f: Box<dyn for<'a> Fn(birb_vision_core::StreamEvent<'a>) + Send + Sync>) -> birb_vision_core::DeviceResult {
        self.callbacks.lock().unwrap().stream_callback = Some(f);
        Ok(())
    }

    fn grab(&self) -> birb_vision_core::DeviceResult {
        // TODO
        Err(DeviceError::NotImplemented)
    }
}
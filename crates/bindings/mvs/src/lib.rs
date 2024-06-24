#![doc = include_str!("../README.md")]

use std::{ffi::c_void, path::Path};

use device::{AccessMode, DeviceInfo};
pub use log;

pub mod property; use property::*;
pub mod device;
mod error; pub use error::MVSError;
mod version; pub use version::MVSVersion;
mod ctx; pub use ctx::{MVSContext, MVSContextCreationError};
pub use mvs_sys;

pub mod prelude {
    pub use crate::{
        MVSContext,
        MVSDevice,
        device::{
            TransportLayerType,
            AccessMode,
        },
    };
}

pub mod ext {
    pub use mvs_sys;
    pub use semver;
    pub use log;
}

/// A Device Handle.
///
/// This is the main object you will be working with when interacting with a camera.
///
/// Note that creating a device handle does not open the camera. You need to call [`MVSDevice::open()`] to do that.
///
/// # Thread Safety
/// The device handle is not thread-safe and implments `!Send` and `!Sync`.
pub struct MVSDevice {
    cx: MVSContext,
    /// The actual device handle
    ///
    /// Note that this correctly makes the struct `!Send` and `!Sync`
    handle: *mut c_void,
}

impl MVSDevice {
    /// Create a new camera handle.
    ///
    /// # Parameters
    /// - `device_info`: Information about the device
    /// - `log`: Whether to log messages from the SDK
    ///
    /// # Notes
    /// The SDK logs messages to a file. The path can be specified with [`MVSContext::set_sdk_log_path()`].
    pub fn new(
        device_info: DeviceInfo,
        log: bool,
    ) -> Result<Self, MVSError> {
        let mut handle = std::ptr::null_mut();
        let cx = device_info.cx.clone();

        if log {
            mvs_try!(cx => MV_CC_CreateHandle(
                &mut handle,
                &device_info.info as *const _ as *mut _,
            ))?;
        } else {
            mvs_try!(cx => MV_CC_CreateHandleWithoutLog(
                &mut handle,
                &device_info.info as *const _ as *mut _,
            ))?;
        }

        assert_ne!(handle, std::ptr::null_mut(), "MV_CC_CreateHandle succeeded but returned a null handle");

        Ok(Self {
            cx,
            handle,
        })
    }

    pub fn cx(&self) -> &MVSContext {
        &self.cx
    }

    /// Check if the device is connected.
    pub fn is_connected(&self) -> bool {
        unsafe { self.cx.ffi().MV_CC_IsDeviceConnected(self.handle) }
    }

    // TODO MV_CC_GetAllMatchInfo() ?

    /// Open the device.
    ///
    /// # Parameters
    /// * `mode`: The access mode to open the device with. If unsure, try [`Exclusive`](AccessMode::Exclusive) which is the default value in the C SDK.
    /// * `switchover_key`: The switchover key to use when opening the device. This is optional and defaults to 0.
    ///
    /// # Remarks
    /// From the SDK documentation:
    /// > You can find the specific device and connect according to inputted device parameters.  
    /// > When calling the interface, the parameters nAccessMode and nSwitchoverKey are optional, and the device access mode is exclusive by default. Currently the device does not support the following preemption modes: MV_ACCESS_ExclusiveWithSwitch, MV_ACCESS_ControlWithSwitch, MV_ACCESS_ControlSwitchEnableWithKey.  
    /// For USB3Vision device, the parameters nAccessMode and nSwitchoverKey are invalid.
    pub fn open(
        &self,
        mode: AccessMode,
        switchover_key: Option<u16>,
    ) -> Result<(), MVSError> {
        let switchover_key = switchover_key.unwrap_or(0);
        mvs_try!(self.cx => MV_CC_OpenDevice(self.handle, mode as u32, switchover_key))
    }

    pub fn close(&self) -> Result<(), MVSError> {
        mvs_try!(self.cx => MV_CC_CloseDevice(self.handle))
    }

    /// Get information about the device.
    ///
    /// # Remarks
    /// From the SDK documentation:
    /// * The API is not supported by GenTL cameras
    /// * If the device is a **GigE** camera, there is a **blocking risk** when calling the API, so it is not recommended to call the API during the streaming process
    pub fn get_info(&self) -> Result<DeviceInfo, MVSError> {
        let mut info = unsafe { std::mem::zeroed() };
        mvs_try!(self.cx => MV_CC_GetDeviceInfo(self.handle, &mut info))?;
        Ok(DeviceInfo {
            cx: self.cx.clone(),
            info,
        })
    }

    pub fn invalidate_nodes(&self) -> Result<(), MVSError> {
        mvs_try!(self.cx => MV_CC_InvalidateNodes(self.handle))
    }

    pub fn get_int_value(&self, key: impl PropId) -> Result<IntValue, MVSError> {
        let mut value = unsafe { std::mem::zeroed() };
        mvs_try!(self.cx => MV_CC_GetIntValue(self.handle, prop_id_to_c_string(key).as_ptr(), &mut value))?;
        Ok(IntValue(value))
    }

    pub fn set_int_value(&self, key: impl PropId, value: u32) -> Result<(), MVSError> {
        mvs_try!(self.cx => MV_CC_SetIntValue(self.handle, prop_id_to_c_string(key).as_ptr(), value))
    }

    pub fn get_enum_value(&self, key: impl PropId) -> Result<EnumValue, MVSError> {
        let mut value = unsafe { std::mem::zeroed() };
        mvs_try!(self.cx => MV_CC_GetEnumValue(self.handle, prop_id_to_c_string(key).as_ptr(), &mut value))?;
        Ok(EnumValue(value))
    }

    pub fn set_enum_value(&self, key: impl PropId, value: u32) -> Result<(), MVSError> {
        mvs_try!(self.cx => MV_CC_SetEnumValue(self.handle, prop_id_to_c_string(key).as_ptr(), value))
    }

    pub fn get_enum_entry_symbolic(&self, key: impl PropId) -> Result<EnumEntrySymbolic, MVSError> {
        let mut entry = unsafe { std::mem::zeroed() };
        mvs_try!(self.cx => MV_CC_GetEnumEntrySymbolic(self.handle, prop_id_to_c_string(key).as_ptr(), &mut entry))?;
        Ok(EnumEntrySymbolic(entry))
    }

    pub fn set_enum_value_by_string(&self, key: impl PropId, value: &str) -> Result<(), MVSError> {
        let c_string = std::ffi::CString::new(value).unwrap();
        mvs_try!(self.cx => MV_CC_SetEnumValueByString(self.handle, prop_id_to_c_string(key).as_ptr(), c_string.as_ptr()))
    }

    pub fn get_float_value(&self, key: impl PropId) -> Result<FloatValue, MVSError> {
        let mut value = unsafe { std::mem::zeroed() };
        mvs_try!(self.cx => MV_CC_GetFloatValue(self.handle, prop_id_to_c_string(key).as_ptr(), &mut value))?;
        Ok(FloatValue(value))
    }

    pub fn set_float_value(&self, key: impl PropId, value: f32) -> Result<(), MVSError> {
        mvs_try!(self.cx => MV_CC_SetFloatValue(self.handle, prop_id_to_c_string(key).as_ptr(), value))
    }

    pub fn get_bool_value(&self, key: impl PropId) -> Result<bool, MVSError> {
        let mut value = false;
        mvs_try!(self.cx => MV_CC_GetBoolValue(self.handle, prop_id_to_c_string(key).as_ptr(), &mut value))?;
        Ok(value)
    }

    pub fn set_bool_value(&self, key: impl PropId, value: bool) -> Result<(), MVSError> {
        mvs_try!(self.cx => MV_CC_SetBoolValue(self.handle, prop_id_to_c_string(key).as_ptr(), value))
    }

    pub fn get_string_value(&self, key: impl PropId) -> Result<StringValue, MVSError> {
        let mut value = unsafe { std::mem::zeroed() };
        mvs_try!(self.cx => MV_CC_GetStringValue(self.handle, prop_id_to_c_string(key).as_ptr(), &mut value))?;
        Ok(StringValue(value))
    }

    pub fn set_string_value(&self, key: impl PropId, value: &str) -> Result<(), MVSError> {
        let c_string = std::ffi::CString::new(value).unwrap();
        mvs_try!(self.cx => MV_CC_SetStringValue(self.handle, prop_id_to_c_string(key).as_ptr(), c_string.as_ptr()))
    }

    /// "Sends" a command to the camera.
    pub fn set_command_value(&self, key: impl PropId) -> Result<(), MVSError> {
        mvs_try!(self.cx => MV_CC_SetCommandValue(self.handle, prop_id_to_c_string(key).as_ptr()))
    }

    /// Import camera property files in XML format.
    pub fn feature_load(&self, path: &Path) -> Result<(), MVSError> {
        let c_string = std::ffi::CString::new(path.to_str().unwrap()).unwrap();
        mvs_try!(self.cx => MV_CC_FeatureLoad(self.handle, c_string.as_ptr()))
    }

    /// Save the camera property file in XML format.
    pub fn feature_save(&self, path: &Path) -> Result<(), MVSError> {
        let c_string = std::ffi::CString::new(path.to_str().unwrap()).unwrap();
        mvs_try!(self.cx => MV_CC_FeatureSave(self.handle, c_string.as_ptr()))
    }

    pub fn open_params_gui(&self) -> Result<(), MVSError> {
        mvs_try!(self.cx => MV_CC_OpenParamsGUI(self.handle))
    }

    /// Read data from device register.
    ///
    /// # Parameters
    /// * `address`: The address of the register. The address can be obtained from Camera.xml, in a form similar to xml node value of xxx_RegAddr (Camera.xml will automatically generate in current program directory after the device is opened).
    /// * `buffer`: The buffer to store the data read from the register (memory value is stored based on **big endian mode**)
    pub fn read_memory(&self, address: u64, buffer: &mut [u8]) -> Result<(), MVSError> {
        mvs_try!(self.cx => MV_CC_ReadMemory(self.handle, buffer.as_mut_ptr() as *mut _, address as i64, buffer.len() as i64))
    }

    /// Write data into a device register.
    ///
    /// # Parameters
    /// * `address`: The address of the register. The address can be obtained from Camera.xml, in a form similar to xml node value of xxx_RegAddr (Camera.xml will automatically generate in current program directory after the device is opened).
    /// * `buffer`: The buffer containing the data to write into the register (the value is to be stored according to **big endian mode**)
    pub fn write_memory(&self, address: u64, buffer: &[u8]) -> Result<(), MVSError> {
        mvs_try!(self.cx => MV_CC_WriteMemory(self.handle, buffer.as_ptr() as *const _, address as i64, buffer.len() as i64))
    }

    #[allow(non_snake_case)]
    pub fn get_GenICam_xml(&self) -> Result<String, MVSError> {
        let mut size = 0;
        mvs_try!(self.cx => MV_XML_GetGenICamXML(self.handle, std::ptr::null_mut(), 0, &mut size))?;
        let mut buffer = vec![0u8; size as usize];
        mvs_try!(self.cx => MV_XML_GetGenICamXML(self.handle, buffer.as_mut_ptr() as *mut _, size, &mut size))?;
        Ok(String::from_utf8(buffer).expect("Failed to convert GenICam XML to UTF-8"))
    }

    pub fn get_node_access_mode(&self, key: impl PropId) -> Result<XmlAccessMode, MVSError> {
        let mut mode = 0;
        mvs_try!(self.cx => MV_XML_GetNodeAccessMode(self.handle, prop_id_to_c_string(key).as_ptr(), &mut mode))?;
        Ok(XmlAccessMode::from_i32(mode))
    }

    pub fn get_node_interface_type(&self, key: impl PropId) -> Result<XmlInterfaceType, MVSError> {
        let mut interface_type = 0;
        mvs_try!(self.cx => MV_XML_GetNodeInterfaceType(self.handle, prop_id_to_c_string(key).as_ptr(), &mut interface_type))?;
        Ok(XmlInterfaceType::from_i32(interface_type))
    }
}

impl Drop for MVSDevice {
    fn drop(&mut self) {
        // try close the camera
        // we don't know if the camera is open or closed at this point, so we ignore the result
        let _ = self.close();

        mvs_try!(self.cx => MV_CC_DestroyHandle(self.handle)).expect("Failed to destroy camera handle")
    }
}
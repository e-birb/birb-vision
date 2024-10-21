#![doc = include_str!("../README.md")]

use std::{ffi::{c_uchar, c_void, CStr}, panic::{catch_unwind, UnwindSafe}, path::Path, pin::Pin, sync::Mutex, time::Duration};

use birb_vision_core::image::DynamicImage;
use device::{AccessMode, DeviceInfo};
pub use log;

pub mod property;
use mvs_sys::MV_FRAME_OUT_INFO_EX;
use pixel::decode_mv_image;
use property::*;
pub mod device;
pub mod error;
pub use error::MVError;
mod version;
pub use version::MVSVersion;
mod ctx;
pub use ctx::{MVContext, MVSContextCreationError};
pub use mvs_sys;
mod genicam;

pub mod pixel;

#[cfg(feature = "birb-vision")]
mod birb_vision_impl;

pub mod prelude {
    pub use crate::{
        device::{AccessMode, TransportLayerType},
        MVContext, MVDevice,
    };
}

pub mod ext {
    pub use log;
    pub use mvs_sys;
    pub use semver;
}

/// A Device Handle.
///
/// This is the main object you will be working with when interacting with a camera.
///
/// Note that creating a device handle does not open the camera. You need to call [`MVSDevice::open()`] to do that.
///
/// # Thread Safety
/// The device handle is not thread-safe and is `!Send` and `!Sync`.
pub struct MVDevice {
    cx: MVContext,
    /// The actual device handle
    ///
    /// Note that this correctly makes the struct `!Send` and `!Sync`
    handle: *mut c_void,

    callbacks: Pin<Box<Mutex<Callbacks>>>,
}

struct Callbacks {
    image_callback: Box<dyn Fn(DynamicImage) + Send + Sync>,
    event_callback: Box<dyn Fn(/*TODO*/) + Send + Sync>,
}

impl Callbacks {
    pub fn new() -> Self {
        Callbacks {
            image_callback: Box::new(|_| {}),
            event_callback: Box::new(|| {}),
        }
    }

    pub fn clear(&mut self) {
        *self = Self::new();
    }
}

type CallbacksPtr = *const Mutex<Callbacks>;

impl MVDevice {
    /// Create a new camera handle.
    ///
    /// # Parameters
    /// - `device_info`: Information about the device
    /// - `log`: Whether to log messages from the SDK
    ///
    /// # Notes
    /// The SDK logs messages to a file. The path can be specified with [`MVSContext::set_sdk_log_path()`].
    pub fn new(device_info: DeviceInfo, log: bool) -> Result<Self, MVError> {
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

        assert_ne!(
            handle,
            std::ptr::null_mut(),
            "MV_CC_CreateHandle succeeded but returned a null handle"
        );

        Ok(Self {
            cx,
            handle,
            callbacks: Box::pin(Mutex::new(Callbacks::new())),
        })
    }

    pub fn cx(&self) -> &MVContext {
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
    pub fn open(&self, mode: AccessMode, switchover_key: Option<u16>) -> Result<(), MVError> {
        let switchover_key = switchover_key.unwrap_or(0);
        mvs_try!(self.cx => MV_CC_OpenDevice(self.handle, mode as u32, switchover_key))?;

        // TODO defer close if any error occurs after this point

        let callbacks: CallbacksPtr = &*self.callbacks;

        mvs_try!(self.cx => MV_CC_RegisterImageCallBackEx(
            self.handle,
            Some(frame_callback),
            callbacks as *mut _,
        ))?;

        mvs_try!(self.cx => MV_CC_RegisterAllEventCallBack(
            self.handle,
            Some(evtent_callback),
            callbacks as *mut _,
        ))?;

        Ok(())
    }

    pub fn close(&self) -> Result<(), MVError> {
        // TODO maybe unregister callbacks?

        // destroy callbacks in order to release any associated resource
        self.callbacks.lock().unwrap().clear();

        mvs_try!(self.cx => MV_CC_CloseDevice(self.handle))
    }

    /// Get information about the device.
    ///
    /// # Remarks
    /// From the SDK documentation:
    /// * The API is not supported by GenTL cameras
    /// * If the device is a **GigE** camera, there is a **blocking risk** when calling the API, so it is not recommended to call the API during the streaming process
    pub fn get_info(&self) -> Result<DeviceInfo, MVError> {
        let mut info = unsafe { std::mem::zeroed() };
        mvs_try!(self.cx => MV_CC_GetDeviceInfo(self.handle, &mut info))?;
        Ok(DeviceInfo {
            cx: self.cx.clone(),
            info,
        })
    }

    pub fn invalidate_nodes(&self) -> Result<(), MVError> {
        mvs_try!(self.cx => MV_CC_InvalidateNodes(self.handle))
    }

    pub fn get_int_value(&self, key: impl PropId) -> Result<IntValue, MVError> {
        let mut value = unsafe { std::mem::zeroed() };
        mvs_try!(self.cx => MV_CC_GetIntValue(self.handle, prop_id_to_c_string(key).as_ptr(), &mut value))?;
        Ok(IntValue(value))
    }

    pub fn set_int_value(&self, key: impl PropId, value: u32) -> Result<(), MVError> {
        mvs_try!(self.cx => MV_CC_SetIntValue(self.handle, prop_id_to_c_string(key).as_ptr(), value))
    }

    pub fn get_enum_value(&self, key: impl PropId) -> Result<EnumValue, MVError> {
        let mut value = unsafe { std::mem::zeroed() };
        mvs_try!(self.cx => MV_CC_GetEnumValue(self.handle, prop_id_to_c_string(key).as_ptr(), &mut value))?;
        Ok(EnumValue(value))
    }

    pub fn set_enum_value(&self, key: impl PropId, value: u32) -> Result<(), MVError> {
        mvs_try!(self.cx => MV_CC_SetEnumValue(self.handle, prop_id_to_c_string(key).as_ptr(), value))
    }

    pub fn get_enum_entry_symbolic(&self, key: impl PropId) -> Result<EnumEntrySymbolic, MVError> {
        let mut entry = unsafe { std::mem::zeroed() };
        mvs_try!(self.cx => MV_CC_GetEnumEntrySymbolic(self.handle, prop_id_to_c_string(key).as_ptr(), &mut entry))?;
        Ok(EnumEntrySymbolic(entry))
    }

    pub fn set_enum_value_by_string(&self, key: impl PropId, value: &str) -> Result<(), MVError> {
        let c_string = std::ffi::CString::new(value).unwrap();
        mvs_try!(self.cx => MV_CC_SetEnumValueByString(self.handle, prop_id_to_c_string(key).as_ptr(), c_string.as_ptr()))
    }

    pub fn get_float_value(&self, key: impl PropId) -> Result<FloatValue, MVError> {
        let mut value = unsafe { std::mem::zeroed() };
        mvs_try!(self.cx => MV_CC_GetFloatValue(self.handle, prop_id_to_c_string(key).as_ptr(), &mut value))?;
        Ok(FloatValue(value))
    }

    pub fn set_float_value(&self, key: impl PropId, value: f32) -> Result<(), MVError> {
        mvs_try!(self.cx => MV_CC_SetFloatValue(self.handle, prop_id_to_c_string(key).as_ptr(), value))
    }

    pub fn get_bool_value(&self, key: impl PropId) -> Result<bool, MVError> {
        let mut value = false;
        mvs_try!(self.cx => MV_CC_GetBoolValue(self.handle, prop_id_to_c_string(key).as_ptr(), &mut value))?;
        Ok(value)
    }

    pub fn set_bool_value(&self, key: impl PropId, value: bool) -> Result<(), MVError> {
        mvs_try!(self.cx => MV_CC_SetBoolValue(self.handle, prop_id_to_c_string(key).as_ptr(), value))
    }

    pub fn get_string_value(&self, key: impl PropId) -> Result<StringValue, MVError> {
        let mut value = unsafe { std::mem::zeroed() };
        mvs_try!(self.cx => MV_CC_GetStringValue(self.handle, prop_id_to_c_string(key).as_ptr(), &mut value))?;
        Ok(StringValue(value))
    }

    pub fn set_string_value(&self, key: impl PropId, value: &str) -> Result<(), MVError> {
        let c_string = std::ffi::CString::new(value).unwrap();
        mvs_try!(self.cx => MV_CC_SetStringValue(self.handle, prop_id_to_c_string(key).as_ptr(), c_string.as_ptr()))
    }

    /// "Sends" a command to the camera.
    pub fn set_command_value(&self, key: impl PropId) -> Result<(), MVError> {
        mvs_try!(self.cx => MV_CC_SetCommandValue(self.handle, prop_id_to_c_string(key).as_ptr()))
    }

    /// Import camera property files in XML format.
    pub fn feature_load(&self, path: &Path) -> Result<(), MVError> {
        let c_string = std::ffi::CString::new(path.to_str().unwrap()).unwrap();
        mvs_try!(self.cx => MV_CC_FeatureLoad(self.handle, c_string.as_ptr()))
    }

    /// Save the camera property file in XML format.
    pub fn feature_save(&self, path: &Path) -> Result<(), MVError> {
        let c_string = std::ffi::CString::new(path.to_str().unwrap()).unwrap();
        mvs_try!(self.cx => MV_CC_FeatureSave(self.handle, c_string.as_ptr()))
    }

    pub fn open_params_gui(&self) -> Result<(), MVError> {
        mvs_try!(self.cx => MV_CC_OpenParamsGUI(self.handle))
    }

    /// Read data from device register.
    ///
    /// # Parameters
    /// * `address`: The address of the register. The address can be obtained from Camera.xml, in a form similar to xml node value of xxx_RegAddr (Camera.xml will automatically generate in current program directory after the device is opened).
    /// * `buffer`: The buffer to store the data read from the register (memory value is stored based on **big endian mode**)
    pub fn read_memory(&self, address: u64, buffer: &mut [u8]) -> Result<(), MVError> {
        mvs_try!(self.cx => MV_CC_ReadMemory(self.handle, buffer.as_mut_ptr() as *mut _, address as i64, buffer.len() as i64))
    }

    /// Write data into a device register.
    ///
    /// # Parameters
    /// * `address`: The address of the register. The address can be obtained from Camera.xml, in a form similar to xml node value of xxx_RegAddr (Camera.xml will automatically generate in current program directory after the device is opened).
    /// * `buffer`: The buffer containing the data to write into the register (the value is to be stored according to **big endian mode**)
    pub fn write_memory(&self, address: u64, buffer: &[u8]) -> Result<(), MVError> {
        mvs_try!(self.cx => MV_CC_WriteMemory(self.handle, buffer.as_ptr() as *const _, address as i64, buffer.len() as i64))
    }

    #[allow(non_snake_case)]
    pub fn get_GenICam_xml(&self) -> Result<String, MVError> {
        // TODO remove println!("a");
        let mut size = {
            let mut size = 0;
            if let Err(MVError::PARAMETER) = mvs_try!(self.cx => MV_XML_GetGenICamXML(self.handle, std::ptr::null_mut(), 0, &mut size)) {
                // HACK: on Linux it appears that this methods fails, so we just assume the size is less then 10 MB
                // TODO remove println!("b");
                10000000 // 10 MB
            } else {
                // TODO remove println!("b2");
                size
            }
        };
        // TODO remove println!("c");
        let mut buffer = vec![0u8; size as usize];
        mvs_try!(self.cx => MV_XML_GetGenICamXML(self.handle, buffer.as_mut_ptr() as *mut _, size, &mut size))?;
        // TODO remove println!("d");
        //if size == buffer.len() as _ {
        //    // HACK: same as above, we try to find the first null byte
        //    size = buffer.iter().position(|&b| b == 0).unwrap_or(buffer.len()) as _;
        //}
        buffer.resize(size as _, 0);
        Ok(String::from_utf8(buffer).expect("Failed to convert GenICam XML to UTF-8"))
    }

    pub fn get_node_access_mode(&self, key: impl PropId) -> Result<XmlAccessMode, MVError> {
        let mut mode = 0;
        mvs_try!(self.cx => MV_XML_GetNodeAccessMode(self.handle, prop_id_to_c_string(key).as_ptr(), &mut mode))?;
        Ok(XmlAccessMode::from_i32(mode))
    }

    pub fn get_node_interface_type(&self, key: impl PropId) -> Result<XmlInterfaceType, MVError> {
        let mut interface_type = 0;
        mvs_try!(self.cx => MV_XML_GetNodeInterfaceType(self.handle, prop_id_to_c_string(key).as_ptr(), &mut interface_type))?;
        Ok(XmlInterfaceType::from_i32(interface_type))
    }

    // TOOD from here on it is a mess, to tidy up

    pub fn start_grabbing(&self) -> Result<(), MVError> {
        mvs_try!(self.cx => MV_CC_StartGrabbing(self.handle))
    }

    pub fn stop_grabbing(&self) -> Result<(), MVError> {
        mvs_try!(self.cx => MV_CC_StopGrabbing(self.handle))
    }

    pub fn get_one_frame_timeout(&self, data: &mut [u8], timeout: Duration) -> Result<(), MVError> {
        let mut info = unsafe { std::mem::zeroed() };
        mvs_try!(self.cx => MV_CC_GetOneFrameTimeout(
            self.handle,
            data.as_mut_ptr() as *mut _,
            data.len() as u32,
            &mut info,
            timeout.as_millis() as u32,
        ))
    }

    //pub fn get_image_buffer(&self, timeout: Option<Duration>) -> Result<(), MVError> {
//
    //}

    // TODO move?
    pub fn set_image_callback(&self, f: Box<dyn Fn(DynamicImage) + Send + Sync + 'static>) {
        self.callbacks.lock().unwrap().image_callback = f;
    }

    pub fn set_all_event_callback(&self, f: Box<dyn Fn(/*TODO*/) + Send + Sync + 'static>) {
        self.callbacks.lock().unwrap().event_callback = f;
    }
}

impl Drop for MVDevice {
    fn drop(&mut self) {
        // try close the camera
        // we don't know if the camera is open or closed at this point, so we ignore the result
        let _ = self.close();

        mvs_try!(self.cx => MV_CC_DestroyHandle(self.handle))
            .expect("Failed to destroy camera handle")
    }
}

#[allow(non_snake_case)]
extern "C" fn frame_callback(pData: *mut c_uchar, pFrameInfo: *mut MV_FRAME_OUT_INFO_EX, pUser: *mut c_void) {
    try_no_panic(|| {
        assert!(!pFrameInfo.is_null());
        let info = unsafe { &*pFrameInfo };
        let w = info.nWidth;
        let h = info.nHeight;
        //println!("Frame: {}x{}", w, h);
    
        assert!(!pUser.is_null());
        let callbacks = pUser as CallbacksPtr;
        let callbacks = unsafe { &*callbacks };
    
        assert!(!pData.is_null());
        let data = unsafe { std::slice::from_raw_parts(pData as *const u8, info.nFrameLen as _) };
        let image = decode_mv_image(w as _, h as _, data, info.enPixelType);
        (callbacks.lock().unwrap().image_callback)(image);
    });
}

#[allow(non_snake_case)]
extern "C" fn evtent_callback(pEventInfo: *mut mvs_sys::MV_EVENT_OUT_INFO, pUser: *mut c_void) {
    try_no_panic(|| {
        assert_ne!(pEventInfo, std::ptr::null_mut());
        let info = unsafe { &*pEventInfo };
        let name: &[u8; 128] = unsafe { std::mem::transmute(&info.EventName) };
        let name = CStr::from_bytes_until_nul(name).unwrap();
        println!("EVENT: {:?}", name);

        assert!(!pUser.is_null());
        let callbacks = pUser as CallbacksPtr;
        let callbacks = unsafe { &*callbacks };

        (callbacks.lock().unwrap().event_callback)();
    });
}

// TODO maybe this is not possible to prove: #[no_panic::no_panic]
fn try_no_panic(f: impl FnOnce() + UnwindSafe) {
    let r = catch_unwind(f);

    if let Err(e) = r {
        let error: &str = if let Some(e) = e.downcast_ref::<String>() {
            e
        } else if let Some(e) = e.downcast_ref::<&str> () {
            e
        } else {
            // This should never happen as panics can only be strings
            "unknown error"
        };

        if let Err(_) = catch_unwind(move || {
            log::error!("MVS callback panicked, callbacks shall never panic as exception cannot propagate in this context: {}", error);
            //drop(e);
        }) {
            eprintln!("MVS callback panicked, callbacks shall never panic as exception cannot propagate in this context. -- Also failed to log or dropping the error! APP STATE MIGHT BE INVALID!");
        }
    }
}
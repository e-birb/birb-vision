use std::{cell::RefCell, ffi::*, sync::{Arc, Mutex}};
use icube_sdk_sys::{v1, v2, SDK};

use crate::*;

#[derive(Clone)]
#[must_use]
#[allow(non_camel_case_types)]
pub struct iCubeContext {
    inner: Arc<iCubeContextInner>,
}

impl iCubeContext {
    pub fn new() -> Result<Self, ContextCreationError> {
        static CURRENT_CONTEXT_INNER: Mutex<RefCell<std::sync::Weak<iCubeContextInner>>> =
            Mutex::new(RefCell::new(std::sync::Weak::new()));

        let ctx = {
            let lock = CURRENT_CONTEXT_INNER
                .lock()
                .expect("Failed to lock the CURRENT_CONTEXT_INNER mutex");
            let current_inner = lock.borrow().upgrade();
            if let Some(inner) = current_inner {
                Self { inner }
            } else {
                let inner = iCubeContextInner::load().map_err(|e| {
                    log::debug!("Failed to load iCube SDK: {}", e);
                    e
                })?;
                *lock.borrow_mut() = Arc::downgrade(&inner);
                Self { inner }
            }
        };

        // TODO check version

        Ok(ctx)
    }

    /// Initializes the device list and calls the provided closure with the list of devices.
    ///
    /// # Notes
    /// The [`ICubeSDK_Init`] function is not well documented, but it is likely that the device list
    /// is invalidated by any subsequent calls to the SDK. Therefore, this function ensures that this
    /// function and other related functions ([`ICubeSDK_GetName`]) are not called concurrently.  
    /// I did not investigate the behavior of the SDK in this regard, but it is better to be safe than sorry.
    /// In the future we may decompile the SDK to understand its behavior better like we did with the
    /// `birb-vision-mvs` crate for the `MV_CC_Initialize` function.
    ///
    /// [`ICubeSDK_Init`]: icube_sdk_sys::ffi::ICubeSDK_Init
    /// [`ICubeSDK_GetName`]: icube_sdk_sys::ffi::ICubeSDK_GetName
    pub fn init_device_list<R>(&self, f: impl FnOnce(Vec<DeviceIndex>) -> R) -> R {
        let sdk = &self.inner.sdk;

        let count = unsafe {
            match sdk {
                SDK::V1(api) => (api.Init)(),
                SDK::V2(api) => (api.Init)(),
            }
        };

        let _lock = self
            .inner
            .enumerate_devices_lock
            .lock()
            .expect("Failed to lock the enumerate_devices_lock mutex");

        let devices: Vec<DeviceIndex> = (0..count)
            .map(|index| {
                let name = unsafe {
                    let mut name = [0i8; v2::NETCAM_NAME_LENGTH as usize];
                    match sdk {
                        SDK::V1(api) => {
                            (api.GetName)(index as _, name.as_mut_ptr(), v2::NETCAM_NAME_LENGTH as _).v2_result()
                        },
                        SDK::V2(api) => {
                            (api.GetName)(index as _, name.as_mut_ptr()).v2_result()
                        },
                    }.map(|_| arr_to_str(&name))
                };

                if name.is_err() {
                    log::error!("Failed to get name for device {}", index)
                }

                DeviceIndex { index, name, ctx: &self }
            })
            .collect();

        f(devices)
    }

    pub(crate) fn sdk(&self) -> &SDK {
        &self.inner.sdk
    }
}

#[allow(non_camel_case_types)]
struct iCubeContextInner {
    sdk: SDK,
    enumerate_devices_lock: Mutex<()>,
}

impl iCubeContextInner {
    fn load() -> Result<Arc<Self>, ContextCreationError> {
        log::trace!("Loading iCube SDK");

        let sdk = unsafe {
            SDK::load().ok_or(ContextCreationError::NotFound)?
        };

        Ok(Arc::new(Self {
            sdk,
            enumerate_devices_lock: Mutex::new(()),
        }))
    }
}

impl Drop for iCubeContextInner {
    fn drop(&mut self) {
        log::trace!("Dropping iCubeContextInner, unloading iCube SDK");
    }
}

#[derive(Debug)]
pub enum ContextCreationError {
    #[allow(non_camel_case_types)]
    iCubeError(iCubeError),
    LoadError(icube_sdk_sys::libloading::Error), // TODO use
    NotFound,
    // TODO version mismatch
}

impl std::fmt::Display for ContextCreationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::iCubeError(e) => write!(f, "iCube error: {}", e),
            Self::LoadError(e) => write!(f, "Failed to load iCube SDK: {}", e),
            Self::NotFound => write!(f, "Failed to find iCube SDK"),
        }
    }
}

impl std::error::Error for ContextCreationError {}

impl From<iCubeError> for ContextCreationError {
    fn from(e: iCubeError) -> Self {
        Self::iCubeError(e)
    }
}

impl From<icube_sdk_sys::libloading::Error> for ContextCreationError {
    fn from(e: icube_sdk_sys::libloading::Error) -> Self {
        Self::LoadError(e)
    }
}

pub struct DeviceIndex<'a> {
    index: i32,
    name: Result<String, iCubeError>,
    ctx: &'a iCubeContext,
}

impl<'a> DeviceIndex<'a> {
    pub fn sdk_index(&self) -> i32 {
        self.index
    }

    pub fn name(&self) -> &Result<String, iCubeError> {
        &self.name
    }

    pub fn open(self) -> Result<iCubeDevice, iCubeError> {
        let sdk = &self.ctx.inner.sdk;

        //ic_try!(ICubeSDK_Open(self.index))?;
        unsafe {
            match sdk {
                SDK::V1(api) => (api.Open)(self.index as _).v1_result()?,
                SDK::V2(api) => (api.Open)(self.index as _).v2_result()?,
            }
        };

        let handle = DeviceHandle {
            ctx: self.ctx.clone(),
            index: self.index,
        };

        // set callback
        let callback = {
            let callback: Box<OptionalCallbackWrapper> = Box::new(Mutex::new(None));

            #[allow(non_snake_case)]
            extern "C" fn raw_callback_v1(buffer: *mut c_void, bufferSize: c_uint, context: *mut c_void) -> c_int {
                assert!(!context.is_null());
                let callback: *const OptionalCallbackWrapper = context as _;
                let callback = unsafe { &*callback };

                let callback = callback.lock().unwrap();

                let Some(callback) = callback.as_ref() else {
                    // no callback, nothing to do
                    return v1::IC_SUCCESS as _;
                };

                let event = {
                    assert!(bufferSize > 0);
                    assert!(!buffer.is_null());
                    let buf: &[u8] = unsafe { std::slice::from_raw_parts(buffer as *const _, bufferSize as _) };
                    CallbackEventType::NEW_FRAME(buf)
                };

                callback(event);

                v1::IC_SUCCESS as _
            }

            #[allow(non_snake_case)]
            extern "C" fn raw_callback_ex_v2(event_type: c_int, pBuf: *mut u8, lBufferSize: c_long, pContext: *mut c_void) -> c_long {
                assert!(!pContext.is_null());
                let callback: *const OptionalCallbackWrapper = pContext as _;
                let callback = unsafe { &*callback };

                let callback = callback.lock().unwrap();

                let Some(callback) = callback.as_ref() else {
                    // no callback, nothing to do
                    return v2::IC_SUCCESS as _;
                };

                let event = match event_type {
                    v2::EVENT_NEW_FRAME => {
                        assert!(lBufferSize > 0);
                        assert!(!pBuf.is_null());
                        let buf: &[u8] = unsafe { std::slice::from_raw_parts(pBuf, lBufferSize as _) };
                        CallbackEventType::NEW_FRAME(buf)
                    },
                    v2::EVENT_DEV_DISCONNECTED => CallbackEventType::DEV_DISCONNECTED,
                    v2::EVENT_DEV_RECONNECTED => CallbackEventType::DEV_RECONNECTED,
                    v2::EVENT_USB_TRANSFER_FAILED => CallbackEventType::USB_TRANSFER_FAILED,
                    _ => CallbackEventType::Unknown(event_type),
                };

                callback(event);

                v2::IC_SUCCESS as _
            }

            let callback_ptr: *const OptionalCallbackWrapper = &*callback;

            unsafe {
                match sdk {
                    SDK::V1(api) => (api.SetCallback)(self.index as _, v1::CALLBACK_RGB as _, raw_callback_v1, callback_ptr as _).v1_result()?,
                    SDK::V2(api) => (api.SetCallbackEx)(self.index as _, v2::CALLBACK_RGB as _, raw_callback_ex_v2, callback_ptr as _).v2_result()?,
                }
            }

            callback
        };

        log::trace!("Opened device {} with handle {:?}", self.index, handle.index);
        Ok(iCubeDevice {
            handle,
            callback,
            //_marker: std::marker::PhantomData,
        })
    }

    pub fn is_open(&self) -> Option<bool> {
        let sdk = &self.ctx.inner.sdk;

        unsafe {
            match sdk {
                SDK::V1(_) => None, // unknown, missing from the SDK

                // The non ex version checks ONLY the current process,
                // the ex version also checks other processes
                SDK::V2(api) => Some((api.IsOpenEx)(self.index as _) == v2::IC_SUCCESS), // TODO strange, to ckeck: IC_SUCCESS is 0...
            }
        }
    }
}
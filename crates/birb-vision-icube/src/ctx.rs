use std::{cell::RefCell, sync::{Arc, Mutex}};
use crate::*;

#[derive(Clone)]
#[must_use]
#[allow(non_camel_case_types)]
pub struct iCubeContext {
    _inner: Arc<iCubeContextInner>,
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
                Self { _inner: inner }
            } else {
                let inner = iCubeContextInner::load().map_err(|e| {
                    log::debug!("Failed to load iCube SDK: {}", e);
                    e
                })?;
                *lock.borrow_mut() = Arc::downgrade(&inner);
                Self { _inner: inner }
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
        let count = unsafe { sdk!(ICubeSDK_Init)() };

        let _lock = self
            ._inner
            .enumerate_devices_lock
            .lock()
            .expect("Failed to lock the enumerate_devices_lock mutex");

        let devices: Vec<DeviceIndex> = (0..count)
            .map(|index| {
                let name = {
                    let mut name = [0i8; ffi::NETCAM_NAME_LENGTH as usize];
                    ic_try!(ICubeSDK_GetName(index as _, name.as_mut_ptr())).map(|_| {
                        let name_len = name.iter().position(|&c| c == 0).unwrap_or(name.len());
                        name[..name_len].iter().map(|&c| c as u8 as char).collect::<String>()
                    })
                };

                if name.is_err() {
                    log::error!("Failed to get name for device {}", index)
                }

                DeviceIndex { index, name, phantom: std::marker::PhantomData }
            })
            .collect();

        f(devices)
    }
}

#[allow(non_camel_case_types)]
struct iCubeContextInner {
    enumerate_devices_lock: Mutex<()>,
}

impl iCubeContextInner {
    fn load() -> Result<Arc<Self>, ContextCreationError> {
        log::trace!("Loading iCube SDK");

        unsafe {
            iCubeError::result_from_code(icube_sdk_sys::load()).map_err(ContextCreationError::iCubeError)?;
        }

        Ok(Arc::new(Self {
            enumerate_devices_lock: Mutex::new(()),
        }))
    }
}

impl Drop for iCubeContextInner {
    fn drop(&mut self) {
        log::trace!("Dropping iCubeContextInner, unloading iCube SDK");

        unsafe {
            icube_sdk_sys::unload();
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ContextCreationError {
    #[allow(non_camel_case_types)]
    iCubeError(iCubeError),
    // TODO version mismatch
}

impl std::fmt::Display for ContextCreationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::iCubeError(e) => write!(f, "iCube error: {}", e),
        }
    }
}

impl std::error::Error for ContextCreationError {}

pub struct DeviceIndex<'a> {
    index: i32,
    name: Result<String, iCubeError>,
    phantom: std::marker::PhantomData<&'a iCubeContext>,
}

impl<'a> DeviceIndex<'a> {
    pub fn sdk_index(&self) -> i32 {
        self.index
    }

    pub fn name(&self) -> &Result<String, iCubeError> {
        &self.name
    }

    pub fn open(self) -> Result<iCubeDevice, iCubeError> {
        ic_try!(ICubeSDK_Open(self.index))?;

        let handle = DeviceHandle {
            index: self.index,
        };

        // set callback
        let callback = {
            let callback: Box<OptionalCallbackWrapper> = Box::new(Mutex::new(None));

            #[allow(non_snake_case)]
            unsafe extern "C" fn raw_callback(
                event_type: ::std::os::raw::c_int,
                pBuf: *mut ffi::BYTE,
                lBufferSize: ffi::LONG,
                pContext: ffi::PVOID,
            ) -> ffi::LONG {
                assert!(!pContext.is_null());
                let callback: *const OptionalCallbackWrapper = pContext as _;
                let callback = &*callback;

                let callback = callback.lock().unwrap();

                let Some(callback) = callback.as_ref() else {
                    // no callback, nothing to do
                    return ffi::IC_SUCCESS as _;
                };

                let event = match event_type as u32 {
                    ffi::EVENT_NEW_FRAME => {
                        assert!(lBufferSize > 0);
                        assert!(!pBuf.is_null());
                        let buf: &[u8] = std::slice::from_raw_parts(pBuf, lBufferSize as _);
                        CallbackEventType::NEW_FRAME(buf)
                    },
                    ffi::EVENT_DEV_DISCONNECTED => CallbackEventType::DEV_DISCONNECTED,
                    ffi::EVENT_DEV_RECONNECTED => CallbackEventType::DEV_RECONNECTED,
                    ffi::EVENT_USB_TRANSFER_FAILED => CallbackEventType::USB_TRANSFER_FAILED,
                    _ => CallbackEventType::Unknown(event_type),
                };

                callback(event);

                ffi::IC_SUCCESS as _
            }

            let callback_ptr: *const OptionalCallbackWrapper = &*callback;

            ic_try!(ICubeSDK_SetCallbackEx(
                self.index,
                ffi::CALLBACK_RGB as _, // TODO maybe use raw if possible
                Some(raw_callback),
                callback_ptr as _,
            ))?;

            callback
        };

        log::trace!("Opened device {} with handle {:?}", self.index, handle);
        Ok(iCubeDevice {
            handle,
            callback,
            _marker: std::marker::PhantomData,
        })
    }

    pub fn is_open(&self) -> bool {
        // The non ex version checks ONLY the current process,
        // the ex version also checks other processes
        let r = unsafe { sdk!(ICubeSDK_IsOpenEx)(self.index) };
        r == ffi::IC_SUCCESS as _
    }
}
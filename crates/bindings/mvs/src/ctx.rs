use crate::*;
use mvs_sys::ext::libloading;

use std::{
    cell::RefCell,
    error::Error,
    fmt::{Debug, Display},
    sync::{Arc, Mutex},
};

use self::device::{AccessMode, TransportLayerType};

/// A macro to simplify calling FFI functions and checking the result.
///
/// # Example
/// ```
/// # use mvs::*;
/// # let ctx = MVSContext::new(None).unwrap();
/// # let mut list = unsafe { std::mem::zeroed() };
/// mvs_try!(ctx => MV_CC_EnumDevices(
///     mvs_sys::MV_USB_DEVICE,
///     &mut list,
/// )).unwrap();
/// ```
#[macro_export]
macro_rules! mvs_try {
    ($ctx:expr => $function:ident ($($args:expr),*$(,)?)) => {
        {
            let result = $crate::MVSError::result_from_code(unsafe {
                $ctx.ffi().$function($($args),*)
            });

            if result.is_err() {
                $crate::log::error!("Failed to call {} with error: {:?}", stringify!($function), result);
            }

            result
        }
    };
}

/// The main context for interacting with the MVS library.
///
/// This struct is the main entry point for interacting with the MVS library.
/// It exposes some high level methods and allows accessing the SDK ffi functions directly.
///
/// Access the SDK ffi functions is exposed by the [`MVSContext::ffi`] method or the [`mvs_try`] macro.
///
/// # Example
/// ```
/// # use mvs::{ MVSContext, mvs_try };
/// let ctx = MVSContext::new(None).unwrap();
///
/// println!("MVS SDK version: {}", ctx.sdk_version());
/// ```
///
/// # Remarks
/// - The context is reference-counted, so you can clone it and pass it around as needed.
/// - [`MV_CC_Initialize`] and [`MV_CC_Finalize`] are called when the first context is created and when the last context is dropped,
///   you SHALL NOT CALL these functions MANUALLY, the context will take care of it.
///
/// [`MV_CC_Initialize`]: mvs_sys::MVS::MV_CC_Initialize
/// [`MV_CC_Finalize`]: mvs_sys::MVS::MV_CC_Finalize
#[derive(Clone)]
#[must_use]
pub struct MVSContext {
    inner: Arc<MVSContextInner>,
}

impl MVSContext {
    /// The required version of the MVS SDK for this crate.
    ///
    /// [`MVSContext::new`] will check the version of the MVS SDK and ensure it is compatible with this version.  
    /// Different versions other than the one specified here may expose different APIs, so it is important to ensure compatibility.
    pub const REQUIRED_SDK_VERSION: &'static str = "~4";

    /// Get or create a current context.
    ///
    /// Returns the current context if it exists, otherwise creates a new context and returns it.  
    /// A version check is performed to ensure compatibility with the MVS SDK, you can pass additional version requirements to the function.
    ///
    /// If you want to perform custom version checks, pass `None` as the `additional_version_requirements` and just use [`MVSContext::sdk_version`].
    ///
    /// # Example
    /// ```
    /// # use mvs::MVSContext;
    /// let ctx = MVSContext::new(None).unwrap();
    /// let ctx = MVSContext::new(Some(">=4.0")).unwrap();
    /// ```
    ///
    /// # Arguments
    /// * `additional_version_requirements`: optional additional version requirements for the MVS SDK in semver format.
    ///
    /// # Panics
    /// This function will panic if the required version is not a valid semver string.
    pub fn new(
        additional_version_requirements: Option<&'static str>,
    ) -> Result<Self, MVSContextCreationError> {
        let ctx = {
            let lock = CURRENT_CONTEXT_INNER
                .lock()
                .expect("Failed to lock the CURRENT_CONTEXT_INNER mutex");
            let current_inner = lock.borrow().upgrade();
            if let Some(inner) = current_inner {
                Self { inner }
            } else {
                let inner = MVSContextInner::load()?;
                *lock.borrow_mut() = Arc::downgrade(&inner);
                Self { inner }
            }
        };

        let version = ctx.sdk_version();
        log::info!("MVS SDK version: {}", version);

        let additional = additional_version_requirements
            .map(|s| ", ".to_string() + s)
            .unwrap_or("".to_string());

        let required = Self::REQUIRED_SDK_VERSION.to_string() + &additional;

        let required = semver::VersionReq::parse(&required)
            .expect("Invalid version requirement passed to MVSContext::new");

        if !required.matches(&version.as_semver()) {
            log::error!(
                "Incompatible MVS SDK version. Required by: {required} but found {version}"
            );
            return Err(MVSContextCreationError::IncompatibleVersion {
                required,
                found: version.as_semver(),
                required_by_mvs_crate: Self::REQUIRED_SDK_VERSION,
                additional_version_requirements,
            });
        }

        Ok(ctx)
    }

    /// Get the current context.
    ///
    /// This function will return `None` if there is no current context.  
    /// Use [`MVSContext::new`] to create a new context.
    ///
    /// This function is useful when you need to decide whether to use MVS at some specific point,
    /// for example at program startup.
    ///
    /// # Example
    /// Correct usage:
    /// ```no_run
    /// # use mvs::MVSContext;
    /// // Create a new context and keep it alive
    /// let _ctx = MVSContext::new(None).unwrap();
    ///
    /// // anywhere else in the code, we can get the current context:
    /// let ctx = MVSContext::current().unwrap();
    /// ```
    ///
    /// This other example will fail because there is no current context:
    /// ```should_panic no_run
    /// # use mvs::MVSContext;
    /// let _ = MVSContext::current().expect("No current context");
    /// ```
    ///
    /// A typical usage example might be the following:
    /// ```no_run
    /// # use mvs::prelude::*;
    /// // Startup section of your program:
    /// // we try to load a specific version of the MVS SDK, this will
    /// // return None if no suitable version is found.
    /// log::trace!("Trying to load the MVS SDK...");
    /// let mvs = MVSContext::new(Some("~4.3")).ok();
    ///
    /// if mvs.is_some() {
    ///     log::info!("MVS SDK loaded successfully");
    /// } else {
    ///     log::warn!("MVS SDK not found, some features will be disabled");
    /// }
    ///
    /// // ---
    ///
    /// // Later in the code we don't want to try to load the MVS SDK again,
    /// // we just want to use it if it is available and continue if it is not:
    ///
    /// let mut devices_count = 0;
    ///
    /// if let Some(cx) = MVSContext::current() {
    ///    devices_count += cx.enumerate_all_devices().unwrap().len();
    /// }
    ///
    /// log::info!("Found {} devices", devices_count);
    /// ```
    pub fn current() -> Option<Self> {
        CURRENT_CONTEXT_INNER
            .lock()
            .expect("Failed to lock the CURRENT_CONTEXT_INNER mutex in MVSContext::current")
            .borrow()
            .upgrade()
            .map(|inner| Self { inner })
    }

    /// Get the version of the MVS SDK.
    ///
    /// # Example
    /// ```no_run
    /// # use mvs::prelude::*;
    /// # let cx = MVSContext::new(None).unwrap();
    /// println!("MVS SDK version: {}", cx.sdk_version());
    /// ```
    pub fn sdk_version(&self) -> MVSVersion {
        MVSVersion::from(unsafe { self.inner.lib.MV_CC_GetSDKVersion() } as u32)
    }

    /// Set the SDK log path.
    ///
    /// You can enable/disable logging while creating a handle with [`MVSDevice::new`].
    #[allow(unused_variables)] // TODO remove
    #[allow(unreachable_code)] // TODO remove
    pub fn set_sdk_log_path(&self, path: impl AsRef<Path>) -> Result<(), Box<dyn Error>> {
        todo!("It is unclear whether the path should be a file or a directory");
        // it is also unclear what happens if you pass an invalid path (dir or file) instead of the expected one
        // because this methods seems to succeed in both cases

        log::info!("Setting MVS SDK log path to {:?}", path.as_ref());

        let path = path.as_ref().to_str().ok_or("Invalid path")?;
        let c_path = std::ffi::CString::new(path)?;

        mvs_try!(self => MV_CC_SetSDKLogPath(c_path.as_ptr()))?;

        Ok(())
    }

    pub fn ffi(&self) -> &mvs_sys::MVS {
        &self.inner.lib
    }

    /// Enumerate all transport layers supported by the MVS SDK.
    pub fn enumerate_transport_layers(&self) -> Vec<TransportLayerType> {
        let layers_value = unsafe { self.ffi().MV_CC_EnumerateTls() } as u32;

        let mut layers = Vec::new();

        let mut layer = 1;
        while layer != 0 {
            if layers_value & layer != 0 {
                layers.push(TransportLayerType::from_u32(layer));
            }

            layer <<= 1;
        }

        layers
    }

    /*/// Call an FFI function and check the result.
    ///
    /// This function should be used to call FFI functions that return an error code.
    ///
    /// # Warning!
    /// **Inside the closure, you should only perform a single call to a ONE ffi function**, it is not adviced to perform other operations inside the closure
    /// because this method is designed to just provide a convienient way to call FFI functions and check the result. Performing other operations inside the closure
    /// **eludes the purpose of this method**.
    /// Prefer [`mvs_try`](crate::mvs_try) and only use this method to get better editor suggestions while **prototypeing**.
    ///
    /// # Example
    /// ```
    /// # use mvs::MVSContext;
    /// # use mvs::ffi::MVSError;
    /// # let ctx = MVSContext::new(None).unwrap();
    /// # let mut list = unsafe { std::mem::zeroed() };
    /// unsafe {
    ///     ctx.try_ffi(|lib| unsafe { lib.MV_CC_EnumDevices(
    ///         mvs::ffi::MV_USB_DEVICE,
    ///         &mut list,
    ///     )})
    /// }.unwrap();
    /// ```
    pub fn try_ffi(&self, f: impl FnOnce(&ffi::MVS) -> c_int) -> Result<(), MVSError> {
        MVSError::result_from_code(f(self.ffi()))
    }*/

    /// Enumerate all devices connected to the system.
    ///
    /// # Examples
    /// ```no_run
    /// # use mvs::{ MVSContext, device::TransportLayerType };
    /// # let ctx = MVSContext::new(None).unwrap();
    /// let devices = ctx.enumerate_devices(TransportLayerType::ALL).unwrap();
    /// let devices = ctx.enumerate_devices([TransportLayerType::Usb]).unwrap();
    /// ```
    ///
    /// # Notes
    /// This crate does not provide any filtering method (i.e. [MV_CC_EnumDevicesEx](mvs_sys::MVS::MV_CC_EnumDevicesEx) or [MV_CC_EnumDevicesEx2](mvs_sys::MVS::MV_CC_EnumDevicesEx2)) other than the transport
    /// layer in order to reduce the methodss bloat. You sould perform finer filtering by yourself.
    pub fn enumerate_devices(
        &self,
        transport_layers: impl IntoIterator<Item = TransportLayerType>,
    ) -> Result<Vec<DeviceInfo>, MVSError> {
        // TODO reference the issue about multithreading
        let _lock = self
            .inner
            .enumerate_devices_lock
            .lock()
            .expect("Failed to lock the enumerate_devices_lock mutex");

        // Note: from what I understood form the MV_CC_EnumDevices documentation, the memory allocated
        // in this list is thread-local and managed by the MVS SDK so we don't need to free it, but since it
        // may be invalidated by subsequent API calls, we immediately convert it to a Vec.
        let mut list: mvs_sys::MV_CC_DEVICE_INFO_LIST = unsafe { std::mem::zeroed() };

        let transport_layers = transport_layers
            .into_iter()
            .map(|ty| ty.code())
            .fold(0, |acc, x| acc | x);

        mvs_try!(self => MV_CC_EnumDevices(
            transport_layers,
            &mut list,
        ))?;

        Ok((0..list.nDeviceNum)
            .map(|i| unsafe { *list.pDeviceInfo[i as usize] })
            .map(|info| DeviceInfo {
                cx: self.clone(),
                info,
            })
            .collect())
    }

    /// Enumerate all devices connected to the system.
    ///
    /// This is equivalent to calling [`MVSContext::enumerate_devices`] with [`TransportLayerType::ALL`].
    pub fn enumerate_all_devices(&self) -> Result<Vec<DeviceInfo>, MVSError> {
        self.enumerate_devices(TransportLayerType::ALL)
    }

    /// Check if a device is accessible.
    pub fn is_accessible(&self, info: &DeviceInfo, mode: AccessMode) -> bool {
        unsafe {
            self.ffi()
                .MV_CC_IsDeviceAccessible(&info.info as *const _ as *mut _, mode as _)
        }
    }
}

struct MVSContextInner {
    lib: mvs_sys::MVS,
    enumerate_devices_lock: Mutex<()>,
}

impl MVSContextInner {
    fn load() -> Result<Arc<Self>, MVSContextCreationError> {
        let lib_result = unsafe { mvs_sys::MVS::load() };
        log::debug!(
            "loaded MVS SDK library: {:?}",
            lib_result.as_ref().map(|_| ())
        );
        let lib = lib_result.map_err(MVSContextCreationError::Loading)?;

        MVSError::result_from_code(unsafe { lib.MV_CC_Initialize() })
            .map_err(MVSContextCreationError::InitializationFailed)?;

        Ok(Arc::new(Self {
            lib,
            enumerate_devices_lock: Mutex::new(()),
        }))
    }
}

impl Drop for MVSContextInner {
    fn drop(&mut self) {
        log::debug!(
            "dropping MVSContextInner (Finalizing MVS SDK) in thread {:?}",
            std::thread::current().id()
        );

        MVSError::result_from_code(unsafe { self.lib.MV_CC_Finalize() })
            .expect("Failed to finalize MVS SDK");
    }
}

static CURRENT_CONTEXT_INNER: Mutex<RefCell<std::sync::Weak<MVSContextInner>>> =
    Mutex::new(RefCell::new(std::sync::Weak::new()));

pub enum MVSContextCreationError {
    Loading(libloading::Error),
    InitializationFailed(MVSError),
    IncompatibleVersion {
        required: semver::VersionReq,
        found: semver::Version,
        required_by_mvs_crate: &'static str,
        additional_version_requirements: Option<&'static str>,
    },
}

impl Debug for MVSContextCreationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Loading(e) => f.debug_tuple("Loading").field(e).finish(),
            Self::InitializationFailed(e) => f.debug_tuple("Initialization").field(e).finish(),
            Self::IncompatibleVersion {
                required,
                found,
                required_by_mvs_crate,
                additional_version_requirements: additional_version_requirement,
            } => f
                .debug_struct("IncompatibleVersion")
                .field("required", &required.to_string())
                .field("found", &found.to_string())
                .field("required_by_mvs_crate", required_by_mvs_crate)
                .field(
                    "additional_version_requirement",
                    additional_version_requirement,
                )
                .finish(),
        }
    }
}

impl Display for MVSContextCreationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Loading(e) => write!(f, "Error loading MVS library: {}", e),
            Self::InitializationFailed(e) => write!(f, "Failed to initialize MVS SDK: {}", e),
            Self::IncompatibleVersion {
                required,
                found,
                required_by_mvs_crate: _,
                additional_version_requirements: _,
            } => write!(
                f,
                "Incompatible MVS library version. Required by: {required} but found {found}"
            ),
        }
    }
}

impl Error for MVSContextCreationError {}

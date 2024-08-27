/*!
 * Backend management.
 *
 * Three main components are defined in this module:
 * - [`BackendSet`]: provides a way to dynamically manage multiple backends.
 * - [`VisionBackend`]: a trait that defines the common methods for backends.
 * - [`DeviceInfo`]: a structure that defines the information about a device.
 *
 * Usually if your app you would define a [`VisionBackendSet`]
 */

use std::{borrow::Cow, collections::HashMap, error::Error, rc::Rc, sync::{Arc, Mutex}};

use serde::{Deserialize, Serialize};

use crate::CameraDevice;

pub type BackendProvider = dyn Fn() -> BackendProviderResult + Send + Sync + 'static;

pub type BackendProviderResult = Result<Box<dyn Backend>, Box<dyn Error>>;

/// A set of [`Backend`] providers.
///
/// This structure provides a way to manage multiple backends.  
/// A [`BackendSet`] is not [`Send`] or [`Sync`] implementations might not be thread-safe.
/// If you want to share the backend set between threads you should use [`BackendProviderSet`]
/// which only defines "how" to create a backend.
pub struct BackendSet {
    providers: BackendProviderSet,
    backends: Mutex<HashMap<String, Rc<dyn Backend>>>,
}

impl BackendSet {
    pub fn new() -> Self {
        Self::new_with_providers(BackendProviderSet::new())
    }

    pub fn new_with_providers(providers: BackendProviderSet) -> Self {
        Self {
            providers,
            backends: Mutex::new(HashMap::new()),
        }
    }

    pub fn providers(&self) -> &BackendProviderSet {
        &self.providers
    }

    /// Get or create a backend for the given type name.
    ///
    /// This function returns `None` if the backend does not exist and an error if
    /// the backend provider failed to create the backend.
    pub fn get_backend(&self, type_name: impl AsRef<str>) -> Option<Result<Rc<dyn Backend>, Box<dyn Error>>> {
        let mut backends = self.backends.lock().unwrap();
        if let Some(backend) = backends.get(type_name.as_ref()) {
            return Some(Ok(backend.clone()));
        } else {
            let backend: Rc<dyn Backend> = match self.providers.get(type_name.as_ref())? {
                Ok(backend) => backend.into(),
                Err(e) => return Some(Err(e)),
            };
            let ok = backends
                .insert(type_name.as_ref().to_string(), backend.clone())
                .is_none();
            debug_assert!(ok, "Backend for type {:?} already exists but this should not happen because it was previously tested", type_name.as_ref());
            Some(Ok(backend))
        }
    }
}

#[derive(Clone)]
pub struct BackendProviderSet {
    frameworks: Arc<Mutex<HashMap<String, Arc<BackendProvider>>>>,
}

impl BackendProviderSet {
    pub fn new() -> Self {
        Self {
            frameworks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Register a new backend provider.
    ///
    /// If the provider already exists, this function will return an error.
    pub fn add(
        &self,
        type_name: impl AsRef<str>,
        provider: impl Fn() -> BackendProviderResult + Send + Sync + 'static,
    ) -> Result<(), Box<dyn Error>> {
        self.frameworks.lock().unwrap()
            .insert(
                type_name.as_ref().to_string(),
                Arc::new(provider),
            ).is_none().then(|| ())
            .ok_or(format!("Backend provider for type {:?} already exists", type_name.as_ref()).into())
    }

    /// Build a new backend for the given type name.
    pub fn get(&self, type_name: impl AsRef<str>) -> Option<BackendProviderResult> {
        self.frameworks.lock().unwrap()
            .get(type_name.as_ref())
            .map(|p| p())
    }
}

/// Common methods for vision backends or "contexts".
///
/// Camera devices are often associated with a context.  
/// Devices should manage the context by themselves or provide some
/// way to keep the context alive.
///
/// This trait is not fundamental as the implementation should
/// provide a more fine control, but this interface is useful to write generic
/// code, especially when dealing with UI.
///
/// Contexts may not be [`Send`] or [`Sync`], for this reason a convenient
/// 
pub trait Backend {
    ///// Unique string identifying this context
    /////
    ///// This is primarily used during serialization.
    //fn type_name(&self) -> Cow<'static, str> {
    //    std::any::type_name::<Self>().into()
    //}

    fn display_name(&self) -> Cow<'static, str>;

    // TODO documentation

    /// Enumerate all devices.
    fn enumerate(&self) -> Result<Vec<DeviceInfo>, Box<dyn Error>>;

    /// Find a device by its information.
    ///
    /// If found, an updated version of the device information is returned.
    fn find(&self, info: &DeviceInfo) -> Result<Vec<DeviceInfo>, Box<dyn Error>>;

    /// Try to create a device.
    ///
    /// If no corresponding device is found, this function should return `Ok(None)`.
    fn create(&self, info: &DeviceInfo) -> Result<Option<Box<dyn CameraDevice>>, Box<dyn Error>>;
}

pub trait GenericDeviceInfo {
    fn id(&self) -> String;
}


/// Information about a device.
///
/// This structure defines two required fields:
/// - [`backend`](Self::backend): the backend **identifier** that this device belongs to.
/// - [`display_name`](Self::display_name): the **display name** of the device.
/// Other fields are stored in the [`other`](Self::other) map and the exact content
/// is backend-specific.
#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(Serialize, Deserialize)]
pub struct DeviceInfo {
    /// The backend that this device belongs to.
    pub backend: String,

    /// Display name of the device.
    pub display_name: String,

    /// Other device information.
    pub other: HashMap<String, DeviceInfoEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Serialize, Deserialize)]
pub struct DeviceInfoEntry {
    pub display_name: String,
    pub value: String,

    /// Whether this entry should be visible to the user.
    pub visible: bool,
}
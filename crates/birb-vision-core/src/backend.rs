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

use std::{collections::{BTreeMap, HashMap}, error::Error, fmt::Debug, hash::Hash, rc::Rc, sync::{Arc, Mutex}};

use serde::{Deserialize, Serialize};

use crate::CameraDevice;

pub enum LogoImageFile {
    Path(String),
    StaticBytes(&'static [u8]),
    Bytes(Arc<Vec<u8>>),
}

impl Debug for LogoImageFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogoImageFile::Path(path) => path.fmt(f),
            LogoImageFile::StaticBytes(_) => f.debug_tuple("LogoImageFile::StaticBytes").finish(),
            LogoImageFile::Bytes(_) => f.debug_tuple("LogoImageFile::Bytes").finish(),
        }
    }
}

pub struct BackendPackage {
    identifier: String,
    display_name: String,
    logo: Option<LogoImageFile>,
    description: Option<String>,
    builder: Box<dyn Fn() -> BackendProviderResult + Send + Sync + 'static>,
}

impl Debug for BackendPackage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BackendPackage")
            .field("identifier", &self.identifier)
            .field("display_name", &self.display_name)
            .field("logo", &self.logo)
            .field("description", &self.description)
            .finish()
    }
}

impl BackendPackage {
    pub fn from_builder_fn<T: Backend>(builder: impl Fn() -> Result<T, Box<dyn Error>> + Send + Sync + 'static) -> Self {
        let identifier = std::any::type_name::<T>().to_string();

        let builder = Box::new(move || -> BackendProviderResult {
            builder().map(|backend| Box::new(backend) as Box<dyn Backend>)
        });

        Self {
            identifier: identifier.clone(),
            display_name: identifier,
            logo: None,
            description: None,
            builder: Box::new(builder),
        }
    }

    pub fn with_display_name(mut self, display_name: impl Into<String>) -> Self {
        self.display_name = display_name.into();
        self
    }

    pub fn with_logo(mut self, logo: LogoImageFile) -> Self {
        self.logo = Some(logo);
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn build_backend(&self) -> BackendProviderResult {
        (self.builder)()
    }
}

type BackendProviderResult = Result<Box<dyn Backend>, Box<dyn Error>>;

/// A set of [`Backend`] providers.
///
/// This structure provides a way to manage multiple backends.  
/// A [`BackendSet`] is not [`Send`] or [`Sync`] implementations might not be thread-safe.
/// If you want to share the backend set between threads you should use [`BackendProviderSet`]
/// which only defines "how" to create a backend.
pub struct BackendSet {
    registry: BackendRegistry,
    backends: Mutex<HashMap<String, Rc<dyn Backend>>>,
}

impl BackendSet {
    pub fn new() -> Self {
        Self::new_with_registry(BackendRegistry::new())
    }

    pub fn new_with_registry(registry: BackendRegistry) -> Self {
        Self {
            registry,
            backends: Mutex::new(HashMap::new()),
        }
    }

    pub fn providers(&self) -> &BackendRegistry {
        &self.registry
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
            let backend: Rc<dyn Backend> = match self.registry.get_backend(type_name.as_ref())? {
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
pub struct BackendRegistry {
    packages: Arc<Mutex<HashMap<String, Arc<BackendPackage>>>>,
}

impl BackendRegistry {
    pub fn new() -> Self {
        Self {
            packages: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Register a new backend provider.
    ///
    /// If the provider already exists, this function will return an error.
    pub fn register(
        &self,
        package: BackendPackage,
    ) -> Result<(), Box<dyn Error>> {
        let identifier = package.identifier.clone();
        self.packages.lock().unwrap()
            .insert(
                identifier.clone(),
                Arc::new(package),
            ).is_none().then(|| ())
            .ok_or(format!("Backend provider for type {:?} already exists", identifier).into())
    }

    /// Build a new backend for the given type name.
    pub fn get_backend(&self, type_name: impl AsRef<str>) -> Option<BackendProviderResult> {
        self.packages.lock().unwrap()
            .get(type_name.as_ref())
            .map(|p| p.build_backend())
    }

    pub fn all_packages(&self) -> HashMap<String, Arc<BackendPackage>> {
        self.packages.lock().unwrap()
            .clone()
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
pub trait Backend: 'static {
    fn available_transport_layers(&self) -> Vec<String>;

    fn default_transport_layers(&self) -> Vec<String> {
        self.available_transport_layers()
    }

    /// Enumerate all devices.
    fn enumerate(&self, transport_layers: &[String]) -> anyhow::Result<Vec<DeviceInfo>>;

    /// Find a device by its information.
    ///
    /// If found, an updated version of the device information is returned.  
    /// This is similar to [`DeviceInfo::new`] except it does not actually create a device.
    ///
    /// # Notes
    /// The default implementation actually calls [`Backend::create`] and returns the device information,
    /// but implementations should provide a more efficient way to find a device.
    fn find(&self, info: &DeviceInfo) -> anyhow::Result<Vec<DeviceInfo>> {
        let device = self.create(info)?;
        if let Some(device) = device {
            let info = device.get_device_info()?;
            Ok(vec![info])
        } else {
            Ok(vec![])
        }
    }

    /// Try to create a device.
    ///
    /// If no corresponding device is found, this function should return `Ok(None)`.
    fn create(&self, info: &DeviceInfo) -> anyhow::Result<Option<Box<dyn CameraDevice>>>;
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
    /// Display name of the device.
    pub display_name: String,

    /// Other device information.
    pub other: BTreeMap<String, DeviceInfoEntry>,
}

impl Hash for DeviceInfo {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.display_name.hash(state);
        for (_, v) in &self.other {
            v.hash(state);
        }
    }
}

impl std::fmt::Display for DeviceInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = f.debug_struct(&self.display_name);

        //s.field("display_name", &self.display_name);

        for (_, value) in &self.other {
            if value.visible {
                s.field(&value.display_name, &format_args!("{}", value.value));
            }
        }

        s.finish()
    }
}

impl DeviceInfo {
    pub fn new() -> Self {
        Self {
            display_name: String::new(),
            other: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Serialize, Deserialize)]
pub struct DeviceInfoEntry {
    pub display_name: String,
    pub value: String,

    /// Whether this entry should be visible to the user.
    pub visible: bool,

    // TODO description / tooltip
}

impl DeviceInfoEntry {
    pub fn new(display_name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            display_name: display_name.into(),
            value: value.into(),
            visible: true,
        }
    }


    pub fn with_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }
}
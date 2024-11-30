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

use std::{collections::BTreeMap, fmt::Debug, hash::Hash};

use serde::{Deserialize, Serialize};

use crate::CameraDevice;

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
pub trait VisionContext: Send + Sync + 'static { // TODO possibly return errors!!!
    fn available_transport_layers(&self) -> Vec<String> {
        vec![]
    }

    fn default_transport_layers(&self) -> Vec<String> {
        self.available_transport_layers()
    }

    /// Enumerate all devices.
    fn enumerate(&self, transport_layers: &[String]) -> anyhow::Result<Vec<DeviceInfo>>;

    ///// Find a device by its information.
    /////
    ///// If found, an updated version of the device information is returned.  
    ///// This is similar to [`DeviceInfo::new`] except it does not actually create a device.
    /////
    ///// # Notes
    ///// The default implementation actually calls [`Backend::create`] and returns the device information,
    ///// but implementations should provide a more efficient way to find a device.
    //fn find(&self, info: &DeviceInfo) -> anyhow::Result<Vec<DeviceInfo>> {
    //    let device = self.create(info)?;
    //    if let Some(device) = device {
    //        let info = device.get_device_info()?;
    //        Ok(vec![info])
    //    } else {
    //        Ok(vec![])
    //    }
    //}

    /// Try to create a device.
    ///
    /// If no corresponding device is found, this function should return `Ok(None)`.
    fn create(&self, info: &DeviceInfo) -> anyhow::Result<Option<Box<dyn CameraDevice>>>; // TODO maybe rename to "open"? with which mode?
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
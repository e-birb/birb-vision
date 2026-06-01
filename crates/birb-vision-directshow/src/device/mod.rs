use std::sync::Arc;
use std::sync::Mutex;

use birb_vision_core::{
    anyhow::anyhow,
    context::{DeviceInfo, DeviceInfoEntry},
    BoolProperty, CameraDevice, DeviceResult, Node, NodeId, NumericProperty, NumericState,
    Property, PropertyState, PropertyValue, Representation, StreamEvent, ValueOrRef,
};
use serde::{Deserialize, Serialize};
use windows::Win32::Media::DirectShow::{
    IAMCameraControl, IAMVideoProcAmp,
    VideoProcAmp_Flags_Auto, VideoProcAmp_Flags_Manual,
};
use windows_core::Interface;

mod control;
mod sample_grabber;

pub use control::DSControl;

use crate::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DSDeviceInfo {
    pub friendly_name: String,
    pub device_path: Option<String>,
}

impl DSDeviceInfo {
    pub fn friendly_name(&self) -> &str {
        &self.friendly_name
    }

    pub fn device_path(&self) -> Option<&str> {
        self.device_path.as_deref()
    }
}

pub struct DirectShowDevice {
    _ctx: Arc<crate::ctx::CtxInner>,
    info: DSDeviceInfo,
    /// The DirectShow capture filter (IBaseFilter) obtained by binding the moniker.
    /// We store it as `IUnknown` so we can query for property interfaces on demand.
    filter: windows_core::IUnknown,
    /// Cached IAMVideoProcAmp interface, if available.
    proc_amp: Option<IAMVideoProcAmp>,
    /// Cached IAMCameraControl interface, if available.
    camera_control: Option<IAMCameraControl>,
    /// Cached list of supported property nodes.
    properties: Vec<Node>,
    callback: Arc<Mutex<Box<dyn Fn(StreamEvent) + Send + Sync>>>,
    is_streaming: Mutex<bool>,
}

// COM interfaces are reference-counted pointers; they are thread-safe under the COM apartment model.
unsafe impl Send for DirectShowDevice {}
unsafe impl Sync for DirectShowDevice {}

impl DirectShowDevice {
    pub fn new(
        ctx: Arc<crate::ctx::CtxInner>,
        info: DSDeviceInfo,
    ) -> DSResult<Self> {
        // Bind the moniker to create the actual DirectShow capture filter
        let filter = ctx.bind_device_filter(&info)?;

        // Query for the two camera-control COM interfaces
        let proc_amp = filter.cast::<IAMVideoProcAmp>().ok();
        let camera_control = filter.cast::<IAMCameraControl>().ok();

        // Enumerate all known controls and cache the supported ones
        let properties = Self::enumerate_properties(proc_amp.as_ref(), camera_control.as_ref());

        Ok(Self {
            _ctx: ctx,
            info,
            filter: filter.into(),
            proc_amp,
            camera_control,
            properties,
            callback: Arc::new(Mutex::new(Box::new(|_| {}))),
            is_streaming: Mutex::new(false),
        })
    }

    /// Enumerate all known DirectShow controls, returning only those the camera supports.
    fn enumerate_properties(
        proc_amp: Option<&IAMVideoProcAmp>,
        camera_control: Option<&IAMCameraControl>,
    ) -> Vec<Node> {
        use strum::IntoEnumIterator;

        let mut nodes = Vec::new();

        for control in DSControl::iter() {
            let range = match Self::get_control_range(control, proc_amp, camera_control) {
                Ok(r) => r,
                Err(_) => continue, // property not supported by this device
            };

            let name = format!("{control:?}");
            let node_id = match control.into_node_id() {
                Ok(id) => id,
                Err(e) => {
                    log::error!("Failed to create NodeId for {control:?}: {e}");
                    continue;
                }
            };

            let property = if control.is_boolean() {
                let default = range.default != 0;
                let mut prop = BoolProperty::new(node_id);
                prop.display_name = name;
                prop.default = Some(default);
                prop.access_mode = property_access_mode(range.caps_flags);
                Property::Bool(prop)
            } else {
                let mut prop = NumericProperty::<i64>::new(node_id);
                prop.display_name = name;
                prop.min = Some(ValueOrRef::Value(range.min as i64));
                prop.max = Some(ValueOrRef::Value(range.max as i64));
                prop.default = Some(range.default as i64);
                prop.increment = Some(ValueOrRef::Value(range.stepping_delta.max(1) as i64));
                prop.representation = Some(Representation::Linear);
                prop.access_mode = property_access_mode(range.caps_flags);
                Property::Integer(prop)
            };

            nodes.push(Node::Property(property));
        }

        nodes
    }

    fn get_control_range(
        control: DSControl,
        proc_amp: Option<&IAMVideoProcAmp>,
        camera_control: Option<&IAMCameraControl>,
    ) -> DSResult<DSControlRange> {
        use control::DSControlKind;

        let mut range = DSControlRange::default();

        let hr = match control.kind() {
            DSControlKind::ProcAmp => {
                let Some(proc_amp) = proc_amp else {
                    return Err(DSError::msg("IAMVideoProcAmp not available"));
                };
                unsafe {
                    proc_amp.GetRange(
                        control.property_id(),
                        &mut range.min,
                        &mut range.max,
                        &mut range.stepping_delta,
                        &mut range.default,
                        &mut range.caps_flags,
                    )
                }
            }
            DSControlKind::CameraControl => {
                let Some(camera_control) = camera_control else {
                    return Err(DSError::msg("IAMCameraControl not available"));
                };
                unsafe {
                    camera_control.GetRange(
                        control.property_id(),
                        &mut range.min,
                        &mut range.max,
                        &mut range.stepping_delta,
                        &mut range.default,
                        &mut range.caps_flags,
                    )
                }
            }
        };

        // HRESULT 0x80070490 = E_PROP_ID_UNSUPPORTED (property not available)
        const E_PROP_ID_UNSUPPORTED: i32 = 0x80070490u32 as i32;
        if let Err(e) = &hr {
            if e.code() == windows_core::HRESULT(E_PROP_ID_UNSUPPORTED) {
                return Err(DSError::msg("Property not supported by this device"));
            }
        }

        hr.map_err(|e| DSError::msg(format!("GetRange failed: {e}")))?;

        Ok(range)
    }

    fn get_control_value(
        &self,
        control: DSControl,
    ) -> DSResult<DSControlValue> {
        use control::DSControlKind;

        let mut value = DSControlValue::default();

        match control.kind() {
            DSControlKind::ProcAmp => {
                let Some(ref proc_amp) = self.proc_amp else {
                    return Err(DSError::msg("IAMVideoProcAmp not available"));
                };
                unsafe {
                    proc_amp.Get(
                        control.property_id(),
                        &mut value.value,
                        &mut value.flags,
                    )?;
                }
            }
            DSControlKind::CameraControl => {
                let Some(ref camera_control) = self.camera_control else {
                    return Err(DSError::msg("IAMCameraControl not available"));
                };
                unsafe {
                    camera_control.Get(
                        control.property_id(),
                        &mut value.value,
                        &mut value.flags,
                    )?;
                }
            }
        }

        Ok(value)
    }

    fn set_control_value(
        &self,
        control: DSControl,
        value: DSControlValue,
    ) -> DSResult<()> {
        use control::DSControlKind;

        match control.kind() {
            DSControlKind::ProcAmp => {
                let Some(ref proc_amp) = self.proc_amp else {
                    return Err(DSError::msg("IAMVideoProcAmp not available"));
                };
                unsafe {
                    proc_amp.Set(control.property_id(), value.value, value.flags)?;
                }
            }
            DSControlKind::CameraControl => {
                let Some(ref camera_control) = self.camera_control else {
                    return Err(DSError::msg("IAMCameraControl not available"));
                };
                unsafe {
                    camera_control.Set(control.property_id(), value.value, value.flags)?;
                }
            }
        }

        Ok(())
    }
}

impl CameraDevice for DirectShowDevice {
    fn get_device_info(&self) -> DeviceResult<DeviceInfo> {
        let mut info = DeviceInfo::new();
        info.display_name = self.info.friendly_name.clone();
        if let Some(ref path) = self.info.device_path {
            info.other.insert(
                "path".into(),
                DeviceInfoEntry::new("Device Path", path.clone()),
            );
        }
        Ok(info)
    }

    fn start_grabbing(&self) -> DeviceResult {
        Err(birb_vision_core::DeviceError::NotImplemented)
    }

    fn stop_grabbing(&self) -> DeviceResult {
        Err(birb_vision_core::DeviceError::NotImplemented)
    }

    fn set_stream_callback(&self, f: Box<dyn Fn(StreamEvent) + Send + Sync>) -> DeviceResult {
        *self.callback.lock().unwrap() = f;
        Ok(())
    }

    fn grab(&self) -> DeviceResult {
        Err(birb_vision_core::DeviceError::NotImplemented)
    }

    fn all_properties(&self) -> DeviceResult<Vec<Node>> {
        Ok(self.properties.clone())
    }

    fn read_property(&self, id: &NodeId) -> DeviceResult<PropertyState> {
        let node_id = DSControl::from_node_id(id)?;

        let control::DSNodeId::Control(control) = node_id;

        let value = self
            .get_control_value(control)
            .map_err(|e| anyhow!("Failed to get control value: {e}"))?;

        let range = Self::get_control_range(control, self.proc_amp.as_ref(), self.camera_control.as_ref())
            .map_err(|e| anyhow!("Failed to get control range: {e}"))?;

        let state = if control.is_boolean() {
            PropertyState::Bool(value.value != 0)
        } else {
            PropertyState::Int(NumericState {
                current: value.value as i64,
                range: range.min as i64..=range.max as i64,
            })
        };

        Ok(state)
    }

    fn write_property(&self, id: &NodeId, value: PropertyValue) -> DeviceResult {
        let node_id = DSControl::from_node_id(id)?;

        let control::DSNodeId::Control(control) = node_id;

        let raw = match (control.is_boolean(), value) {
            (true, PropertyValue::Bool(v)) => {
                DSControlValue {
                    value: if v { 1 } else { 0 },
                    flags: VideoProcAmp_Flags_Manual.0,
                }
            }
            (false, PropertyValue::Integer(v)) => {
                DSControlValue {
                    value: v as i32,
                    flags: VideoProcAmp_Flags_Manual.0,
                }
            }
            _ => return Err(anyhow!("Unexpected property value type for control {control:?}").into()),
        };

        self.set_control_value(control, raw)
            .map_err(|e| anyhow!("Failed to set control value: {e}"))?;

        Ok(())
    }
}

/// Convert DirectShow `VideoProcAmpFlags` / `CameraControlFlags` caps to `AccessMode`.
fn property_access_mode(caps_flags: i32) -> birb_vision_core::AccessMode {
    use birb_vision_core::AccessMode;

    // If the caps_flags has only auto set, it's effectively read-only from the user's perspective
    // (though the driver adjusts it automatically).
    // For simplicity, if neither manual nor auto is set, we assume read-write.
    let has_manual = (caps_flags & VideoProcAmp_Flags_Manual.0) != 0;
    let has_auto = (caps_flags & VideoProcAmp_Flags_Auto.0) != 0;

    match (has_manual, has_auto) {
        (true, _) => AccessMode::ReadWrite,
        (false, true) => AccessMode::ReadOnly,
        (false, false) => AccessMode::ReadWrite,
    }
}

/// Re-export the range/value types used by the device constructor.
pub use control::DSControlRange;
pub use control::DSControlValue;
pub use control::DSNodeId;

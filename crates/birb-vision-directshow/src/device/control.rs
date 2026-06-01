//! DirectShow camera control definitions
//!
//! Maps DirectShow IAMVideoProcAmp and IAMCameraControl properties
//! to the birb-vision-core property system.

use birb_vision_core::{DeviceResult, NodeId};
use serde::{Deserialize, Serialize};
use windows::Win32::Media::DirectShow::{
    CameraControl_Exposure, CameraControl_Focus, CameraControl_Iris,
    CameraControl_Pan, CameraControl_Roll, CameraControl_Tilt, CameraControl_Zoom,
    VideoProcAmp_BacklightCompensation, VideoProcAmp_Brightness, VideoProcAmp_ColorEnable,
    VideoProcAmp_Contrast, VideoProcAmp_Gain, VideoProcAmp_Gamma, VideoProcAmp_Hue,
    VideoProcAmp_Saturation, VideoProcAmp_Sharpness, VideoProcAmp_WhiteBalance,
};

/// Known DirectShow camera controls that can be enumerated.
///
/// These correspond to properties exposed through either
/// [`IAMVideoProcAmp`](windows::Win32::Media::DirectShow::IAMVideoProcAmp)
/// or [`IAMCameraControl`](windows::Win32::Media::DirectShow::IAMCameraControl).
///
/// Use [`DSControl::iter`](strum::IntoEnumIterator::iter) to enumerate all
/// known controls and query the device for support (unsupported controls
/// will fail at the [`GetRange`] call and be silently skipped).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, strum::EnumIter)]
pub enum DSControl {
    // --- IAMVideoProcAmp properties ---
    Brightness,
    Contrast,
    Hue,
    Saturation,
    Sharpness,
    Gamma,
    WhiteBalance,
    BacklightCompensation,
    Gain,
    ColorEnable,
    // --- IAMCameraControl properties ---
    Pan,
    Tilt,
    Roll,
    Zoom,
    Exposure,
    Iris,
    Focus,
}

/// Which COM interface this control belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DSControlKind {
    /// Property on [`IAMVideoProcAmp`](windows::Win32::Media::DirectShow::IAMVideoProcAmp)
    /// (image adjustments: brightness, contrast, hue, etc.).
    ProcAmp,
    /// Property on [`IAMCameraControl`](windows::Win32::Media::DirectShow::IAMCameraControl)
    /// (physical camera adjustments: pan, tilt, zoom, etc.).
    CameraControl,
}

impl DSControl {
    /// Returns which interface this control lives on.
    pub fn kind(&self) -> DSControlKind {
        use DSControl::*;
        match self {
            Brightness | Contrast | Hue | Saturation | Sharpness | Gamma
            | WhiteBalance | BacklightCompensation | Gain | ColorEnable => {
                DSControlKind::ProcAmp
            }
            Pan | Tilt | Roll | Zoom | Exposure | Iris | Focus => {
                DSControlKind::CameraControl
            }
        }
    }

    /// Returns the raw `i32` property ID for this control.
    pub fn property_id(&self) -> i32 {
        use DSControl::*;
        match self {
            Brightness => VideoProcAmp_Brightness.0,
            Contrast => VideoProcAmp_Contrast.0,
            Hue => VideoProcAmp_Hue.0,
            Saturation => VideoProcAmp_Saturation.0,
            Sharpness => VideoProcAmp_Sharpness.0,
            Gamma => VideoProcAmp_Gamma.0,
            WhiteBalance => VideoProcAmp_WhiteBalance.0,
            BacklightCompensation => VideoProcAmp_BacklightCompensation.0,
            Gain => VideoProcAmp_Gain.0,
            ColorEnable => VideoProcAmp_ColorEnable.0,
            Pan => CameraControl_Pan.0,
            Tilt => CameraControl_Tilt.0,
            Roll => CameraControl_Roll.0,
            Zoom => CameraControl_Zoom.0,
            Exposure => CameraControl_Exposure.0,
            Iris => CameraControl_Iris.0,
            Focus => CameraControl_Focus.0,
        }
    }

    /// Whether this control is boolean (on/off) vs a ranged value.
    pub fn is_boolean(&self) -> bool {
        use DSControl::*;
        matches!(self, BacklightCompensation | ColorEnable)
    }

    /// Convert this control into a [`NodeId`] for the property system.
    pub fn into_node_id(&self) -> DeviceResult<NodeId> {
        NodeId::try_serialyze_value(DSNodeId::Control(*self))
    }

    /// Parse a [`NodeId`] back into a [`DSNodeId`].
    pub fn from_node_id(id: &NodeId) -> DeviceResult<DSNodeId> {
        id.clone().try_deserialize_value()
    }
}

/// The deserializable node-id payload for DirectShow controls.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DSNodeId {
    /// A known DirectShow camera control.
    Control(DSControl),
}

/// Range metadata returned by
/// [`IAMVideoProcAmp::GetRange`](windows::Win32::Media::DirectShow::IAMVideoProcAmp::GetRange)
/// or [`IAMCameraControl::GetRange`](windows::Win32::Media::DirectShow::IAMCameraControl::GetRange).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DSControlRange {
    /// Minimum value.
    pub min: i32,
    /// Maximum value.
    pub max: i32,
    /// Step size.
    pub stepping_delta: i32,
    /// Default value.
    pub default: i32,
    /// Capability flags (`VideoProcAmpFlags` / `CameraControlFlags`).
    pub caps_flags: i32,
}

/// Current value returned by
/// [`IAMVideoProcAmp::Get`](windows::Win32::Media::DirectShow::IAMVideoProcAmp::Get)
/// or [`IAMCameraControl::Get`](windows::Win32::Media::DirectShow::IAMCameraControl::Get).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DSControlValue {
    /// The raw control value.
    pub value: i32,
    /// Control flags (`VideoProcAmpFlags` / `CameraControlFlags`).
    pub flags: i32,
}

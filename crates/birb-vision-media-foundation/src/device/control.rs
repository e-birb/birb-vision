use num_enum::{IntoPrimitive, TryFromPrimitive};
use windows::Win32::Media::DirectShow::{
    CameraControl_Exposure, CameraControl_Focus, CameraControl_Iris, CameraControl_Pan,
    CameraControl_Tilt, CameraControl_Zoom, VideoProcAmp_BacklightCompensation,
    VideoProcAmp_Brightness, VideoProcAmp_Contrast, VideoProcAmp_Gain, VideoProcAmp_Gamma,
    VideoProcAmp_Hue, VideoProcAmp_Saturation, VideoProcAmp_Sharpness, VideoProcAmp_WhiteBalance,
};

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Eq)]
pub enum MFKnownControlKind {
    Boolean,
    Range,
}

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
#[repr(i32)]
pub enum MFKnownControl {
    Brightness,
    Contrast,
    Hue,
    Saturation,
    Sharpness,
    Gamma,
    WhiteBalance,
    BacklightComp,
    Gain,
    Pan,
    Tilt,
    Zoom,
    Exposure,
    Iris,
    Focus,
    //Other(i32),
}

impl MFKnownControl {
    // TODO use strum
    pub const ALL: &[Self] = &[
        Self::Brightness,
        Self::Contrast,
        Self::Hue,
        Self::Saturation,
        Self::Sharpness,
        Self::Gamma,
        Self::WhiteBalance,
        Self::BacklightComp,
        Self::Gain,
        Self::Pan,
        Self::Tilt,
        Self::Zoom,
        Self::Exposure,
        Self::Iris,
        Self::Focus,
    ];

    pub(super) fn control_id(&self) -> Option<MFControlId> {
        // see https://github.com/l1npengtul/nokhwa/blob/aabdaeb0623208a31707ea838dfed555282e2890/nokhwa-bindings-windows/src/lib.rs#L380
        let control_id = match self {
            Self::Brightness => MFControlId::ProcAmp(VideoProcAmp_Brightness.0),
            Self::Contrast => MFControlId::ProcAmp(VideoProcAmp_Contrast.0),
            Self::Hue => MFControlId::ProcAmp(VideoProcAmp_Hue.0),
            Self::Saturation => MFControlId::ProcAmp(VideoProcAmp_Saturation.0),
            Self::Sharpness => MFControlId::ProcAmp(VideoProcAmp_Sharpness.0),
            Self::Gamma => MFControlId::ProcAmp(VideoProcAmp_Gamma.0),
            Self::WhiteBalance => MFControlId::ProcAmp(VideoProcAmp_WhiteBalance.0),
            Self::BacklightComp => MFControlId::ProcAmp(VideoProcAmp_BacklightCompensation.0),
            Self::Gain => MFControlId::ProcAmp(VideoProcAmp_Gain.0), // !
            Self::Pan => MFControlId::CameraControl(CameraControl_Pan.0), // !
            Self::Tilt => MFControlId::CameraControl(CameraControl_Tilt.0), // !
            Self::Zoom => MFControlId::CameraControl(CameraControl_Zoom.0), // !
            Self::Exposure => MFControlId::CameraControl(CameraControl_Exposure.0),
            Self::Iris => MFControlId::CameraControl(CameraControl_Iris.0), // !
            Self::Focus => MFControlId::CameraControl(CameraControl_Focus.0), // !
        };

        Some(control_id)
    }

    pub const fn kind(&self) -> MFKnownControlKind {
        match self {
            Self::Brightness => MFKnownControlKind::Range,
            Self::Contrast => MFKnownControlKind::Range,
            Self::Hue => MFKnownControlKind::Range,
            Self::Saturation => MFKnownControlKind::Range,
            Self::Sharpness => MFKnownControlKind::Range,
            Self::Gamma => MFKnownControlKind::Range,
            Self::WhiteBalance => MFKnownControlKind::Range,
            Self::BacklightComp => MFKnownControlKind::Boolean,
            Self::Gain => MFKnownControlKind::Range,
            Self::Pan => MFKnownControlKind::Range,
            Self::Tilt => MFKnownControlKind::Range,
            Self::Zoom => MFKnownControlKind::Range,
            Self::Exposure => MFKnownControlKind::Range,
            Self::Iris => MFKnownControlKind::Range,
            Self::Focus => MFKnownControlKind::Range,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Eq)]
pub(super) enum MFControlId {
    ProcAmp(i32),
    CameraControl(i32),
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MFControlRange {
    pub min: i32,
    pub max: i32,
    pub default: i32,
    pub stepping_delta: i32,
    pub caps_flags: i32,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MFControlValue {
    pub value: i32,
    pub flags: i32,
}

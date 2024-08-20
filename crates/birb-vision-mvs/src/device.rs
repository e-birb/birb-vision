use std::fmt::Debug;

use crate::*;

#[derive(Clone)]
pub struct DeviceInfo {
    pub(crate) cx: MVContext,
    pub(crate) info: mvs_sys::MV_CC_DEVICE_INFO,
}

impl DeviceInfo {
    pub fn major_version(&self) -> u16 {
        self.info.nMajorVer
    }

    pub fn minor_version(&self) -> u16 {
        self.info.nMinorVer
    }

    pub fn mac_address(&self) -> u64 {
        ((self.info.nMacAddrHigh as u64) << 32) | self.info.nMacAddrLow as u64
    }

    pub fn transport_layer_type(&self) -> TransportLayerType {
        TransportLayerType::from_u32(self.info.nTLayerType)
    }

    pub fn device_type_info(&self) -> DeviceTypeInfo {
        DeviceTypeInfo(self.info.nDevTypeInfo)
    }

    pub fn special_info(&self) -> SpecialDeviceInfo {
        todo!()
    }

    pub fn into_device(self, log: bool) -> Result<MVDevice, MVError> {
        MVDevice::new(self, log)
    }
}

impl Debug for DeviceInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeviceInfo")
            .field("major_version", &self.major_version())
            .field("minor_version", &self.minor_version())
            .field("mac_address", &format_args!("{:#018x}", self.mac_address()))
            .field("transport_layer_type", &self.transport_layer_type())
            .field("device_type_info", &self.device_type_info())
            //.field("special_info", &self.special_info())
            .finish()
    }
}

/// Information about the type of a device.
///
/// Bytes:
/// - From inline documentation:
///   * 7 - 0 bit: Reserved
///   * 15 - 8 bit: Product Subtype
///   * 23 - 16 bit: Product Type
///   * 31 - 24 bit: Product Line
/// - From online documentation:
///   * 7 - 0 bit: Reserved
///   * 15 - 8 bit: Product sub-category
///   * 23 - 16 bit: Product category
///   * 31 - 24 bit: Product line (e.g., 0x01 standard product, 0x02 3D product, 0x03 intelligent ID product).
///
/// See [`DeviceInfo::device_type_info`].
#[derive(Clone, Copy)]
pub struct DeviceTypeInfo(u32);

impl Debug for DeviceTypeInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeviceTypeInfo")
            .field("bits", &format_args!("{:#010x}", self.0))
            .field(
                "product_subcategory",
                &format_args!("{:#04x}", self.product_subcategory()),
            )
            .field(
                "product_category",
                &format_args!("{:#04x}", self.product_category()),
            )
            .field(
                "product_line",
                &format_args!("{:#04x}", self.product_line()),
            )
            .finish()
    }
}

impl DeviceTypeInfo {
    pub fn product_subcategory(&self) -> u8 {
        (self.0 >> 8) as u8
    }

    pub fn product_category(&self) -> u8 {
        (self.0 >> 16) as u8
    }

    pub fn product_line(&self) -> u8 {
        (self.0 >> 24) as u8
    }
}

/// The type of transport layer used by a device.
///
/// See [`DeviceInfo::transport_layer_type`].
#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum TransportLayerType {
    Unknown = mvs_sys::MV_UNKNOW_DEVICE,
    Gige = mvs_sys::MV_GIGE_DEVICE,
    _1394 = mvs_sys::MV_1394_DEVICE,
    Usb = mvs_sys::MV_USB_DEVICE,
    Cameralink = mvs_sys::MV_CAMERALINK_DEVICE,
    VirGige = mvs_sys::MV_VIR_GIGE_DEVICE,
    VirUsb = mvs_sys::MV_VIR_USB_DEVICE,
    GentlGige = mvs_sys::MV_GENTL_GIGE_DEVICE,
    GentlCameralink = mvs_sys::MV_GENTL_CAMERALINK_DEVICE,
    GentlCxp = mvs_sys::MV_GENTL_CXP_DEVICE,
    GentlXof = mvs_sys::MV_GENTL_XOF_DEVICE,
    /// The library returned an unhandeled value.
    Error(u32),
}

impl TransportLayerType {
    /// All handled values of `TransportLayerType`.
    pub const ALL: [Self; 11] = [
        Self::Unknown,
        Self::Gige,
        Self::_1394,
        Self::Usb,
        Self::Cameralink,
        Self::VirGige,
        Self::VirUsb,
        Self::GentlGige,
        Self::GentlCameralink,
        Self::GentlCxp,
        Self::GentlXof,
    ];

    pub fn from_u32(value: u32) -> Self {
        match value {
            mvs_sys::MV_UNKNOW_DEVICE => Self::Unknown,
            mvs_sys::MV_GIGE_DEVICE => Self::Gige,
            mvs_sys::MV_1394_DEVICE => Self::_1394,
            mvs_sys::MV_USB_DEVICE => Self::Usb,
            mvs_sys::MV_CAMERALINK_DEVICE => Self::Cameralink,
            mvs_sys::MV_VIR_GIGE_DEVICE => Self::VirGige,
            mvs_sys::MV_VIR_USB_DEVICE => Self::VirUsb,
            mvs_sys::MV_GENTL_GIGE_DEVICE => Self::GentlGige,
            mvs_sys::MV_GENTL_CAMERALINK_DEVICE => Self::GentlCameralink,
            mvs_sys::MV_GENTL_CXP_DEVICE => Self::GentlCxp,
            mvs_sys::MV_GENTL_XOF_DEVICE => Self::GentlXof,
            value => Self::Error(value),
        }
    }

    pub fn code(&self) -> u32 {
        match self {
            Self::Unknown => mvs_sys::MV_UNKNOW_DEVICE,
            Self::Gige => mvs_sys::MV_GIGE_DEVICE,
            Self::_1394 => mvs_sys::MV_1394_DEVICE,
            Self::Usb => mvs_sys::MV_USB_DEVICE,
            Self::Cameralink => mvs_sys::MV_CAMERALINK_DEVICE,
            Self::VirGige => mvs_sys::MV_VIR_GIGE_DEVICE,
            Self::VirUsb => mvs_sys::MV_VIR_USB_DEVICE,
            Self::GentlGige => mvs_sys::MV_GENTL_GIGE_DEVICE,
            Self::GentlCameralink => mvs_sys::MV_GENTL_CAMERALINK_DEVICE,
            Self::GentlCxp => mvs_sys::MV_GENTL_CXP_DEVICE,
            Self::GentlXof => mvs_sys::MV_GENTL_XOF_DEVICE,
            Self::Error(value) => *value,
        }
    }
}

pub enum SpecialDeviceInfo {
    GigE(GigEDeviceInfo),
    Usb(UsbDeviceInfo),
    Cameralink(CameralinkDeviceInfo),
    CameralinkFramegrabber(CameralinkFramegrabberDeviceInfo),
    CoaXPress(CoaXPressDeviceInfo),
    Xof(XofDeviceInfo),
}

#[allow(dead_code)] // TODO
pub struct GigEDeviceInfo(mvs_sys::MV_GIGE_DEVICE_INFO);

impl GigEDeviceInfo {
    // TODO
}

#[allow(dead_code)] // TODO
pub struct UsbDeviceInfo(mvs_sys::MV_USB3_DEVICE_INFO);

impl UsbDeviceInfo {
    // TODO
}

#[allow(dead_code)] // TODO
pub struct CameralinkDeviceInfo(mvs_sys::MV_CamL_DEV_INFO);

impl CameralinkDeviceInfo {
    // TODO
}

#[allow(dead_code)] // TODO
pub struct CameralinkFramegrabberDeviceInfo(mvs_sys::MV_CML_DEVICE_INFO);

impl CameralinkFramegrabberDeviceInfo {
    // TODO
}

#[allow(dead_code)] // TODO
pub struct CoaXPressDeviceInfo(mvs_sys::MV_CXP_DEVICE_INFO);

impl CoaXPressDeviceInfo {
    // TODO
}

#[allow(dead_code)] // TODO
pub struct XofDeviceInfo(mvs_sys::MV_XOF_DEVICE_INFO);

impl XofDeviceInfo {
    // TODO
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u32)]
pub enum AccessMode {
    /// Exclusive permission, for other apps, the CCP register is only allowed to be read
    Exclusive = mvs_sys::MV_ACCESS_Exclusive,
    /// Preempt permission in mode 5, and then open with exclusive permission
    ExclusiveWithSwitch = mvs_sys::MV_ACCESS_ExclusiveWithSwitch,
    /// Preempt permission in mode 5, and then open with exclusive permission
    Control = mvs_sys::MV_ACCESS_Control,
    /// Preempt permission in mode 5, and then open with control permission
    ControlWithSwitch = mvs_sys::MV_ACCESS_ControlWithSwitch,
    /// Open with control permission that can be preempted
    ControlSwitchEnable = mvs_sys::MV_ACCESS_ControlSwitchEnable,
    /// Preempt permission in mode 5, and then open with control permission that can be preempted
    ControlSwitchEnableWithKey = mvs_sys::MV_ACCESS_ControlSwitchEnableWithKey,
    /// Open device with reading mode, suitable under control permission
    Monitor = mvs_sys::MV_ACCESS_Monitor,
}

impl AccessMode {
    pub fn from_u32(value: u32) -> Self {
        match value {
            mvs_sys::MV_ACCESS_Exclusive => Self::Exclusive,
            mvs_sys::MV_ACCESS_ExclusiveWithSwitch => Self::ExclusiveWithSwitch,
            mvs_sys::MV_ACCESS_Control => Self::Control,
            mvs_sys::MV_ACCESS_ControlWithSwitch => Self::ControlWithSwitch,
            mvs_sys::MV_ACCESS_ControlSwitchEnable => Self::ControlSwitchEnable,
            mvs_sys::MV_ACCESS_ControlSwitchEnableWithKey => Self::ControlSwitchEnableWithKey,
            mvs_sys::MV_ACCESS_Monitor => Self::Monitor,
            value => panic!("Unknown access mode: {}", value),
        }
    }
}

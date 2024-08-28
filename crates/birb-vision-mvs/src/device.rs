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

    pub fn special_info(&self) -> Option<SpecialDeviceInfo> {
        use TransportLayerType::*;

        // TODO maybe transport_layer_type is the wrong way to go here, but I don't know what else to use
        match self.transport_layer_type() {
            Unknown => None,
            Gige => Some(SpecialDeviceInfo::GigE(GigEDeviceInfo(unsafe { self.info.SpecialInfo.stGigEInfo }))),
            _1394 => None, // TODO ???
            Usb => Some(SpecialDeviceInfo::Usb(UsbDeviceInfo(unsafe { self.info.SpecialInfo.stUsb3VInfo }))),
            Cameralink => Some(SpecialDeviceInfo::Cameralink(CameralinkDeviceInfo(unsafe { self.info.SpecialInfo.stCamLInfo }))),
            VirGige => None, // TODO ???
            VirUsb => None, // TODO ???
            GentlGige => None, // TODO ???
            GentlCameralink => None, // TODO ???
            GentlCxp => Some(SpecialDeviceInfo::CoaXPress(CoaXPressDeviceInfo(unsafe { self.info.SpecialInfo.stCXPInfo }))),
            GentlXof => Some(SpecialDeviceInfo::Xof(XofDeviceInfo(unsafe { self.info.SpecialInfo.stXoFInfo }))),
            Error(_) => None,
        }
    }

    pub fn is_device_accessible(&self, mode: AccessMode) -> bool {
        unsafe { (self.cx.ffi().MV_CC_IsDeviceAccessible)(&self.info as *const _ as *mut _, mode as _) }
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
            .field("special_info", &self.special_info())
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
    pub fn value(&self) -> u32 {
        self.0
    }

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

    pub const DEFAULT_TRANSPORT_LAYERS: [Self; 2] = [
        Self::Gige,
        Self::Usb,
    ];

    pub fn name(&self) -> Option<String> {
        match self {
            Self::Unknown => None,
            Self::Gige => Some("Gige".into()),
            Self::_1394 => Some("_1394".into()),
            Self::Usb => Some("Usb".into()),
            Self::Cameralink => Some("Cameralink".into()),
            Self::VirGige => Some("VirGige".into()),
            Self::VirUsb => Some("VirUsb".into()),
            Self::GentlGige => Some("GentlGige".into()),
            Self::GentlCameralink => Some("GentlCameralink".into()),
            Self::GentlCxp => Some("GentlCxp".into()),
            Self::GentlXof => Some("GentlXof".into()),
            Self::Error(_) => None,
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        for ty in &Self::ALL {
            if ty.name().as_ref().map(|n| n.as_str()) == Some(name) {
                return Some(ty.clone());
            }
        }
        None
    }

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

#[derive(Debug, Clone)]
pub enum SpecialDeviceInfo {
    GigE(GigEDeviceInfo),
    Usb(UsbDeviceInfo),
    Cameralink(CameralinkDeviceInfo),
    CameralinkFramegrabber(CameralinkFramegrabberDeviceInfo),
    CoaXPress(CoaXPressDeviceInfo),
    Xof(XofDeviceInfo),
}

#[derive(Clone)]
#[allow(dead_code)] // TODO
pub struct GigEDeviceInfo(mvs_sys::MV_GIGE_DEVICE_INFO);

impl Debug for GigEDeviceInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GigEDeviceInfo")
            .field("ip_configuration_options", &format_args!("{:#010x}", self.ip_configuration_options()))
            .field("ip_configuration_current", &format_args!("{:#010x}", self.ip_configuration_current()))
            .field("current_ip_address", &format_args!("{:#010x}", self.current_ip_address()))
            .field("current_subnet_mask", &format_args!("{:#010x}", self.current_subnet_mask()))
            .field("default_gateway", &format_args!("{:#010x}", self.default_gateway()))
            .field("manufacturer_name", &self.manufacturer_name())
            .field("model_name", &self.model_name())
            .field("device_version", &self.device_version())
            .field("manufacturer_specific_info", &self.manufacturer_specific_info())
            .field("serial_number", &self.serial_number())
            .field("user_defined_name", &self.user_defined_name())
            .field("network_ip_address", &format_args!("{:#010x}", self.network_ip_address()))
            .finish()
    }
}

impl GigEDeviceInfo {
    pub fn ip_configuration_options(&self) -> u32 {
        self.0.nIpCfgOption
    }

    pub fn ip_configuration_current(&self) -> u32 {
        self.0.nIpCfgCurrent
    }

    pub fn current_ip_address(&self) -> u32 {
        self.0.nCurrentIp
    }

    pub fn current_subnet_mask(&self) -> u32 {
        self.0.nCurrentSubNetMask
    }

    pub fn default_gateway(&self) -> u32 {
        self.0.nDefultGateWay
    }

    pub fn manufacturer_name(&self) -> &CStr {
        CStr::from_bytes_until_nul(&self.0.chManufacturerName).unwrap()
    }

    pub fn model_name(&self) -> &CStr {
        CStr::from_bytes_until_nul(&self.0.chModelName).unwrap()
    }

    pub fn device_version(&self) -> &CStr {
        CStr::from_bytes_until_nul(&self.0.chDeviceVersion).unwrap()
    }

    pub fn manufacturer_specific_info(&self) -> &CStr {
        CStr::from_bytes_until_nul(&self.0.chManufacturerSpecificInfo).unwrap()
    }

    pub fn serial_number(&self) -> &CStr {
        CStr::from_bytes_until_nul(&self.0.chSerialNumber).unwrap()
    }

    pub fn user_defined_name(&self) -> &CStr {
        CStr::from_bytes_until_nul(&self.0.chUserDefinedName).unwrap()
    }

    pub fn network_ip_address(&self) -> u32 {
        self.0.nNetExport
    }
}

#[derive(Clone)]
#[allow(dead_code)] // TODO
pub struct UsbDeviceInfo(mvs_sys::MV_USB3_DEVICE_INFO);

impl Debug for UsbDeviceInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UsbDeviceInfo")
            .field("control_input_endpoint", &self.control_input_endpoint())
            .field("control_output_endpoint", &self.control_output_endpoint())
            .field("stream_endpoint", &self.stream_endpoint())
            .field("event_endpoint", &self.event_endpoint())
            .field("vendor_id", &self.vendor_id())
            .field("product_id", &self.product_id())
            .field("device_number", &self.device_number())
            .field("device_guid", &self.device_guid())
            .field("vendor_name", &self.vendor_name())
            .field("model_name", &self.model_name())
            .field("family_name", &self.family_name())
            .field("device_version", &self.device_version())
            .field("manufacturer_name", &self.manufacturer_name())
            .field("serial_number", &self.serial_number())
            .field("user_defined_name", &self.user_defined_name())
            .field("support_usb_protocol", &self.support_usb_protocol())
            .field("device_address", &format_args!("{:#010x}", self.device_address()))
            .finish()
    }
}

impl UsbDeviceInfo {
    pub fn control_input_endpoint(&self) -> u8 {
        self.0.CrtlInEndPoint
    }

    pub fn control_output_endpoint(&self) -> u8 {
        self.0.CrtlOutEndPoint
    }

    pub fn stream_endpoint(&self) -> u8 {
        self.0.StreamEndPoint
    }

    pub fn event_endpoint(&self) -> u8 {
        self.0.EventEndPoint
    }

    pub fn vendor_id(&self) -> u16 {
        self.0.idVendor
    }

    pub fn product_id(&self) -> u16 {
        self.0.idProduct
    }

    pub fn device_number(&self) -> u32 {
        self.0.nDeviceNumber
    }

    pub fn device_guid(&self) -> &CStr {
        CStr::from_bytes_until_nul(&self.0.chDeviceGUID).unwrap()
    }

    pub fn vendor_name(&self) -> &CStr {
        CStr::from_bytes_until_nul(&self.0.chVendorName).unwrap()
    }

    pub fn model_name(&self) -> &CStr {
        CStr::from_bytes_until_nul(&self.0.chModelName).unwrap()
    }

    pub fn family_name(&self) -> &CStr {
        CStr::from_bytes_until_nul(&self.0.chFamilyName).unwrap()
    }

    pub fn device_version(&self) -> &CStr {
        CStr::from_bytes_until_nul(&self.0.chDeviceVersion).unwrap()
    }

    pub fn manufacturer_name(&self) -> &CStr {
        CStr::from_bytes_until_nul(&self.0.chManufacturerName).unwrap()
    }

    pub fn serial_number(&self) -> &CStr {
        CStr::from_bytes_until_nul(&self.0.chSerialNumber).unwrap()
    }

    pub fn user_defined_name(&self) -> &CStr {
        CStr::from_bytes_until_nul(&self.0.chUserDefinedName).unwrap()
    }

    pub fn support_usb_protocol(&self) -> u32 {
        self.0.nbcdUSB
    }

    pub fn device_address(&self) -> u32 {
        self.0.nDeviceAddress
    }
}

#[derive(Clone)]
#[allow(dead_code)] // TODO
pub struct CameralinkDeviceInfo(mvs_sys::MV_CamL_DEV_INFO);

impl Debug for CameralinkDeviceInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CameralinkDeviceInfo")
            .field("port_id", &self.port_id())
            .field("model_name", &self.model_name())
            .field("family_name", &self.family_name())
            .field("device_version", &self.device_version())
            .field("manufacturer_name", &self.manufacturer_name())
            .field("serial_number", &self.serial_number())
            .finish()
    }
}

impl CameralinkDeviceInfo {
    pub fn port_id(&self) -> &CStr {
        CStr::from_bytes_until_nul(&self.0.chPortID).unwrap()
    }

    pub fn model_name(&self) -> &CStr {
        CStr::from_bytes_until_nul(&self.0.chModelName).unwrap()
    }

    pub fn family_name(&self) -> &CStr {
        CStr::from_bytes_until_nul(&self.0.chFamilyName).unwrap()
    }

    pub fn device_version(&self) -> &CStr {
        CStr::from_bytes_until_nul(&self.0.chDeviceVersion).unwrap()
    }

    pub fn manufacturer_name(&self) -> &CStr {
        CStr::from_bytes_until_nul(&self.0.chManufacturerName).unwrap()
    }

    pub fn serial_number(&self) -> &CStr {
        CStr::from_bytes_until_nul(&self.0.chSerialNumber).unwrap()
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)] // TODO
pub struct CameralinkFramegrabberDeviceInfo(mvs_sys::MV_CML_DEVICE_INFO);

impl CameralinkFramegrabberDeviceInfo {
    // TODO
}

#[derive(Clone)]
#[allow(dead_code)] // TODO
pub struct CoaXPressDeviceInfo(mvs_sys::MV_CXP_DEVICE_INFO);

impl Debug for CoaXPressDeviceInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CoaXPressDeviceInfo")
            .field("interface_id", &self.interface_id())
            .field("vendor_name", &self.vendor_name())
            .field("model_name", &self.model_name())
            .field("manufacturer_info", &self.manufacturer_info())
            .field("device_version", &self.device_version())
            .field("serial_number", &self.serial_number())
            .field("user_defined_name", &self.user_defined_name())
            .field("device_id", &self.device_id())
            .finish()
    }
}

impl CoaXPressDeviceInfo {
    pub fn interface_id(&self) -> &CStr {
        CStr::from_bytes_until_nul(&self.0.chInterfaceID).unwrap()
    }

    pub fn vendor_name(&self) -> &CStr {
        CStr::from_bytes_until_nul(&self.0.chVendorName).unwrap()
    }

    pub fn model_name(&self) -> &CStr {
        CStr::from_bytes_until_nul(&self.0.chModelName).unwrap()
    }

    pub fn manufacturer_info(&self) -> &CStr {
        CStr::from_bytes_until_nul(&self.0.chManufacturerInfo).unwrap()
    }

    pub fn device_version(&self) -> &CStr {
        CStr::from_bytes_until_nul(&self.0.chDeviceVersion).unwrap()
    }

    pub fn serial_number(&self) -> &CStr {
        CStr::from_bytes_until_nul(&self.0.chSerialNumber).unwrap()
    }

    pub fn user_defined_name(&self) -> &CStr {
        CStr::from_bytes_until_nul(&self.0.chUserDefinedName).unwrap()
    }

    pub fn device_id(&self) -> &CStr {
        CStr::from_bytes_until_nul(&self.0.chDeviceID).unwrap()
    }
}

#[derive(Clone)]
#[allow(dead_code)] // TODO
pub struct XofDeviceInfo(mvs_sys::MV_XOF_DEVICE_INFO);

impl Debug for XofDeviceInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("XofDeviceInfo")
            .field("interface_id", &self.interface_id())
            .field("vendor_name", &self.vendor_name())
            .field("model_name", &self.model_name())
            .field("manufacturer_info", &self.manufacturer_info())
            .field("device_version", &self.device_version())
            .field("serial_number", &self.serial_number())
            .field("user_defined_name", &self.user_defined_name())
            .field("device_id", &self.device_id())
            .finish()
    }
}

impl XofDeviceInfo {
    pub fn interface_id(&self) -> &CStr {
        CStr::from_bytes_until_nul(&self.0.chInterfaceID).unwrap()
    }

    pub fn vendor_name(&self) -> &CStr {
        CStr::from_bytes_until_nul(&self.0.chVendorName).unwrap()
    }

    pub fn model_name(&self) -> &CStr {
        CStr::from_bytes_until_nul(&self.0.chModelName).unwrap()
    }

    pub fn manufacturer_info(&self) -> &CStr {
        CStr::from_bytes_until_nul(&self.0.chManufacturerInfo).unwrap()
    }

    pub fn device_version(&self) -> &CStr {
        CStr::from_bytes_until_nul(&self.0.chDeviceVersion).unwrap()
    }

    pub fn serial_number(&self) -> &CStr {
        CStr::from_bytes_until_nul(&self.0.chSerialNumber).unwrap()
    }

    pub fn user_defined_name(&self) -> &CStr {
        CStr::from_bytes_until_nul(&self.0.chUserDefinedName).unwrap()
    }

    pub fn device_id(&self) -> &CStr {
        CStr::from_bytes_until_nul(&self.0.chDeviceID).unwrap()
    }
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

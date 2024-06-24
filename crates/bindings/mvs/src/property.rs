use std::fmt::Debug;

use mvs_sys::{MVCC_ENUMENTRY, MVCC_STRINGVALUE, MV_MAX_XML_SYMBOLIC_NUM};

pub trait PropId {
    fn property_name(&self) -> &str;
}

impl PropId for &str {
    fn property_name(&self) -> &str {
        self
    }
}

impl PropId for String {
    fn property_name(&self) -> &str {
        self
    }
}

pub(crate) fn prop_id_to_c_string<T: PropId>(prop_id: T) -> std::ffi::CString {
    std::ffi::CString::new(prop_id.property_name()).unwrap()
}

#[derive(Clone, Copy)]
pub struct IntValue(pub(crate) mvs_sys::MVCC_INTVALUE);

impl IntValue {
    pub fn current(&self) -> u32 {
        self.0.nCurValue
    }

    pub fn min(&self) -> u32 {
        self.0.nMin
    }

    pub fn max(&self) -> u32 {
        self.0.nMax
    }

    pub fn increment(&self) -> u32 {
        self.0.nInc
    }
}

impl Debug for IntValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IntValue")
            .field("current", &self.current())
            .field("min", &self.min())
            .field("max", &self.max())
            .field("increment", &self.increment())
            .finish()
    }
}

#[derive(Clone, Copy)]
pub struct EnumValue(pub(crate) mvs_sys::MVCC_ENUMVALUE);

impl EnumValue {
    pub const MAX_SUPPORTED_VALUES: usize = MV_MAX_XML_SYMBOLIC_NUM as _;

    pub fn current_value(&self) -> u32 {
        self.0.nCurValue
    }

    pub fn support(&self) -> &[u32] {
        &self.0.nSupportValue[..self.0.nSupportedNum as usize]
    }
}

impl Debug for EnumValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EnumValue")
            .field("current_value", &self.current_value())
            .field("support", &self.support())
            .finish()
    }
}

#[derive(Clone, Copy)]
pub struct EnumEntrySymbolic(pub(crate) MVCC_ENUMENTRY);

impl EnumEntrySymbolic {
    pub fn value(&self) -> u32 {
        self.0.nValue
    }

    pub fn symbolic(&self) -> &str {
        unsafe {
            std::ffi::CStr::from_ptr(self.0.chSymbolic.as_ptr())
                .to_str()
                .expect("symbolic value is not valid utf-8")
        }
    }
}

impl Debug for EnumEntrySymbolic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EnumEntrySymbolic")
            .field("value", &self.value())
            .field("symbolic", &self.symbolic())
            .finish()
    }
}

#[derive(Clone, Copy)]
pub struct FloatValue(pub(crate) mvs_sys::MVCC_FLOATVALUE);

impl FloatValue {
    pub fn current(&self) -> f32 {
        self.0.fCurValue
    }

    pub fn min(&self) -> f32 {
        self.0.fMin
    }

    pub fn max(&self) -> f32 {
        self.0.fMax
    }
}

impl Debug for FloatValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FloatValue")
            .field("current", &self.current())
            .field("min", &self.min())
            .field("max", &self.max())
            .finish()
    }
}

#[derive(Clone, Copy)]
pub struct StringValue(pub(crate) MVCC_STRINGVALUE);

impl StringValue {
    pub fn max_length(&self) -> usize {
        self.0.nMaxLength as usize
    }

    pub fn current_value(&self) -> &str {
        unsafe {
            std::ffi::CStr::from_ptr(self.0.chCurValue.as_ptr())
                .to_str()
                .expect("current value is not valid utf-8")
        }
    }
}

impl Debug for StringValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StringValue")
            .field("max_length", &self.max_length())
            .field("current_value", &self.current_value())
            .finish()
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(i32)]
pub enum XmlAccessMode {
    /// Not implemented.
    NotImplemented = mvs_sys::MV_XML_AccessMode_AM_NI,
    /// Not available.
    NotAvailable = mvs_sys::MV_XML_AccessMode_AM_NA,
    /// Write only.
    WriteOnly = mvs_sys::MV_XML_AccessMode_AM_WO,
    /// Read only.
    ReadOnly = mvs_sys::MV_XML_AccessMode_AM_RO,
    /// Read and write.
    ReadWrite = mvs_sys::MV_XML_AccessMode_AM_RW,
    /// Object is not initialized.
    Undefined = mvs_sys::MV_XML_AccessMode_AM_Undefined,
    /// Used internally for AccessMode cycle detection.
    CycleDetect = mvs_sys::MV_XML_AccessMode_AM_CycleDetect,
}

impl XmlAccessMode {
    pub fn from_i32(value: i32) -> Self {
        match value {
            mvs_sys::MV_XML_AccessMode_AM_NI => Self::NotImplemented,
            mvs_sys::MV_XML_AccessMode_AM_NA => Self::NotAvailable,
            mvs_sys::MV_XML_AccessMode_AM_WO => Self::WriteOnly,
            mvs_sys::MV_XML_AccessMode_AM_RO => Self::ReadOnly,
            mvs_sys::MV_XML_AccessMode_AM_RW => Self::ReadWrite,
            mvs_sys::MV_XML_AccessMode_AM_Undefined => Self::Undefined,
            mvs_sys::MV_XML_AccessMode_AM_CycleDetect => Self::CycleDetect,
            _ => panic!("unhandled value: {}", value),
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(i32)]
pub enum XmlInterfaceType {
    /// IValue interface
    IValue = mvs_sys::MV_XML_InterfaceType_IFT_IValue,
    /// IBase interface
    IBase = mvs_sys::MV_XML_InterfaceType_IFT_IBase,
    /// IInteger interface
    IInteger = mvs_sys::MV_XML_InterfaceType_IFT_IInteger,
    /// IBoolean interface
    IBoolean = mvs_sys::MV_XML_InterfaceType_IFT_IBoolean,
    /// ICommand interface
    ICommand = mvs_sys::MV_XML_InterfaceType_IFT_ICommand,
    /// IFloat interface
    IFloat = mvs_sys::MV_XML_InterfaceType_IFT_IFloat,
    /// IString interface
    IString = mvs_sys::MV_XML_InterfaceType_IFT_IString,
    /// IRegister interface
    IRegister = mvs_sys::MV_XML_InterfaceType_IFT_IRegister,
    /// ICategory interface
    ICategory = mvs_sys::MV_XML_InterfaceType_IFT_ICategory,
    /// IEnumeration interface
    IEnumeration = mvs_sys::MV_XML_InterfaceType_IFT_IEnumeration,
    /// IEnumEntry interface
    IEnumEntry = mvs_sys::MV_XML_InterfaceType_IFT_IEnumEntry,
    /// IPort interface
    IPort = mvs_sys::MV_XML_InterfaceType_IFT_IPort,
}

impl XmlInterfaceType {
    pub fn from_i32(value: i32) -> Self {
        match value {
            mvs_sys::MV_XML_InterfaceType_IFT_IValue => Self::IValue,
            mvs_sys::MV_XML_InterfaceType_IFT_IBase => Self::IBase,
            mvs_sys::MV_XML_InterfaceType_IFT_IInteger => Self::IInteger,
            mvs_sys::MV_XML_InterfaceType_IFT_IBoolean => Self::IBoolean,
            mvs_sys::MV_XML_InterfaceType_IFT_ICommand => Self::ICommand,
            mvs_sys::MV_XML_InterfaceType_IFT_IFloat => Self::IFloat,
            mvs_sys::MV_XML_InterfaceType_IFT_IString => Self::IString,
            mvs_sys::MV_XML_InterfaceType_IFT_IRegister => Self::IRegister,
            mvs_sys::MV_XML_InterfaceType_IFT_ICategory => Self::ICategory,
            mvs_sys::MV_XML_InterfaceType_IFT_IEnumeration => Self::IEnumeration,
            mvs_sys::MV_XML_InterfaceType_IFT_IEnumEntry => Self::IEnumEntry,
            mvs_sys::MV_XML_InterfaceType_IFT_IPort => Self::IPort,
            _ => panic!("unhandled value: {}", value),
        }
    }
}

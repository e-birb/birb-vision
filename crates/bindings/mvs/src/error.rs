

macro_rules! define_sub_error {
    (
        $short:literal
        $(#[$attr:meta])*
        $error:ident {
            $(
                $variant_short:literal
                $(#[$variant_attr:meta])*
                $name:ident = $value:ident,
            )*
        }
    ) => {
        #[doc=$short]
        #[doc="\n\n"]
        $(#[$attr])*
        #[repr(u32)]
        #[derive(Copy, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
        #[allow(non_camel_case_types)]
        pub enum $error {
            $(
                $(#[$variant_attr])*
                #[doc="\n\n"]
                #[doc=concat!(
                    "Corresponding error code: [`mvs_sys::",
                    stringify!($value),
                    "`]",
                )]
                $name = $value,
            )*
        }

        impl $error {
            #[inline(always)]
            pub fn native_from_code(code: u32) -> Option<Self> {
                match code {
                    $(
                        $value => Some($error::$name),
                    )*
                    _ => None,
                }
            }

            #[inline(always)]
            pub fn native_code(&self) -> u32 {
                match self {
                    $(
                        $error::$name => $value,
                    )*
                }
            }

            #[inline(always)]
            pub fn native_name(&self) -> &'static str {
                match self {
                    $(
                        $error::$name => stringify!($name),
                    )*
                }
            }

            #[inline(always)]
            pub fn mv_native_name(&self) -> &'static str {
                match self {
                    $(
                        $error::$name => stringify!($value),
                    )*
                }
            }
        }
    };
}

macro_rules! define_error_enum {
    ($prefix:literal $error:ident {
        native: {
            $(
                $short:literal
                $(#[$attr:meta])*
                $name:ident = $value:ident,
            )*
        }
        other: {
            $($other:tt)*
        }
    }) => {
        #[repr(u32)]
        #[derive(Copy, Clone, Eq, PartialEq)]
        #[allow(non_camel_case_types)]
        pub enum $error {
            $(
                $(#[$attr])*
                #[doc="\n\n"]
                #[doc=concat!(
                    "Corresponding error code: [`mvs_sys::",
                    stringify!($value),
                    "`]",
                )]
                $name = $value,
            )*
            $($other)*
        }

        impl $error {
            #[inline(always)]
            pub fn native_from_code(code: u32) -> Option<Self> {
                match code {
                    $(
                        $value => Some($error::$name),
                    )*
                    _ => None,
                }
            }

            #[inline(always)]
            pub fn native_code(&self) -> Option<u32> {
                match self {
                    $(
                        $error::$name => Some($value),
                    )*
                    _ => None,
                }
            }

            #[inline(always)]
            pub fn native_name(&self) -> Option<&'static str> {
                match self {
                    $(
                        $error::$name => Some(concat!($prefix, stringify!($name))),
                    )*
                    _ => None,
                }
            }

            #[inline(always)]
            pub fn mv_native_name(&self) -> Option<&'static str> {
                match self {
                    $(
                        $error::$name => Some(stringify!($value)),
                    )*
                    _ => None,
                }
            }

            #[inline(always)]
            pub fn is_native(&self) -> bool {
                self.native_code().is_some()
            }
        }
    };
}

use std::fmt::Debug;

use mvs_sys::*;

define_error_enum! {
    "" MVSError {
        native: {
            "invalid handle"
            ///
            /// This error code indicates that the handle is null or invalid.
            HANDLE = MV_E_HANDLE,

            "function not supported"
            SUPPORT = MV_E_SUPPORT,

            "buffer overflow"
            BUFOVER = MV_E_BUFOVER,

            "function calling order error"
            ///
            /// This could happen, for example, if the `MV_CC_Initialize` has not been called before.
            CALLORDER = MV_E_CALLORDER,

            "invalid parameter"
            ///
            /// This error code indicates that some parameters passed to the function are invalid.
            PARAMETER = MV_E_PARAMETER,

            "applying resource failed"
            RESOURCE = MV_E_RESOURCE,

            "no data"
            NODATA = MV_E_NODATA,

            "precondition error, or running environment changed"
            PRECONDITION = MV_E_PRECONDITION,

            "version mismatches"
            VERSION = MV_E_VERSION,

            "insufficient memory"
            NOENOUGH_BUF = MV_E_NOENOUGH_BUF,

            "abnormal image, maybe incomplete image because of lost packet"
            ABNORMAL_IMAGE = MV_E_ABNORMAL_IMAGE,

            "load library failed"
            LOAD_LIBRARY = MV_E_LOAD_LIBRARY,

            "no Avaliable Buffer"
            NOOUTBUF = MV_E_NOOUTBUF,

            "encryption error"
            ENCRYPT = MV_E_ENCRYPT,

            "open file error"
            OPENFILE = MV_E_OPENFILE,

            "unknown error"
            UNKNOW = MV_E_UNKNOW,
        }
        other: {
            GenICam(GenICamError),
            Usb(UsbError),
            Upg(UpgError),
            Alg(AlgError),
            GigE(GigEError),
            Exception(ExceptionError),
            Other(u32) = 0,
        }
    }
}

impl MVSError {
    pub fn result_from_code(code: i32) -> Result<(), Self> {
        match MVSError::from_code(code as u32) {
            Some(e) => Err(e),
            None => Ok(()),
        }
    }

    pub fn from_code(code: u32) -> Option<Self> {
        if code == mvs_sys::MV_OK {
            None
        } else if let Some(native) = Self::native_from_code(code) {
            Some(native)
        } else if let Some(genicam) = GenICamError::native_from_code(code) {
            Some(MVSError::GenICam(genicam))
        } else if let Some(usb) = UsbError::native_from_code(code) {
            Some(MVSError::Usb(usb))
        } else {
            Some(MVSError::Other(code))
        }
    }

    pub fn code(&self) -> u32 {
        if let Some(native) = self.native_code() {
            native
        } else {
            match self {
                MVSError::Other(code) => *code,
                MVSError::GenICam(genicam) => genicam.native_code(),
                MVSError::Usb(usb) => usb.native_code(),
                MVSError::Upg(upg) => upg.native_code(),
                MVSError::Alg(alg) => alg.native_code(),
                MVSError::GigE(gige) => gige.native_code(),
                MVSError::Exception(exception) => exception.native_code(),
                _ => unreachable!(),
            }
        }
    }

    fn name(&self) -> &'static str {
        if let Some(native) = self.native_name() {
            native
        } else {
            match self {
                MVSError::Other(_) => "Other",
                MVSError::GenICam(genicam) => genicam.native_name(),
                MVSError::Usb(usb) => usb.native_name(),
                MVSError::Upg(upg) => upg.native_name(),
                MVSError::Alg(alg) => alg.native_name(),
                MVSError::GigE(gige) => gige.native_name(),
                MVSError::Exception(exception) => exception.native_name(),
                _ => unreachable!(),
            }
        }
    }

    pub fn mv_name(&self) -> Option<&'static str> {
        if let Some(native) = self.mv_native_name() {
            Some(native)
        } else {
            match self {
                MVSError::Other(_) => None,
                MVSError::GenICam(genicam) => Some(genicam.mv_native_name()),
                MVSError::Usb(usb) => Some(usb.mv_native_name()),
                MVSError::Upg(upg) => Some(upg.mv_native_name()),
                MVSError::Alg(alg) => Some(alg.mv_native_name()),
                MVSError::GigE(gige) => Some(gige.mv_native_name()),
                MVSError::Exception(exception) => Some(exception.mv_native_name()),
                _ => unreachable!(),
            }
        }
    }
}

impl std::fmt::Debug for MVSError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let code = self.code();
        let name = self.name();
        match self.mv_name() {
            Some(mvs_name) => write!(f, "{name}({code:#08x}={mvs_name})"),
            None => write!(f, "{name}({code:#08x})"),
        }
    }
}

impl std::fmt::Display for MVSError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //#[allow(unreachable_patterns)] // <- for doc purposes, when no errors are defined
        //match self {
        //    MVSError::Other(code) => write!(f, "Unknown error code: {:#08x}", code),
        //    _ => write!(f, "{self:?} ({:#08x})", self.code()),
        //}
        Debug::fmt(self, f) // TODO use a proper implementation
    }
}

impl std::error::Error for MVSError {}

impl PartialOrd for MVSError {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.code().cmp(&other.code()))
    }
}

impl Ord for MVSError {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.code().cmp(&other.code())
    }
}

impl std::hash::Hash for MVSError {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.code().hash(state)
    }
}

define_sub_error! {
    "GenICam error"
    GenICamError {
        "general error"
        GENERIC = MV_E_GC_GENERIC,

        "illegal argument"
        ARGUMENT = MV_E_GC_ARGUMENT,

        "value out of range"
        RANGE = MV_E_GC_RANGE,

        "property"
        PROPERTY = MV_E_GC_PROPERTY,

        "running environment error"
        RUNTIME = MV_E_GC_RUNTIME,

        "logical error"
        LOGICAL = MV_E_GC_LOGICAL,

        "node accessing condition error"
        ACCESS = MV_E_GC_ACCESS,

        "timeout"
        TIMEOUT = MV_E_GC_TIMEOUT,

        "transformation exception"
        DYNAMICCAST = MV_E_GC_DYNAMICCAST,

        "GenICam unknown error"
        UNKNOW = MV_E_GC_UNKNOW,
    }
}

define_sub_error! {
    "USB error"
    UsbError {
        "readin usb error"
        READ = MV_E_USB_READ,

        "writing usb error"
        WRITE = MV_E_USB_WRITE,

        "device exception"
        DEVICE = MV_E_USB_DEVICE,

        "GenICam error"
        GENICAM = MV_E_USB_GENICAM,

        "insufficient bandwidth"
        BANDWIDTH = MV_E_USB_BANDWIDTH,

        "driver mismatch or unmounted driver"
        DRIVER = MV_E_USB_DRIVER,

        "unknown error"
        UNKNOW = MV_E_USB_UNKNOW,
    }
}

define_sub_error! {
    "Upgrade error"
    UpgError {
        "firmware mismatches"
        FILE_MISMATCH = MV_E_UPG_FILE_MISMATCH,

        "firmware language mismatches"
        LANGUSGE_MISMATCH = MV_E_UPG_LANGUSGE_MISMATCH,

        "Upgrading conflicted (repeated upgrading requests during device upgrade)"
        CONFLICT = MV_E_UPG_CONFLICT,

        "Camera internal error during upgrade"
        INNER_ERR = MV_E_UPG_INNER_ERR,

        "Unknown error during upgrade"
        UNKNOW = MV_E_UPG_UNKNOW,
    }
}

define_sub_error! {
    "ISP error"
    AlgError {
        "???"
        ABILITY_ARG = MV_ALG_E_ABILITY_ARG,

        "???"
        MEM_NULL = MV_ALG_E_MEM_NULL,

        "???"
        MEM_ALIGN = MV_ALG_E_MEM_ALIGN,

        "???"
        MEM_LACK = MV_ALG_E_MEM_LACK,

        "???"
        MEM_SIZE_ALIGN = MV_ALG_E_MEM_SIZE_ALIGN,

        "???"
        MEM_ADDR_ALIGN = MV_ALG_E_MEM_ADDR_ALIGN,

        "???"
        IMG_FORMAT = MV_ALG_E_IMG_FORMAT,

        "???"
        IMG_SIZE = MV_ALG_E_IMG_SIZE,

        "???"
        IMG_STEP = MV_ALG_E_IMG_STEP,

        "???"
        IMG_DATA_NULL = MV_ALG_E_IMG_DATA_NULL,

        "???"
        CFG_TYPE = MV_ALG_E_CFG_TYPE,

        "???"
        CFG_SIZE = MV_ALG_E_CFG_SIZE,

        "???"
        PRC_TYPE = MV_ALG_E_PRC_TYPE,

        "???"
        PRC_SIZE = MV_ALG_E_PRC_SIZE,

        "???"
        FUNC_TYPE = MV_ALG_E_FUNC_TYPE,

        "???"
        FUNC_SIZE = MV_ALG_E_FUNC_SIZE,

        "???"
        PARAM_INDEX = MV_ALG_E_PARAM_INDEX,

        "???"
        PARAM_VALUE = MV_ALG_E_PARAM_VALUE,

        "???"
        PARAM_NUM = MV_ALG_E_PARAM_NUM,

        "???"
        NULL_PTR = MV_ALG_E_NULL_PTR,

        "???"
        OVER_MAX_MEM = MV_ALG_E_OVER_MAX_MEM,

        "???"
        CALL_BACK = MV_ALG_E_CALL_BACK,

        "???"
        ENCRYPT = MV_ALG_E_ENCRYPT,

        "???"
        EXPIRE = MV_ALG_E_EXPIRE,

        "???"
        BAD_ARG = MV_ALG_E_BAD_ARG,

        "???"
        DATA_SIZE = MV_ALG_E_DATA_SIZE,

        "???"
        STEP = MV_ALG_E_STEP,

        "???"
        CPUID = MV_ALG_E_CPUID,

        "???"
        TIME_OUT = MV_ALG_E_TIME_OUT,

        "???"
        LIB_VERSION = MV_ALG_E_LIB_VERSION,

        "???"
        MODEL_VERSION = MV_ALG_E_MODEL_VERSION,

        "???"
        GPU_MEM_ALLOC = MV_ALG_E_GPU_MEM_ALLOC,

        "???"
        FILE_NON_EXIST = MV_ALG_E_FILE_NON_EXIST,

        "???"
        NONE_STRING = MV_ALG_E_NONE_STRING,

        "???"
        IMAGE_CODEC = MV_ALG_E_IMAGE_CODEC,

        "???"
        FILE_OPEN = MV_ALG_E_FILE_OPEN,

        "???"
        FILE_READ = MV_ALG_E_FILE_READ,

        "???"
        FILE_WRITE = MV_ALG_E_FILE_WRITE,

        "???"
        FILE_READ_SIZE = MV_ALG_E_FILE_READ_SIZE,

        "???"
        FILE_TYPE = MV_ALG_E_FILE_TYPE,

        "???"
        MODEL_TYPE = MV_ALG_E_MODEL_TYPE,

        "???"
        MALLOC_MEM = MV_ALG_E_MALLOC_MEM,

        "???"
        BIND_CORE_FAILED = MV_ALG_E_BIND_CORE_FAILED,

        "???"
        DENOISE_NE_IMG_FORMAT = MV_ALG_E_DENOISE_NE_IMG_FORMAT,

        "???"
        DENOISE_NE_FEATURE_TYPE = MV_ALG_E_DENOISE_NE_FEATURE_TYPE,

        "???"
        DENOISE_NE_PROFILE_NUM = MV_ALG_E_DENOISE_NE_PROFILE_NUM,

        "???"
        DENOISE_NE_GAIN_NUM = MV_ALG_E_DENOISE_NE_GAIN_NUM,

        "???"
        DENOISE_NE_GAIN_VAL = MV_ALG_E_DENOISE_NE_GAIN_VAL,

        "???"
        DENOISE_NE_BIN_NUM = MV_ALG_E_DENOISE_NE_BIN_NUM,

        "???"
        DENOISE_NE_INIT_GAIN = MV_ALG_E_DENOISE_NE_INIT_GAIN,

        "???"
        DENOISE_NE_NOT_INIT = MV_ALG_E_DENOISE_NE_NOT_INIT,

        "???"
        DENOISE_COLOR_MODE = MV_ALG_E_DENOISE_COLOR_MODE,

        "???"
        DENOISE_ROI_NUM = MV_ALG_E_DENOISE_ROI_NUM,

        "???"
        DENOISE_ROI_ORI_PT = MV_ALG_E_DENOISE_ROI_ORI_PT,

        "???"
        DENOISE_ROI_SIZE = MV_ALG_E_DENOISE_ROI_SIZE,

        "???"
        DENOISE_GAIN_NOT_EXIST = MV_ALG_E_DENOISE_GAIN_NOT_EXIST,

        "???"
        DENOISE_GAIN_BEYOND_RANGE = MV_ALG_E_DENOISE_GAIN_BEYOND_RANGE,

        "???"
        DENOISE_NP_BUF_SIZE = MV_ALG_E_DENOISE_NP_BUF_SIZE,

        // TODO ALG_ERR = MV_ALG_ERR, maybe just add something like is_alg_error
    }
}

define_sub_error! {
    "GigE status error"
    GigEError {
        "The command is not supported by device"
        NOT_IMPLEMENTED = MV_E_NOT_IMPLEMENTED,

        "The target address being accessed does not exist"
        INVALID_ADDRESS = MV_E_INVALID_ADDRESS,

        "The target address is not writable"
        WRITE_PROTECT = MV_E_WRITE_PROTECT,

        "No permission"
        ACCESS_DENIED = MV_E_ACCESS_DENIED,

        "Device is busy, or network disconnected"
        BUSY = MV_E_BUSY,

        "Network data packet error"
        PACKET = MV_E_PACKET,

        "Network error"
        NETER = MV_E_NETER,

        "SwitchKey error"
        KEY_VERIFICATION = MV_E_KEY_VERIFICATION,

        "Device IP conflict"
        IP_CONFLICT = MV_E_IP_CONFLICT,
    }
}

define_sub_error! {
    "Exception message"
    ExceptionError {
        "the device is disconnected"
        DEV_DISCONNECT = MV_EXCEPTION_DEV_DISCONNECT,

        "SDK does not match the driver version"
        VERSION_CHECK = MV_EXCEPTION_VERSION_CHECK,
    }
}

/*
    
    
    
    
    
    
    
    
    
    
    UPG_
    UPG_
    UPG_
    UPG_
    UPG_
    
    
    
    
    
    
    
    
    
    
    
    ALG_E_
    ALG_E_
    ALG_E_
    ALG_E_
    ALG_E_
    ALG_E_
    ALG_E_
    ALG_E_
    ALG_E_
    ALG_E_
    ALG_E_
    ALG_E_
    ALG_E_
    ALG_E_
    ALG_E_
    ALG_E_
    ALG_E_
    ALG_E_
    ALG_E_
    ALG_E_
    ALG_E_
    ALG_E_
    ALG_E_
    ALG_E_
    ALG_E_
    ALG_E_
    ALG_E_
    ALG_E_
    ALG_E_
    ALG_E_
    ALG_E_
    ALG_E_
    ALG_E_
    ALG_E_
    ALG_E_
    ALG_E_
    ALG_E_
    ALG_E_
    ALG_E_
    ALG_E_
    ALG_E_
    ALG_E_
    ALG_E_
    ALG_E_
    ALG_E_
    ALG_E_
    ALG_E_
    ALG_E_
    ALG_E_
    ALG_E_
    ALG_E_
    ALG_E_
    ALG_E_
    ALG_E_
    ALG_E_
    ALG_E_
    ALG_E_
    ALG_E_
    ,
    EXCEPTION_DEV_DISCONNECT = MV_EXCEPTION_DEV_DISCONNECT,
    EXCEPTION_VERSION_CHECK = MV_EXCEPTION_VERSION_CHECK,
*/
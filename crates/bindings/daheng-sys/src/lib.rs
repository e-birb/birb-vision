#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::{ffi::CStr, fmt::Debug};

pub enum SDK {
    V1(v1::API),
    V2(v2::API),
}

impl SDK {
    /// Try to load the SDK using [`default_lib_name`].
    pub unsafe fn auto_select() -> Option<Self> { // TODO exhaustive error handling
        let lib_name = default_lib_name().ok()?;

        match v2::API::new(lib_name) {
            Ok(v2) => {
                return Some(SDK::V2(v2));
            },
            Err(e) => {
                log::warn!("Failed to load Daheng SDK v2: {e}");
            },
        }

        match v1::API::new(lib_name) {
            Ok(v1) => {
                let version = CStr::from_ptr(v1.GXGetLibVersion());
                if version.to_str().unwrap().starts_with("1.") {
                    return Some(SDK::V1(v1));
                } else {
                    log::error!("The version of the Daheng SDK v1 is wrong ({version:?} but expected 1.x)");
                }
            },
            Err(e) => {
                log::warn!("Failed to load Daheng SDK v1: {e}");
            },
        }

        log::error!("Failed to load Daheng SDK, could not be interpreted neither as v1 nor v2");

        None
    }
}

impl Debug for SDK {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SDK::V1(_) => write!(f, "v1"),
            SDK::V2(_) => write!(f, "v2"),
        }
    }
}

pub mod v1 {
    include!(concat!(env!("OUT_DIR"), "/bindings_v1.rs"));
}

pub mod v2 {
    include!(concat!(env!("OUT_DIR"), "/bindings_v2.rs"));
}


pub fn default_lib_name() -> Result<&'static str, UnsupportedPlatformError> {
    // TODO verify
    log::warn!("The default library name is not verified for all platforms/versions yet");
    #[cfg(target_os = "linux")]
    {
        //return Ok("libGxIAPI.so");
        return Ok("libgxiapi.so");
    }
    #[cfg(target_os = "windows")]
    {
        return Ok("GxIAPI.dll");
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    return Err(UnsupportedPlatformError);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct UnsupportedPlatformError;

impl std::fmt::Display for UnsupportedPlatformError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Unsupported platform")
    }
}

impl std::error::Error for UnsupportedPlatformError {}
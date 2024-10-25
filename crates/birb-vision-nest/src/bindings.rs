#![allow(dead_code)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::{ffi::{c_char, CStr, OsStr}, fmt};

use anyhow::anyhow;

use crate::bindings;

pub fn load(path: impl AsRef<OsStr>) -> Result<Api, anyhow::Error> { // TODO return a more specific error (use thiserror)
    let lib = unsafe {
        libloading::Library::new(path)
    }?;

    log::trace!("Loaded library: {:?}", lib);

    type VersionFunction = unsafe extern "C" fn() -> *const c_char;

    let version_fn = unsafe {
        lib.get::<VersionFunction>(b"birb_vision_nest_interface_version")
    }.map_err(|e| anyhow::anyhow!("Failed to load version function: {}", e))?;

    log::trace!("Loaded version function: {:?}", version_fn);

    let version = unsafe {
        CStr::from_ptr(version_fn())
    }.to_str().map_err(|e| anyhow!("Invalid version string: {e}"))?;

    log::debug!("Loaded version declared by the library: {:?}", version);

    macro_rules! version {
        ($v:ident) => {
            CStr::from_bytes_with_nul(bindings::$v::BIRB_VISION_NEST_INTERFACE_VERSION).unwrap().to_str().unwrap()
        };
    }

    let result = if version == version!(v0) {
        Ok(Api::V0(unsafe { bindings::v0::Api::from_library(lib)? }))
    } else {
        log::error!("Unsupported version: {:?}", version);
        Err(anyhow!("version {version:?} is not supported by this program"))
    };

    if let Ok(result) = &result {
        log::trace!("Loaded API as: {result}");
    }

    result
}

pub enum Api {
    V0(v0::Api),
}

impl fmt::Display for Api {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::V0(_) => write!(f, "v0"),
        }
    }
}

macro_rules! call_common {
    ($api:expr => $fn:ident($($arg:expr),*)) => {
        match $api {
            Self::V0(api) => unsafe { api.$fn($($arg),*) },
        }
    };
}

extern "C" fn stupid_logger(level: u8, message: *const c_char) {
    let message = unsafe { CStr::from_ptr(message) };
    let message = format!("PLUGIN: {message:?}"); // TODO which plugin?
    match level {
        0 => log::trace!("{message}"),
        1 => log::debug!("{message}"),
        2 => log::info!("{message}"),
        3 => log::warn!("{message}"),
        4.. => log::error!("{message}"),
    }
}

impl Api {
    pub fn get_version(&self) -> anyhow::Result<&str> {
        let v = call_common!(self => birb_vision_nest_interface_version());
        Ok(unsafe { CStr::from_ptr(v) }.to_str()?)
    }

    pub fn supported_transport_layers(&self) -> anyhow::Result<Vec<String>> {
        let list = call_common!(self => supported_transport_layers(Some(stupid_logger)));
        scopeguard::defer! {
            call_common!(self => transport_layer_list_free(list));
        };

        let mut layers = Vec::new();
        for i in 0.. {
            let layer = call_common!(self => transport_layer_list_get(list, i));
            if layer.is_null() {
                break;
            }
            let layer = unsafe { CStr::from_ptr(layer) }.to_str()?.to_owned();
            layers.push(layer);
        }

        Ok(layers)
    }
}

pub mod v0 {
    include!(concat!(env!("OUT_DIR"), "/bindings/v0.rs"));
}
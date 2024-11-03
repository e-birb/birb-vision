use std::{sync::{Arc, Mutex, Weak}, time::Duration};

use anyhow::{anyhow, Error};
use daheng_sys::SDK;

use crate::{DahengError, DeviceInfo, GxError};

pub struct Ctx {
    inner: Arc<CtxInner>,
}

macro_rules! try_common {
    ($sdk:expr => $f:ident ($($args:tt)*)) => {
        $crate::GxError::result($sdk, match $sdk {
            daheng_sys::SDK::V1(api) => unsafe { api.$f($($args)*) },
            daheng_sys::SDK::V2(api) => unsafe { api.$f($($args)*) },
        })
    };
}
pub(crate) use try_common;

impl Ctx {
    pub fn new() -> Result<Self, DahengError> { // TODO a proper error handling
        let inner = CtxInner::get()?;

        Ok(Self {
            inner
        })
    }

    pub fn update_device_list(&self, timeout: Duration) -> Result<u32, DahengError> {
        let mut n = 0;
        try_common!(self.sdk() => GXUpdateDeviceList(&mut n, timeout.as_millis() as u32))?;
        Ok(n)
    }

    pub fn update_all_device_list(&self, timeout: Duration) -> Result<u32, DahengError> {
        let mut n = 0;
        try_common!(self.sdk() => GXUpdateAllDeviceList(&mut n, timeout.as_millis() as u32))?;
        Ok(n)
    }

    pub fn get_all_device_base_info(&self) -> Result<Vec<DeviceInfo>, DahengError> {
        let mut buffer_size = 0;
        try_common!(self.sdk() => GXGetAllDeviceBaseInfo(std::ptr::null_mut(), &mut buffer_size))?;

        if buffer_size == 0 {
            return Ok(Vec::new());
        }

        let buffer = vec![0u8; buffer_size];
        try_common!(self.sdk() => GXGetAllDeviceBaseInfo(buffer.as_ptr() as *mut _, &mut buffer_size))?;

        let devices = match self.sdk() {
            SDK::V1(_) => {
                use daheng_sys::v1::GX_DEVICE_BASE_INFO as INFO;
                if buffer.len() % size_of::<INFO>() != 0 {
                    return Err(anyhow!("GXGetAllDeviceBaseInfo returned a buffer with size not multiple of GX_DEVICE_BASE_INFO size").into());
                }
                let count = buffer.len() / size_of::<INFO>();
                let info_list = unsafe { std::slice::from_raw_parts(
                    buffer.as_ptr() as *const INFO,
                    count,
                ) };
                info_list.iter().map(|info| (DeviceInfo::V1(info.clone()))).collect()
            },
            SDK::V2(_) => {
                use daheng_sys::v2::GX_DEVICE_BASE_INFO as INFO;
                if buffer.len() % size_of::<INFO>() != 0 {
                    return Err(anyhow!("GXGetAllDeviceBaseInfo returned a buffer with size not multiple of GX_DEVICE_BASE_INFO size").into());
                }
                let count = buffer.len() / size_of::<INFO>();
                let info_list = unsafe { std::slice::from_raw_parts(
                    buffer.as_ptr() as *const INFO,
                    count,
                ) };
                info_list.iter().map(|info| (DeviceInfo::V2(info.clone()))).collect()
            },
        };
        Ok(devices)
    }

    pub(crate) fn sdk(&self) -> &SDK {
        &self.inner.sdk
    }
}

struct CtxInner {
    sdk: Arc<SDK>,
}

impl CtxInner {
    fn get() -> Result<Arc<CtxInner>, Error> {
        static INSTANCE: Mutex<Weak<CtxInner>> = Mutex::new(Weak::new());

        let global_instance = INSTANCE.lock().unwrap();

        if let Some(instance) = global_instance.upgrade() {
            return Ok(instance);
        }

        let sdk = unsafe { SDK::auto_select() }.ok_or(anyhow!("Could not load Daheng SDK"))?;

        GxError::result(&sdk, match &sdk {
            SDK::V1(api) => unsafe { api.GXInitLib() },
            SDK::V2(api) => unsafe { api.GXInitLib() },
        })?;

        Ok(Arc::new(CtxInner {
            sdk: Arc::new(sdk),
        }))
    }
}

impl Drop for CtxInner {
    fn drop(&mut self) {
        GxError::result(&self.sdk, match &*self.sdk {
            SDK::V1(api) => unsafe { api.GXCloseLib() },
            SDK::V2(api) => unsafe { api.GXCloseLib() },
        }).unwrap();

        // FIXME
        // HACK: for some reason when unloading the dll it freezes the program... so we just leak it
        #[cfg(windows)]
        std::mem::forget(self.sdk.clone());
    }
}
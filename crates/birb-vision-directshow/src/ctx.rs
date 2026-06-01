use std::sync::Arc;
use std::cell::RefCell;

use birb_vision_core::{
    anyhow::{self},
    context::{DeviceInfo, DeviceInfoEntry, VisionContext},
    CameraDevice,
};
use windows::{
    core::{GUID, HRESULT, HSTRING},
    Win32::{
        Media::DirectShow::{
            IBaseFilter, ICreateDevEnum,
        },
        System::Com::{
            CoCreateInstance, CLSCTX_INPROC_SERVER, CoInitializeEx, CoUninitialize,
            COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE,
            IEnumMoniker, IMoniker, CreateBindCtx,
        },
        System::Com::StructuredStorage::IPropertyBag,
    },
};

use crate::{device::{DSDeviceInfo, DirectShowDevice}, *};

#[allow(non_upper_case_globals)]
const CLSID_SystemDeviceEnum: GUID = GUID::from_u128(0x62BE5D10_60EB_11d0_BD3B_00A0C911CE86);

#[allow(non_upper_case_globals)]
const CLSID_VideoInputDeviceCategory: GUID = GUID::from_u128(0x860BB310_5D01_11d0_BD3B_00A0C911CE86);

pub struct DirectShowContext {
    _inner: Arc<CtxInner>,
}

impl DirectShowContext {
    pub fn new() -> DSResult<Self> {
        thread_local! {
            static CTX: RefCell<std::sync::Weak<CtxInner>> = RefCell::new(std::sync::Weak::new());
        }

        let ctx = CTX.with(|ctx| -> DSResult<Arc<CtxInner>> {
            let mut ctx = ctx.borrow_mut();
            if let Some(ctx) = ctx.upgrade() {
                return Ok(ctx);
            }
            let cx = Arc::new(CtxInner::new()?);
            *ctx = Arc::downgrade(&cx);
            Ok(cx)
        })?;

        Ok(Self { _inner: ctx })
    }

    /// Enumerate all DirectShow video capture devices.
    pub fn enumerate_devices(&self) -> DSResult<Vec<DSDeviceInfo>> {
        // Create the system device enumerator
        let dev_enum: ICreateDevEnum = unsafe {
            CoCreateInstance(&CLSID_SystemDeviceEnum, None, CLSCTX_INPROC_SERVER)?
        };

        // Get the enumerator for video input devices
        let mut enum_moniker: Option<IEnumMoniker> = None;
        unsafe {
            dev_enum.CreateClassEnumerator(
                &CLSID_VideoInputDeviceCategory,
                &mut enum_moniker,
                0,
            )?;
        }

        let Some(enum_moniker) = enum_moniker else {
            // No devices of this category
            return Ok(vec![]);
        };

        let mut devices = Vec::new();

        loop {
            // Get next moniker (request 1 at a time)
            let mut monikers: [Option<IMoniker>; 1] = [None];
            let mut fetched: u32 = 0;
            let hr = unsafe { enum_moniker.Next(&mut monikers, Some(&mut fetched)) };

            if hr.is_err() || fetched == 0 {
                break;
            }

            let Some(ref moniker) = monikers[0] else {
                break;
            };

            // Create bind context
            let bind_ctx = unsafe { CreateBindCtx(0) }?;

            // Get the property bag from the moniker
            let prop_bag: IPropertyBag = match unsafe {
                moniker.BindToStorage::<_, _, IPropertyBag>(&bind_ctx, None)
            } {
                Ok(bag) => bag,
                Err(e) => {
                    log::error!("Failed to bind to property bag: {e}");
                    continue;
                }
            };

            // Read FriendlyName
            let friendly_name = match read_property_bag_string(&prop_bag, "FriendlyName") {
                Ok(name) => name,
                Err(e) => {
                    log::error!("Failed to read FriendlyName: {e}");
                    continue;
                }
            };

            // Read DevicePath (optional)
            let device_path = read_property_bag_string(&prop_bag, "DevicePath").ok();

            let info = DSDeviceInfo {
                friendly_name: friendly_name.clone(),
                device_path,
            };

            log::info!("Found DirectShow device: {friendly_name}");
            devices.push(info);
        }

        Ok(devices)
    }
}

/// Helper to read a string property from an IPropertyBag.
fn read_property_bag_string(prop_bag: &IPropertyBag, name: &str) -> DSResult<String> {
    let mut variant = windows_core::VARIANT::new();
    let name_hstring = HSTRING::from(name);
    unsafe {
        prop_bag.Read(&name_hstring, &mut variant as *mut windows_core::VARIANT, None)?;
    }

    match windows_core::BSTR::try_from(&variant) {
        Ok(bstr) => Ok(bstr.to_string()),
        Err(_) => Err(DSError::msg(format!(
            "Property '{name}' is not a string (or conversion failed)",
        ))),
    }
}

impl VisionContext for DirectShowContext {
    fn enumerate(&self, _transport_layers: &[String]) -> anyhow::Result<Vec<DeviceInfo>> {
        let devices = self.enumerate_devices()?;
        let infos = devices.into_iter().map(|d| {
            let mut info = DeviceInfo::new();
            info.display_name = d.friendly_name.clone();
            if let Some(path) = d.device_path {
                info.other.insert(
                    "path".into(),
                    DeviceInfoEntry::new("Device Path", path),
                );
            }
            info
        }).collect();
        Ok(infos)
    }

    fn create(&self, info: &DeviceInfo) -> anyhow::Result<Option<Box<dyn CameraDevice>>> {
        // Find the device by display name
        let devices = self.enumerate_devices()?;
        let device_info = devices.into_iter().find(|d| {
            if d.friendly_name == info.display_name {
                return true;
            }
            // Also try matching by device path
            if let Some(path) = &d.device_path {
                if let Some(entry) = info.other.get("path") {
                    if entry.value == *path {
                        return true;
                    }
                }
            }
            false
        });

        match device_info {
            Some(di) => {
                let device = DirectShowDevice::new(
                    self._inner.clone(),
                    di,
                ).map_err(|e| anyhow::anyhow!("Failed to create DirectShow device: {e}"))?;
                Ok(Some(Box::new(device)))
            }
            None => Ok(None),
        }
    }
}

pub(crate) struct CtxInner {
    _com_guard: ComGuard,
}

impl CtxInner {
    fn new() -> DSResult<Self> {
        let com_guard = ComGuard::new()?;
        Ok(Self {
            _com_guard: com_guard,
        })
    }

    /// Find the moniker matching `info` and bind it to an [`IBaseFilter`].
    ///
    /// This re-enumerates the video-input-device category and compares
    /// `FriendlyName` / `DevicePath` against the given [`DSDeviceInfo`].
    pub fn bind_device_filter(&self, info: &DSDeviceInfo) -> DSResult<IBaseFilter> {
        let moniker = find_moniker(info)?;

        let bind_ctx = unsafe { CreateBindCtx(0) }?;

        let filter: IBaseFilter = unsafe {
            moniker.BindToObject(&bind_ctx, None)
        }
        .map_err(|e| {
            DSError::msg(format!(
                "Failed to bind moniker to IBaseFilter for device '{}': {e}",
                info.friendly_name
            ))
        })?;

        log::info!("Bound filter for device: {}", info.friendly_name);

        Ok(filter)
    }
}

/// Re-enumerate video-input devices and return the moniker that matches `info`.
fn find_moniker(info: &DSDeviceInfo) -> DSResult<IMoniker> {
    let dev_enum: ICreateDevEnum = unsafe {
        CoCreateInstance(&CLSID_SystemDeviceEnum, None, CLSCTX_INPROC_SERVER)?
    };

    let mut enum_moniker: Option<IEnumMoniker> = None;
    unsafe {
        dev_enum.CreateClassEnumerator(
            &CLSID_VideoInputDeviceCategory,
            &mut enum_moniker,
            0,
        )?;
    }

    let Some(enum_moniker) = enum_moniker else {
        return Err(DSError::msg("No video input device category found"));
    };

    loop {
        let mut monikers: [Option<IMoniker>; 1] = [None];
        let mut fetched: u32 = 0;
        let hr = unsafe { enum_moniker.Next(&mut monikers, Some(&mut fetched)) };

        if hr.is_err() || fetched == 0 {
            break;
        }

        let Some(ref moniker) = monikers[0] else {
            break;
        };

        let bind_ctx = unsafe { CreateBindCtx(0) }?;

        let prop_bag: IPropertyBag = match unsafe {
            moniker.BindToStorage::<_, _, IPropertyBag>(&bind_ctx, None)
        } {
            Ok(bag) => bag,
            Err(_) => continue,
        };

        let friendly_name = read_property_bag_string(&prop_bag, "FriendlyName").ok();
        let device_path = read_property_bag_string(&prop_bag, "DevicePath").ok();

        // Match by friendly name OR device path
        let matches = friendly_name.as_deref() == Some(&info.friendly_name)
            || (info.device_path.is_some()
                && device_path.as_deref() == info.device_path.as_deref());

        if matches {
            // Return the original moniker from the enumerator, not a clone
            return Ok(monikers[0].take().unwrap());
        }
    }

    Err(DSError::msg(format!(
        "Device '{}' not found during re-enumeration",
        info.friendly_name
    )))
}

struct ComGuard;

impl ComGuard {
    fn new() -> DSResult<Self> {
        init_com()?;
        Ok(Self)
    }
}

impl Drop for ComGuard {
    fn drop(&mut self) {
        uninit_com();
    }
}

fn init_com() -> DSResult<()> {
    let hresult = unsafe {
        CoInitializeEx(
            None,
            COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE,
        )
    };

    if hresult.is_ok() || hresult == HRESULT(0x00000001) /* S_FALSE - already initialized */ {
        Ok(())
    } else {
        Err(DSError::WinError(windows::core::Error::from_hresult(hresult)))
    }
}

fn uninit_com() {
    unsafe { CoUninitialize() }
}

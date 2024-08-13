use std::{cell::RefCell, mem::MaybeUninit, sync::{Arc, Weak}};

use windows::{core::PWSTR, Win32::{Media::MediaFoundation::{IMFActivate, IMFAttributes, MFCreateAttributes, MFEnumDeviceSources, MF_DEVSOURCE_ATTRIBUTE_FRIENDLY_NAME, MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE, MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_GUID, MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_SYMBOLIC_LINK}, System::Com::{self, COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE}}};

use crate::*;


pub struct MediaFoundationContext {
    _inner: Arc<CtxInner>,
}

impl MediaFoundationContext {
    pub fn new() -> MFResult<Self> {
        thread_local! {
            static CTX: RefCell<Weak<CtxInner>> = RefCell::new(Weak::new());
        }

        let ctx = CTX.with(|ctx| -> MFResult<Arc<CtxInner>> {
            let mut ctx = ctx.borrow_mut();
            if let Some(ctx) = ctx.upgrade() {
                return Ok(ctx);
            }
            let cx = Arc::new(CtxInner::new()?);
            *ctx = Arc::downgrade(&cx);
            Ok(cx)
        })?;

        Ok(Self {
            _inner: ctx,
        })
    }

    pub fn enumerate_devices(&self) -> MFResult<Vec<MFDeviceInfo>> {
        let list: Vec<MFDeviceInfo> = self.query_activate_pointers()?
            .into_iter()
            .map(|imf_activate| -> MFResult<MFDeviceInfo> {
                let mut pwstr_name = PWSTR(&mut 0_u16);
                let mut len_pwstrname = 0;
                let mut pwstr_symlink = PWSTR(&mut 0_u16);
                let mut len_pwstrsymlink = 0;
        
                unsafe {
                    imf_activate.GetAllocatedString(
                        &MF_DEVSOURCE_ATTRIBUTE_FRIENDLY_NAME,
                        &mut pwstr_name,
                        &mut len_pwstrname,
                    )?;
                    imf_activate.GetAllocatedString(
                        &MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_SYMBOLIC_LINK,
                        &mut pwstr_symlink,
                        &mut len_pwstrsymlink,
                    )?;
                }
        
                if pwstr_name.is_null() {
                    return Err(MFError::Other("pwstr_name is null".into()));
                }

                if pwstr_symlink.is_null() {
                    return Err(MFError::Other("pwstr_symlink is null".into()));
                }

                let name = unsafe {
                    pwstr_name
                        .to_string()
                        .map_err(|e| {
                            let r: Box<dyn Error> = Box::new(e);
                            r
                        })?
                };

                let symlink = unsafe {
                    pwstr_symlink
                        .to_string()
                        .map_err(|e| {
                            let r: Box<dyn Error> = Box::new(e);
                            r
                        })?
                };

                Ok(MFDeviceInfo {
                    name,
                    symlink,
                })
            })
            .filter_map(|r| {
                match r {
                    Ok(d) => Some(d),
                    Err(e) => {
                        log::error!("Error: {}", e);
                        None
                    }
                }
            })
            .collect();


        Ok(list)
    }

    pub(crate) fn query_activate_pointers(&self) -> MFResult<Vec<IMFActivate>> {
        let mut attributes: Option<IMFAttributes> = None;
        unsafe { MFCreateAttributes(&mut attributes, 1) }?;

        let Some(attributes) = attributes else {
            return Err(MFError::Other("MFCreateAttributes failed to create attributes in MediaFoundationContext::enumerate_devices".into()));
        };

        unsafe {
            attributes.SetGUID(
                &MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE,
                &MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_GUID,
            )
        }?;

        let mut count: u32 = 0;
        let mut unused_mf_activate: MaybeUninit<*mut Option<IMFActivate>> = MaybeUninit::uninit();

        unsafe {
            MFEnumDeviceSources(
                &attributes,
                unused_mf_activate.as_mut_ptr(),
                &mut count,
            )
        }?;

        let device_list = unsafe {
            Vec::from_raw_parts(
                unused_mf_activate.assume_init(),
                count as usize,
                count as usize,
            )
        };

        let device_list = device_list
            .into_iter()
            .filter_map(|e| e)
            .collect::<Vec<_>>();

        Ok(device_list)
    }
}

struct CtxInner {
    _com_guard: ComGuard,
}

impl CtxInner {
    fn new() -> MFResult<Self> {
        let com_guard = ComGuard::new()?;
        Ok(Self {
            _com_guard: com_guard,
        })
    }
}

struct ComGuard;

impl ComGuard {
    fn new() -> MFResult<Self> {
        init_com()?;
        Ok(Self)
    }
}

impl Drop for ComGuard {
    fn drop(&mut self) {
        uninit_com();
    }
}

fn init_com() -> MFResult<()> {
    let hresult = unsafe {
        Com::CoInitializeEx(
            None,
            // see https://learn.microsoft.com/en-us/windows/win32/api/objbase/ne-objbase-coinit#remarks
            COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE,
        )
    };

    MFError::from_hresult(hresult)
}

fn uninit_com() {
    unsafe {
        Com::CoUninitialize()
    }
}
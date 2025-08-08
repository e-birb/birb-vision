use std::{error::Error, sync::{Mutex, Arc}};

use serde::{Deserialize, Serialize};
use windows::Win32::{Media::MediaFoundation::{IMFAttributes, IMFMediaSource, IMFSourceReaderCallback, MFCreateAttributes, MFCreateSourceReaderFromMediaSource, MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_SYMBOLIC_LINK, MF_READWRITE_DISABLE_CONVERTERS, MF_SOURCE_READER_ASYNC_CALLBACK}};
use windows_core::PWSTR;

use crate::{device::ReaderCallback, MFDevice, MFError, MFResult, MediaFoundationContext};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MFDeviceInfo {
    pub(crate) name: String,
    pub(crate) symlink: String,
}

impl MFDeviceInfo {
    pub fn friendly_name(&self) -> &str {
        &self.name
    }

    pub fn symlink(&self) -> &str {
        &self.symlink
    }
}

impl MFDeviceInfo {
    pub fn create_device(&self) -> MFResult<MFDevice> {
        let cx = MediaFoundationContext::new()?;
        let activate_pointers = cx.query_activate_pointers()?;
        for imf_activate in activate_pointers {
            let mut pwstr_symlink = PWSTR(&mut 0_u16);
            let mut len_pwstrsymlink = 0;

            unsafe {
                imf_activate.GetAllocatedString(
                    &MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_SYMBOLIC_LINK,
                    &mut pwstr_symlink,
                    &mut len_pwstrsymlink,
                )?;
            }

            if pwstr_symlink.is_null() {
                return Err(MFError::Other("pwstr_symlink is null".into()));
            }

            let symlink = unsafe {
                pwstr_symlink
                    .to_string()
                    .map_err(|e| {
                        let r: Box<dyn Error> = Box::new(e);
                        r
                    })?
            };

            if self.symlink != symlink {
                continue;
            }

            let media_source = unsafe {
                imf_activate.ActivateObject::<IMFMediaSource>()?
            };

            // NOTE: since this is declared before the source_reader AND source_reader_attributes, it will be dropped AFTER
            // even if it we fail and return/panic somewhere in the middle
            // ! DO NOT MOVE THIS LINES BELOW THE source_reader_attributes OR source_reader DECLARATION!!!!!
            let callback = ReaderCallback::new();
            let callback_inner = callback.inner.clone();
            let callback: IMFSourceReaderCallback = callback.into();
            let callback: Box<IMFSourceReaderCallback> = Box::new(callback);

            let source_reader_attributes = {
                let mut attributes: Option<IMFAttributes> = None;
                unsafe { MFCreateAttributes(&mut attributes, 1) }?;

                let Some(attributes) = attributes else {
                    return Err(MFError::Other("MFCreateAttributes failed to create attributes in MediaFoundationContext::enumerate_devices".into()));
                };

                unsafe {
                    attributes.SetUINT32(&MF_READWRITE_DISABLE_CONVERTERS, u32::from(true))?;
                }

                attributes
            };

            unsafe {
                source_reader_attributes.SetUnknown(&MF_SOURCE_READER_ASYNC_CALLBACK, &*callback).unwrap(); // !!!!!!
            }

            let source_reader = unsafe {
                MFCreateSourceReaderFromMediaSource(&media_source, &source_reader_attributes)?
            };

            let source_reader = Arc::new(source_reader);
            callback_inner.lock().unwrap().source_reader = Arc::downgrade(&source_reader);

            let device = MFDevice {
                _cx: cx,
                info: self.clone(),
                is_streaming: Mutex::new(false),
                _callback: callback,
                callback_inner,
                source_reader,
            };

            // TODO maybe select a default format here?

            return Ok(device);
        }

        Err(MFError::Other("No device not present anymore".into()))
    }
}
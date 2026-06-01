//! Custom COM interface definitions for ISampleGrabber and ISampleGrabberCB.
//!
//! These interfaces are from `qedit.h` and are not (yet) in the `windows` crate.
//!
//! The `non_snake_case`, `non_upper_case_globals` and `dead_code` warnings are expected:
//! COM vtable struct fields and GUID constants use Windows naming conventions,
//! and these types won't be used until filter-graph rendering is implemented.

#![allow(
    non_snake_case,
    non_upper_case_globals,
    dead_code,
    reason = "COM interop types – naming matches the Windows SDK headers"
)]

use windows::Win32::Foundation::BOOL;
use windows_core::*;

// SAFETY: COM interface pointers are reference-counted and safe to send/access
// across threads under the COM apartment model when the underlying COM object supports it.
// DirectShow filters are free-threaded.
unsafe impl Send for ISampleGrabber {}
unsafe impl Sync for ISampleGrabber {}
unsafe impl Send for ISampleGrabberCB {}
unsafe impl Sync for ISampleGrabberCB {}

// IID_ISampleGrabber: {6B652FFF-11FE-4fce-92AD-0266B5D7C78F}
const IID_ISampleGrabber: GUID = GUID::from_u128(0x6B652FFF_11FE_4fce_92AD_0266B5D7C78F);

// IID_ISampleGrabberCB: {0579154A-2B53-4994-B0D0-E773148EFF85}
const IID_ISampleGrabberCB: GUID = GUID::from_u128(0x0579154A_2B53_4994_B0D0_E773148EFF85);

// CLSID_SampleGrabber: {C1F400A0-3F08-11D3-9F0B-006008039E37}
pub const CLSID_SampleGrabber: GUID = GUID::from_u128(0xC1F400A0_3F08_11D3_9F0B_006008039E37);

// CLSID_NullRenderer: {C1F400A4-3F08-11D3-9F0B-006008039E37}
pub const CLSID_NullRenderer: GUID = GUID::from_u128(0xC1F400A4_3F08_11D3_9F0B_006008039E37);

// CLSID_FilterGraph: {E436EBB3-524F-11CE-9F53-0020AF0BA770}
pub const CLSID_FilterGraph: GUID = GUID::from_u128(0xE436EBB3_524F_11CE_9F53_0020AF0BA770);

// MEDIATYPE_Video: {73646976-0000-0010-8000-00AA00389B71}
pub const MEDIATYPE_Video: GUID = GUID::from_u128(0x73646976_0000_0010_8000_00AA00389B71);

// MEDIASUBTYPE_RGB24: {E436EB7D-524F-11CE-9F53-0020AF0BA770}
pub const MEDIASUBTYPE_RGB24: GUID = GUID::from_u128(0xE436EB7D_524F_11CE_9F53_0020AF0BA770);

// PIN_CATEGORY_CAPTURE: {FB6C4281-0353-11D1-905F-0000C0CC16BA}
pub const PIN_CATEGORY_CAPTURE: GUID = GUID::from_u128(0xFB6C4281_0353_11D1_905F_0000C0CC16BA);

/// CLSID_CaptureGraphBuilder2: {BF87BFA1-8DE2-11d0-A580-00A0C922E48A}
pub const CLSID_CaptureGraphBuilder2: GUID = GUID::from_u128(0xBF87BFA1_8DE2_11d0_A580_00A0C922E48A);

/// PIN_CATEGORY_PREVIEW: {FB6C4282-0353-11D1-905F-0000C0CC16BA}
#[allow(unused)]
pub const PIN_CATEGORY_PREVIEW: GUID = GUID::from_u128(0xFB6C4282_0353_11D1_905F_0000C0CC16BA);

/// MEDIASUBTYPE_RGB32: {E436EB7E-524F-11CE-9F53-0020AF0BA770}
#[allow(unused)]
pub const MEDIASUBTYPE_RGB32: GUID = GUID::from_u128(0xE436EB7E_524F_11CE_9F53_0020AF0BA770);

/// FORMAT_VideoInfo: {05589F80-C356-11CE-BF01-00AA0055595A}
pub const FORMAT_VideoInfo: GUID = GUID::from_u128(0x05589f80_c356_11ce_bf01_00aa0055595a);

/// FORMAT_VideoInfo2: {F72A76A0-EB0A-11D0-ACE4-0000C0CC16BA}
#[allow(unused)]
pub const FORMAT_VideoInfo2: GUID = GUID::from_u128(0xf72a76a0_eb0a_11d0_ace4_0000c0cc16ba);

windows_core::imp::define_interface!(ISampleGrabber, ISampleGrabber_Vtbl, 0x6B652FFF_11FE_4fce_92AD_0266B5D7C78F);

impl ISampleGrabber {
    pub unsafe fn SetOneShot(&self, one_shot: bool) -> Result<()> {
        (Interface::vtable(self).SetOneShot)(Interface::as_raw(self), BOOL(one_shot.into())).ok()
    }

    pub unsafe fn SetMediaType(&self, media_type: *const AM_MEDIA_TYPE) -> Result<()> {
        (Interface::vtable(self).SetMediaType)(Interface::as_raw(self), core::mem::transmute(media_type)).ok()
    }

    pub unsafe fn GetConnectedMediaType(&self, media_type: *mut AM_MEDIA_TYPE) -> Result<()> {
        (Interface::vtable(self).GetConnectedMediaType)(Interface::as_raw(self), core::mem::transmute(media_type)).ok()
    }

    pub unsafe fn SetBufferSamples(&self, buffer_them: bool) -> Result<()> {
        (Interface::vtable(self).SetBufferSamples)(Interface::as_raw(self), BOOL(buffer_them.into())).ok()
    }

    pub unsafe fn GetCurrentBuffer(&self, buffer_size: *mut i32, buffer: *mut i32) -> Result<()> {
        (Interface::vtable(self).GetCurrentBuffer)(Interface::as_raw(self), core::mem::transmute(buffer_size), core::mem::transmute(buffer)).ok()
    }

    pub unsafe fn SetCallback(&self, callback: Option<&ISampleGrabberCB>, which_method: i32) -> Result<()> {
        (Interface::vtable(self).SetCallback)(Interface::as_raw(self), core::mem::transmute(callback), which_method).ok()
    }
}

#[repr(C)]
pub struct ISampleGrabber_Vtbl {
    pub base__: IUnknown_Vtbl,
    pub SetOneShot: unsafe extern "system" fn(*mut core::ffi::c_void, BOOL) -> HRESULT,
    pub SetMediaType: unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> HRESULT,
    pub GetConnectedMediaType: unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> HRESULT,
    pub SetBufferSamples: unsafe extern "system" fn(*mut core::ffi::c_void, BOOL) -> HRESULT,
    pub GetCurrentBuffer: unsafe extern "system" fn(*mut core::ffi::c_void, *mut i32, *mut i32) -> HRESULT,
    pub GetCurrentSample: unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void) -> HRESULT,
    pub SetCallback: unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, i32) -> HRESULT,
}

windows_core::imp::define_interface!(ISampleGrabberCB, ISampleGrabberCB_Vtbl, 0x0579154A_2B53_4994_B0D0_E773148EFF85);

#[repr(C)]
pub struct ISampleGrabberCB_Vtbl {
    pub base__: IUnknown_Vtbl,
    pub SampleCB: unsafe extern "system" fn(*mut core::ffi::c_void, f64, *mut core::ffi::c_void) -> HRESULT,
    pub BufferCB: unsafe extern "system" fn(*mut core::ffi::c_void, f64, *mut u8, i32) -> HRESULT,
}

/// AM_MEDIA_TYPE structure (from strmif.h)
#[repr(C)]
pub struct AM_MEDIA_TYPE {
    pub majortype: GUID,
    pub subtype: GUID,
    pub bFixedSizeSamples: BOOL,
    pub bTemporalCompression: BOOL,
    pub lSampleSize: u32,
    pub formattype: GUID,
    pub pUnk: *mut IUnknown,
    pub cbFormat: u32,
    pub pbFormat: *mut u8,
}

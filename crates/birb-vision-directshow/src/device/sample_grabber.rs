//! Custom COM interface definitions for ISampleGrabber and ISampleGrabberCB.
//!
//! These interfaces are from `qedit.h` and are not (yet) in the `windows` crate.
//!
//! The `non_snake_case`, `non_upper_case_globals` and `dead_code` warnings are expected:
//! COM vtable struct fields and GUID constants use Windows naming conventions,
//! and some types/consts may be unused depending on which DirectShow graph features are enabled.

#![allow(
    non_snake_case,
    non_upper_case_globals,
    dead_code,
    reason = "COM interop types – naming matches the Windows SDK headers"
)]

use windows::Win32::Foundation::BOOL;
use windows_core::*;


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

// ─── ISampleGrabberCB implementation ─────────────────────────────────────
//
// Instead of polling ISampleGrabber::GetCurrentBuffer from a background
// thread (which requires cross-apartment COM marshaling that ISampleGrabber
// does not support), we implement ISampleGrabberCB here and register it
// via SetCallback.  BufferCB is called directly on the DirectShow streaming
// thread, so no marshaling is needed.

use std::borrow::Cow;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use birb_vision_core::{
    FlatSample, FlatSampleLayout, ImageSampleBuffer, PixelFormat, Sample, SampleType, StreamEvent,
};

/// A COM object implementing [`ISampleGrabberCB`] (via `BufferCB`).
///
/// # Safety / COM rules
///
/// The vtable pointer is the first field (`#[repr(C)]`), which is the
/// standard COM object layout.  The sample grabber calls `AddRef` when
/// the callback is set, and `Release` when the callback is cleared or the
/// filter is destroyed.  The `BufferCB` method is invoked on the DirectShow
/// streaming thread — we must NOT block for long.
#[repr(C)]
pub(crate) struct SampleGrabberCB {
    vtable: &'static ISampleGrabberCB_Vtbl,
    ref_count: AtomicU32,
    inner: SampleGrabberCBInner,
}

struct SampleGrabberCBInner {
    callback: Arc<Mutex<Box<dyn Fn(StreamEvent) + Send + Sync>>>,
    width: u32,
    height: u32,
}

impl SampleGrabberCB {
    /// Allocate a new `SampleGrabberCB` and return a raw COM pointer to it.
    ///
    /// The returned pointer has a reference count of 1.  The caller (typically
    /// the sample grabber's `SetCallback`) should take ownership through the
    /// normal COM `AddRef` / `Release` discipline.
    pub(crate) fn new_raw(
        callback: Arc<Mutex<Box<dyn Fn(StreamEvent) + Send + Sync>>>,
        width: u32,
        height: u32,
    ) -> *mut Self {
        let obj = Box::into_raw(Box::new(Self {
            vtable: &VTBL,
            ref_count: AtomicU32::new(1),
            inner: SampleGrabberCBInner {
                callback,
                width,
                height,
            },
        }));
        obj
    }
}

// SAFETY: The vtable functions are safe to call from any thread as long as
// the underlying data (callback Arc/Mutex) is thread-safe, which it is.
unsafe impl Send for SampleGrabberCB {}
unsafe impl Sync for SampleGrabberCB {}

// ── VTable ────────────────────────────────────────────────────────────────

static VTBL: ISampleGrabberCB_Vtbl = ISampleGrabberCB_Vtbl {
    base__: IUnknown_Vtbl {
        QueryInterface: cb_query_interface,
        AddRef: cb_add_ref,
        Release: cb_release,
    },
    SampleCB: cb_sample_cb,
    BufferCB: cb_buffer_cb,
};

// IID_IUnknown: {00000000-0000-0000-C000-000000000046}
const IID_IUnknown: GUID = GUID::from_u128(0x00000000_0000_0000_C000_000000000046);

// ── IUnknown methods ──────────────────────────────────────────────────────

unsafe extern "system" fn cb_query_interface(
    this: *mut std::ffi::c_void,
    iid: *const GUID,
    interface: *mut *mut std::ffi::c_void,
) -> HRESULT {
    if iid.is_null() || interface.is_null() {
        return HRESULT(0x80070057u32 as i32); // E_INVALIDARG
    }

    let guid = unsafe { &*iid };

    if *guid == IID_IUnknown || *guid == IID_ISampleGrabberCB {
        // AddRef before returning the interface
        unsafe {
            cb_add_ref(this);
        }
        unsafe {
            *interface = this;
        }
        HRESULT(0) // S_OK
    } else {
        unsafe {
            *interface = std::ptr::null_mut();
        }
        HRESULT(0x80004002u32 as i32) // E_NOINTERFACE
    }
}

unsafe extern "system" fn cb_add_ref(this: *mut std::ffi::c_void) -> u32 {
    let obj = unsafe { &*(this as *const SampleGrabberCB) };
    obj.ref_count.fetch_add(1, Ordering::SeqCst) + 1
}

unsafe extern "system" fn cb_release(this: *mut std::ffi::c_void) -> u32 {
    let obj = unsafe { &*(this as *const SampleGrabberCB) };
    let remaining = obj.ref_count.fetch_sub(1, Ordering::SeqCst) - 1;
    if remaining == 0 {
        // Reconstruct the box and drop it
        unsafe {
            let _ = Box::from_raw(this as *mut SampleGrabberCB);
        }
    }
    remaining
}

// ── ISampleGrabberCB methods ──────────────────────────────────────────────

/// `SampleCB( SampleTime, IMediaSample *pSample )`
///
/// We only implement `BufferCB`; this just returns S_OK.
unsafe extern "system" fn cb_sample_cb(
    _this: *mut std::ffi::c_void,
    _sample_time: f64,
    _p_sample: *mut std::ffi::c_void,
) -> HRESULT {
    HRESULT(0) // S_OK — not used
}

/// `BufferCB( double SampleTime, BYTE *pBuffer, long BufferLen )`
///
/// Called by the DirectShow streaming thread for every frame.  We copy the
/// buffer, build a `FlatSample`, and forward it to the Rust callback.
unsafe extern "system" fn cb_buffer_cb(
    this: *mut std::ffi::c_void,
    _sample_time: f64,
    p_buffer: *mut u8,
    buffer_len: i32,
) -> HRESULT {
    if this.is_null() {
        return HRESULT(0x80070057u32 as i32); // E_INVALIDARG
    }

    let obj = unsafe { &*(this as *const SampleGrabberCB) };

    if buffer_len <= 0 || p_buffer.is_null() {
        return HRESULT(0); // S_OK — skip empty frame
    }

    // Copy the buffer (the pointer is only valid during this callback)
    let data = unsafe {
        std::slice::from_raw_parts(p_buffer, buffer_len as usize)
    };
    let owned = data.to_vec();

    let sample = FlatSample {
        buffer: ImageSampleBuffer::Cow(Cow::Owned(owned)),
        layout: FlatSampleLayout {
            offset: 0,
            sample_type: SampleType::Plain(PixelFormat::BGR8Packed),
            width: obj.inner.width,
            height: obj.inner.height,
            row_major: true,
            stride: (obj.inner.width as i32 * 3),
        },
    };

    // Forward to the Rust callback.
    if let Ok(cb) = obj.inner.callback.try_lock() {
        cb(StreamEvent::Sample(Ok(Sample::ImageSample(sample))));
    }

    HRESULT(0) // S_OK
}

use std::{ffi::c_void, sync::Mutex};

use anyhow::anyhow;
use birb_vision_core::{utils::try_no_panic, DeviceResult, FlatSample, FlatSampleLayout, ImageSampleBuffer, PixelFormat, Sample, SampleType};
use daheng_sys::{v1, v2, SDK};

use crate::{DahengError, GxError};

use super::Device;


pub(crate) struct DeviceCallbacks {
    pub stream_callback: Option<Box<dyn for<'a> Fn(birb_vision_core::Event<'a>) + Send + Sync>>,
}

type DeviceCallbacksPtr = *const Mutex<DeviceCallbacks>;

impl DeviceCallbacks {
    pub fn new() -> Box<Mutex<Self>> {
        Box::new(Mutex::new(Self {
            stream_callback: None,
        }))
    }

    pub fn setup(device: &Device) -> Result<(), DahengError> {
        let callbacks_ptr: DeviceCallbacksPtr = &*device.callbacks;
        GxError::result(device.cx.sdk(), match device.cx.sdk() {
            SDK::V1(v1) => unsafe {
                v1.GXRegisterCaptureCallback(device.handle, callbacks_ptr as *mut c_void, Some(capture_callback_v1))
            },
            SDK::V2(v2) => unsafe {
                v2.GXRegisterCaptureCallback(device.handle, callbacks_ptr as *mut c_void, Some(capture_callback_v2))
            },
        })?;

        Ok(())
    }
}

unsafe extern "C" fn capture_callback_v1(
    #[allow(non_snake_case)]
    pFrameData: *mut v1::GX_FRAME_CALLBACK_PARAM,
) {
    try_no_panic(move || {
        assert!(!pFrameData.is_null());
        let frame_data = &*pFrameData;
        assert!(!frame_data.pUserParam.is_null());
        let callbacks_ptr = &*(frame_data.pUserParam as DeviceCallbacksPtr);

        let sample_type = convert_sample_type_v1(frame_data.nPixelFormat as u32).unwrap();
        let layout = FlatSampleLayout {
            offset: 0,
            row_major: true,
            sample_type,
            width: frame_data.nWidth as u32,
            height: frame_data.nHeight as u32,
            stride: 0,
        };

        let sample: DeviceResult<Sample> = match frame_data.status {
            v1::GX_FRAME_STATUS_LIST_GX_FRAME_STATUS_SUCCESS => {
                let buffer = frame_data.pImgBuf as *const u8;
                let buffer = std::slice::from_raw_parts(buffer, frame_data.nImgSize as usize); // TODO check if size > 0, otherwise return empty slice | or use an unchecked version
                Ok(Sample::ImageSample(FlatSample {
                    buffer: ImageSampleBuffer::Cow(buffer.into()),
                    layout,
                }))
            },
            v1::GX_FRAME_STATUS_LIST_GX_FRAME_STATUS_INCOMPLETE => Err(anyhow!("Incomplete frame").into()),
            v1::GX_FRAME_STATUS_LIST_GX_FRAME_STATUS_INVALID_IMAGE_INFO => Err(anyhow!("Invalid image info").into()),
            status => Err(anyhow!("Unknown frame status: {status:#x}").into()),
        };

        let callbacks = callbacks_ptr.lock().unwrap();
        if let Some(f) = callbacks.stream_callback.as_ref() {
            f(birb_vision_core::Event::Sample(sample));
        }
    });
}

unsafe extern "C" fn capture_callback_v2(
    #[allow(non_snake_case)]
    pFrameData: *mut v2::GX_FRAME_CALLBACK_PARAM,
) {
    try_no_panic(move || {
        assert!(!pFrameData.is_null());
        let frame_data = &*pFrameData;
        assert!(!frame_data.pUserParam.is_null());
        let callbacks_ptr = &*(frame_data.pUserParam as DeviceCallbacksPtr);

        let sample_type = convert_sample_type_v2(frame_data.nPixelFormat as u32).unwrap();
        let layout = FlatSampleLayout {
            offset: 0,
            row_major: true,
            sample_type,
            width: frame_data.nWidth as u32,
            height: frame_data.nHeight as u32,
            stride: 0,
        };

        let sample: DeviceResult<Sample> = match frame_data.status {
            v2::GX_FRAME_STATUS_LIST_GX_FRAME_STATUS_SUCCESS => {
                let buffer = frame_data.pImgBuf as *const u8;
                let buffer = std::slice::from_raw_parts(buffer, frame_data.nImgSize as usize); // TODO check if size > 0, otherwise return empty slice | or use an unchecked version
                Ok(Sample::ImageSample(FlatSample {
                    buffer: ImageSampleBuffer::Cow(buffer.into()),
                    layout,
                }))
            },
            v2::GX_FRAME_STATUS_LIST_GX_FRAME_STATUS_INCOMPLETE => Err(anyhow!("Incomplete frame").into()),
            status => Err(anyhow!("Unknown frame status: {status:#x}").into()),
        };

        let callbacks = callbacks_ptr.lock().unwrap();
        if let Some(f) = callbacks.stream_callback.as_ref() {
            f(birb_vision_core::Event::Sample(sample));
        }
    });
}

fn convert_sample_type_v1(pf: u32) -> Result<SampleType, DahengError> {
    use v1::*;
    use SampleType::*;
    let sample_type = match pf {
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_UNDEFINED
        GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_MONO8 => Plain(PixelFormat::Mono8Packed),
        GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_MONO8_SIGNED => Plain(PixelFormat::Mono8Signed),
        GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_MONO10 => Plain(PixelFormat::Mono10),
        GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_MONO12 => Plain(PixelFormat::Mono12),
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_MONO14
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_MONO16
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BAYER_GR8
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BAYER_RG8
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BAYER_GB8
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BAYER_BG8
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BAYER_GR10
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BAYER_RG10
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BAYER_GB10
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BAYER_BG10
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BAYER_GR12
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BAYER_RG12
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BAYER_GB12
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BAYER_BG12
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BAYER_GR16
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BAYER_RG16
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BAYER_GB16
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BAYER_BG16
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_RGB8_PLANAR
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_RGB10_PLANAR
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_RGB12_PLANAR
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_RGB16_PLANAR
        GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_RGB8 => Plain(PixelFormat::RGB8Packed),
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_RGB10
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_RGB12
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_RGB14
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_RGB16
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BGR8
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BGR10
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BGR12
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BGR14
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BGR16
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_RGBA8
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BGRA8
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_ARGB8
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_ABGR8
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_YUV444_8
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_YUV422_8
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_YUV411_8
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_YUV420_8_PLANAR
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_YCBCR444_8
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_YCBCR422_8
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_YCBCR411_8
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_YCBCR601_444_8
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_YCBCR601_422_8
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_YCBCR601_411_8
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_YCBCR709_444_8
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_YCBCR709_422_8
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_YCBCR709_411_8
        GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_MONO10_PACKED => Plain(PixelFormat::Mono10Packed),
        GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_MONO12_PACKED => Plain(PixelFormat::Mono12Packed),
        pf => return Err(anyhow!("Unknown pixel format: {pf}").into()),
    };

    Ok(sample_type)
}

fn convert_sample_type_v2(pf: u32) -> Result<SampleType, DahengError> {
    use v2::*;
    use SampleType::*;
    let pf = match pf {
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_UNDEFINED
        GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_MONO8 => Plain(PixelFormat::Mono8Packed),
        GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_MONO8_SIGNED => Plain(PixelFormat::Mono8Signed),
        GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_MONO10 => Plain(PixelFormat::Mono10),
        GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_MONO10_P => Plain(PixelFormat::Mono10Packed),
        GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_MONO12 => Plain(PixelFormat::Mono12),
        GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_MONO12_P => Plain(PixelFormat::Mono12Packed),
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_MONO14
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_MONO14_P
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_MONO16
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BAYER_GR8
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BAYER_RG8
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BAYER_GB8
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BAYER_BG8
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BAYER_GR10
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BAYER_GR10_P
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BAYER_RG10
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BAYER_RG10_P
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BAYER_GB10
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BAYER_GB10_P
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BAYER_BG10
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BAYER_BG10_P
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BAYER_GR12
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BAYER_GR12_P
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BAYER_RG12
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BAYER_RG12_P
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BAYER_GB12
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BAYER_GB12_P
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BAYER_BG12
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BAYER_BG12_P
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BAYER_GR14
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BAYER_GR14_P
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BAYER_RG14
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BAYER_RG14_P
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BAYER_GB14
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BAYER_GB14_P
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BAYER_BG14
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BAYER_BG14_P
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BAYER_GR16
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BAYER_RG16
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BAYER_GB16
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BAYER_BG16
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_RGB8_PLANAR
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_RGB10_PLANAR
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_RGB12_PLANAR
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_RGB16_PLANAR
        GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_RGB8 => Plain(PixelFormat::RGB8Packed),
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_RGB10
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_RGB12
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_RGB14
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_RGB16
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BGR8
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BGR10
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BGR12
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BGR14
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BGR16
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_RGBA8
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BGRA8
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_ARGB8
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_ABGR8
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_R8
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_G8
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_B8
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_COORD3D_ABC32F
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_COORD3D_ABC32F_PLANAR
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_COORD3D_C16
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_COORD3D_C16_I16
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_COORD3D_C16_S16
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_COORD3D_C16_I16_S16
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_YUV444_8
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_YUV422_8
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_YUV411_8
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_YUV420_8_PLANAR
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_YCBCR444_8
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_YCBCR422_8
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_YCBCR411_8
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_YCBCR601_444_8
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_YCBCR601_422_8
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_YCBCR601_411_8
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_YCBCR709_444_8
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_YCBCR709_422_8
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_YCBCR709_411_8
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_MONO10_PACKED
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_MONO12_PACKED
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BAYER_BG10_PACKED
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BAYER_BG12_PACKED
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BAYER_GB10_PACKED
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BAYER_GB12_PACKED
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BAYER_GR10_PACKED
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BAYER_GR12_PACKED
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BAYER_RG10_PACKED
        //GX_PIXEL_FORMAT_ENTRY_GX_PIXEL_FORMAT_BAYER_RG12_PACKED
        pf => return Err(anyhow!("Unknown pixel format: {pf}").into()),
    };

    Ok(pf)
}
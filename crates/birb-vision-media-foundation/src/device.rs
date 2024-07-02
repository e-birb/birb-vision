use std::{borrow::Cow, error::Error};

use birb_vision::decoders::{buf_yuyv422_to_rgb, decode_mjpg, nv12_to_rgb_image};
use image::{DynamicImage, Rgb, RgbImage};
use windows::{core::{GUID, PWSTR}, Win32::Media::{KernelStreaming::KSCAMERA_EXTENDEDPROP_FOCUSSTATE, MediaFoundation::{IMFAttributes, IMFMediaSource, IMFMediaType, IMFSourceReader, IMFTransform, MFCreateAttributes, MFCreateSample, MFCreateSampleCopierMFT, MFCreateSourceReaderFromMediaSource, MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_SYMBOLIC_LINK, MF_MT_FRAME_RATE, MF_MT_FRAME_RATE_RANGE_MAX, MF_MT_FRAME_RATE_RANGE_MIN, MF_MT_FRAME_SIZE, MF_MT_SUBTYPE, MF_READWRITE_DISABLE_CONVERTERS}}};

use crate::*;



#[derive(Debug, Clone)]
pub struct MFDeviceInfo {
    pub(crate) name: String,
    pub(crate) symlink: String,
}

impl MFDeviceInfo {
    pub fn name(&self) -> &str {
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

            let source_reader = unsafe {
                MFCreateSourceReaderFromMediaSource(&media_source, &source_reader_attributes)?
            };

            let media_type = unsafe {
                source_reader.GetCurrentMediaType(MEDIA_FOUNDATION_FIRST_VIDEO_STREAM)?
            };

            return Ok(MFDevice {
                cx,
                info: self.clone(),
                is_open: false,
                source_reader,
            });
        }

        Err(MFError::Other("No device not present anymore".into()))
    }
}

pub struct MFDevice {
    cx: MediaFoundationContext,
    info: MFDeviceInfo,
    is_open: bool,
    source_reader: IMFSourceReader,
}

impl MFDevice {
    pub fn info(&self) -> &MFDeviceInfo {
        &self.info
    }

    pub fn compatible_format_list(&self) -> MFResult<Vec<VideoFormat>> {
        let mut list = vec![];

        let mut index = 0;
        while let Ok(media_type) = unsafe {
            self.source_reader.GetNativeMediaType(MEDIA_FOUNDATION_FIRST_VIDEO_STREAM, index)
        } {
            index += 1;

            match VideoFormat::list(&media_type) {
                Ok(framerates) => {
                    list.extend(framerates);
                },
                Err(e) => log::warn!("Failed to list video formats for media type {media_type:?}, error: {e}"),
            };
        }

        Ok(list)
    }

    pub fn get_current_format(&self) -> MFResult<VideoFormat> {
        let media_type = unsafe {
            self.source_reader.GetCurrentMediaType(MEDIA_FOUNDATION_FIRST_VIDEO_STREAM)?
        };

        VideoFormat::from_media_type(&media_type)
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }

    pub fn select_format(&mut self, query: impl Into<VideoFormatQuery>) -> MFResult<VideoFormat> {
        // This function if much different from the original nokhwa one: https://github.com/l1npengtul/nokhwa/blob/58454663b811f45388cc5a0cd681cb397ef51922/nokhwa-bindings-windows/src/lib.rs#L1009
        // but I think this is more correct

        let query: VideoFormatQuery = query.into();
        log::debug!("query: {:?}", query);

        let mut index = 0;
        while let Ok(media_type) = unsafe {
            self.source_reader.GetNativeMediaType(MEDIA_FOUNDATION_FIRST_VIDEO_STREAM, index)
        } {
            index += 1;

            let Ok(format) = VideoFormat::from_media_type(&media_type) else {
                log::warn!("Failed to parse media type {media_type:?}");
                continue;
            };

            // TODO use query.matches(&format) instead for better methods organization
            if format.satisfies(&query) {
                unsafe {
                    self.source_reader.SetCurrentMediaType(MEDIA_FOUNDATION_FIRST_VIDEO_STREAM, None, &media_type)?;
                }

                log::debug!("Selected format: {format:?}");

                return Ok(format);
            }
        }

        Err(MFError::Other("No matching format found".into()))
    }

    pub fn start_stream(&mut self) -> MFResult<()> {
        unsafe {
            self.source_reader.SetStreamSelection(MEDIA_FOUNDATION_FIRST_VIDEO_STREAM, true)?
        };

        self.is_open = true;
        Ok(())
    }

    //pub fn raw_bytes(&mut self) -> MFResult<Cow<[u8]>> {
    pub fn receive_raw_bytes<R>(&mut self, f: impl FnOnce(&[u8]) -> R) -> MFResult<R> {
        let mut imf_sample = unsafe {
            Some(MFCreateSample()?)
        };

        let mut stream_flags = 0;
        loop {
            unsafe {
                self.source_reader.ReadSample(
                    MEDIA_FOUNDATION_FIRST_VIDEO_STREAM,
                    0,
                    None,
                    Some(&mut stream_flags),
                    None,
                    Some(&mut imf_sample),
                )?;
            }

            if imf_sample.is_some() {
                break;
            }
        }

        let imf_sample = imf_sample.unwrap();

        let buffer = unsafe {
            imf_sample.ConvertToContiguousBuffer()?
        };

        let mut buffer_valid_length = 0;
        let mut buffer_start_ptr = std::ptr::null_mut::<u8>();

        unsafe {
            buffer.Lock(&mut buffer_start_ptr, None, Some(&mut buffer_valid_length))?
        };

        scopeguard::defer! {
            unsafe {
                if let Err(e) = buffer.Unlock() {
                    log::error!("Failed to unlock buffer: {e}");
                }
            }
        }

        let slice = unsafe {
            std::slice::from_raw_parts(buffer_start_ptr, buffer_valid_length as usize)
        };

        return Ok(f(slice));

        //let mut transform: IMFTransform = unsafe {
        //    MFCreateSampleCopierMFT()?
        //};

        //transform.SetInputType(dwinputstreamid, ptype, dwflags)

        //let mut transform : IMFTransform = unsafe {
        //    self.source_reader.GetServiceForStream(MEDIA_FOUNDATION_FIRST_VIDEO_STREAM, 0)?
        //};

        //todo!()
    }

    pub fn receive_and_decode_frame(&mut self) -> MFResult<DynamicImage> {
        let format = self.get_current_format()?;
        let subtype = format.recognize_supported_media_subtype()
            .ok_or(MFError::Other("No supported media subtype".into()))?;

        let bytes = self.receive_raw_bytes(|bytes| bytes.to_vec())?;

        match subtype {
            VideoSubtype::Uncompressed(pixel_format) => {
                match pixel_format {
                    PixelFormat::RGB24 => {
                        let stride = format.stride().unwrap_or(format.width() as i32 * 3); // TODO check if row major

                        // HACK: windows actually uses BGR, so we need to convert it to swap the RED and BLUE channels
                        let img = birb_vision::decoders::decode_bgr(
                            &bytes,
                            format.width(),
                            format.height(),
                            stride,
                            true
                        );

                        Ok(DynamicImage::ImageRgb8(img))
                    }
                    PixelFormat::NV12 => Ok(DynamicImage::ImageRgb8(nv12_to_rgb_image(format.width(), format.height(), &bytes, false)?)),
                    PixelFormat::YUY2 => {
                        let mut pixels = vec![0u8; format.width() as usize * format.height() as usize * 3];
                        buf_yuyv422_to_rgb(&bytes, &mut pixels, true)?;
                        Ok(DynamicImage::ImageRgb8(RgbImage::from_raw(format.width(), format.height(), pixels).unwrap()))
                    }
                    _ => todo!("Uncompressed pixel format: {pixel_format:?}"),
                }
            },
            VideoSubtype::CompressedFrame(compressed_frame) => {
                match compressed_frame {
                    CompressedFrame::MJpeg => Ok(DynamicImage::ImageRgb8(decode_mjpg(&bytes)?)),
                }
            }
        }
    }

    pub fn flush(&mut self) -> MFResult<()> {
        unsafe {
            self.source_reader.Flush(MEDIA_FOUNDATION_FIRST_VIDEO_STREAM)?
        };

        Ok(())
    }
}

//struct Decoder {
//    transform: IMFTransform,
//}
//
//impl Decoder {
//    fn new(input_type: IMFMediaType) -> MFResult<Self> {
//        let transform = unsafe {
//            //MFCreateSampleCopierMFT(&input_type, &MF_VIDEO_FORMAT_YUY2, &MF_VIDEO_FORMAT_MJPEG)?
//        };
//        
//        Ok(Self {
//            transform,
//        })
//    }
//}

const MEDIA_FOUNDATION_FIRST_VIDEO_STREAM: u32 = 0xFFFF_FFFC;

const MF_VIDEO_FORMAT_YUY2: GUID = GUID::from_values(
    0x3259_5559,
    0x0000,
    0x0010,
    [0x80, 0x00, 0x00, 0xAA, 0x00, 0x38, 0x9B, 0x71],
);
const MF_VIDEO_FORMAT_MJPEG: GUID = GUID::from_values(
    0x4750_4A4D,
    0x0000,
    0x0010,
    [0x80, 0x00, 0x00, 0xAA, 0x00, 0x38, 0x9B, 0x71],
);
const MF_VIDEO_FORMAT_GRAY: GUID = GUID::from_values(
    0x3030_3859,
    0x0000,
    0x0010,
    [0x80, 0x00, 0x00, 0xAA, 0x00, 0x38, 0x9B, 0x71],
);
const MF_VIDEO_FORMAT_NV12: GUID = GUID::from_values(
    0x3231_564E,
    0x0000,
    0x0010,
    [0x80, 0x00, 0x00, 0xAA, 0x00, 0x38, 0x9B, 0x71],
);
const MF_VIDEO_FORMAT_RGB24: GUID = GUID::from_values(
    0x0000_0014,
    0x0000,
    0x0010,
    [0x80, 0x00, 0x00, 0xAA, 0x00, 0x38, 0x9B, 0x71],
);
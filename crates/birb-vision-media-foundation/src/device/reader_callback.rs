use std::{
    borrow::Cow,
    sync::{Arc, Mutex, Weak},
};

use birb_vision_core::{
    anyhow::anyhow, DeviceResult, FlatSample, FlatSampleLayout, FourCC, ImageSampleBuffer, NodeId, PixelFormat, Sample, SampleType, StreamEvent
};
use windows::Win32::Media::MediaFoundation::{
    IMFSourceReader, IMFSourceReaderCallback, IMFSourceReaderCallback_Impl,
    MF_SOURCE_READERF_ALLEFFECTSREMOVED, MF_SOURCE_READERF_CURRENTMEDIATYPECHANGED,
    MF_SOURCE_READERF_ENDOFSTREAM, MF_SOURCE_READERF_ERROR,
    MF_SOURCE_READERF_NATIVEMEDIATYPECHANGED, MF_SOURCE_READERF_NEWSTREAM,
    MF_SOURCE_READERF_STREAMTICK, MF_SOURCE_READER_FLAG,
};

use crate::{CompressedFrame, HandledPixelFormat, MFDevice, VideoFormat, VideoSubtype};

pub(super) struct ReaderCallbackInner {
    pub format: Option<VideoFormat>,
    pub subtype: Option<VideoSubtype>,
    pub tx: Box<dyn Fn(StreamEvent)>,
    pub source_reader: Weak<IMFSourceReader>,
    pub flushing: bool,
    pub capture: bool,
    //on_read_sample: Option<Box<dyn FnMut() -> windows_core::Result<()>>>,
    //on_flush: Option<Box<dyn FnMut() -> windows_core::Result<()>>>,
    //on_event: Option<Box<dyn FnMut() -> windows_core::Result<()>>>,
}

impl ReaderCallbackInner {
    pub fn send_event_impl(&mut self, event: StreamEvent) {
        (self.tx)(event);
    }

    pub fn send_sample(
        &mut self,
        sample: Result<DeviceResult<birb_vision_core::Sample>, MF_SOURCE_READER_FLAG>,
    ) {
        //self.send_event_impl(Event::Frame(sample.unwrap())); // TODO handle error

        let sample = match sample {
            Ok(frame) => frame,
            Err(flag) => match flag {
                MF_SOURCE_READERF_ERROR => todo!(),
                MF_SOURCE_READERF_ENDOFSTREAM => todo!(),
                MF_SOURCE_READERF_NEWSTREAM => todo!(),
                MF_SOURCE_READERF_NATIVEMEDIATYPECHANGED => todo!(),
                MF_SOURCE_READERF_CURRENTMEDIATYPECHANGED => todo!(),
                MF_SOURCE_READERF_STREAMTICK => {
                    return;
                }
                MF_SOURCE_READERF_ALLEFFECTSREMOVED => todo!(),
                _ => todo!(),
            },
        };

        self.send_event_impl(StreamEvent::Sample(sample));
    }

    pub fn send_flushed(&mut self) {
        self.send_event_impl(StreamEvent::Flushed);
    }

    pub fn send_property_changed(&mut self, property: NodeId) {
        self.send_event_impl(StreamEvent::PropertyChanged(property));
    }
}

#[windows::core::implement(IMFSourceReaderCallback)]
pub(super) struct ReaderCallback {
    pub inner: Arc<Mutex<ReaderCallbackInner>>,
}

impl ReaderCallback {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(ReaderCallbackInner {
                format: None,
                subtype: None,
                tx: Box::new(|_| {}),
                flushing: false,
                source_reader: Weak::new(),
                capture: false,
                //on_read_sample: None,
                //on_flush: None,
                //on_event: None,
            })),
        }
    }
}

impl IMFSourceReaderCallback_Impl for ReaderCallback_Impl {
    fn OnReadSample(
        &self,
        hrstatus: windows_core::HRESULT,
        dwstreamindex: u32,
        dwstreamflags: u32,
        _lltimestamp: i64,
        psample: Option<&windows::Win32::Media::MediaFoundation::IMFSample>,
    ) -> windows_core::Result<()> {
        //println!("OnReadSample: {hrstatus}, {dwstreamindex}, {dwstreamflags}, {lltimestamp}, {psample:?}");

        if dwstreamindex != 0 {
            todo!("dwstreamindex != 0");
        }

        let mut inner = self.inner.lock().unwrap();
        let source_reader = inner.source_reader.clone();
        let capture = inner.capture;

        scopeguard::defer! {
            if capture {
                if let Some(source_reader) = source_reader.upgrade() {
                    MFDevice::send_read_sample(&source_reader);
                }
            }
        }

        if dwstreamflags != 0 {
            inner.send_sample(Err(MF_SOURCE_READER_FLAG(dwstreamflags as _)));
            return Ok(());
        }

        if hrstatus.is_err() {
            let e = windows_core::Error::from_hresult(hrstatus);
            inner.send_sample(Ok(Err(anyhow!("Failed to read sample: {e}").into()))); // TODO better error handling
            return Ok(());
        }

        let Some(imf_sample) = psample else {
            inner.send_sample(Ok(Err(anyhow!("No sample").into()))); // TODO better error handling
            return Ok(());
        };

        let Some(format) = inner.format.clone() else {
            inner.send_sample(Ok(Err(anyhow!("No format").into()))); // TODO better error handling
            return Ok(());
        };

        let Some(subtype) = inner.subtype.clone() else {
            inner.send_sample(Ok(Err(anyhow!("No subtype").into()))); // TODO better error handling
            return Ok(());
        };

        let buffer = unsafe { imf_sample.ConvertToContiguousBuffer()? };

        let mut buffer_valid_length = 0;
        let mut buffer_start_ptr = std::ptr::null_mut::<u8>();

        unsafe { buffer.Lock(&mut buffer_start_ptr, None, Some(&mut buffer_valid_length))? };

        scopeguard::defer! {
            unsafe {
                if let Err(e) = buffer.Unlock() {
                    log::error!("Failed to unlock buffer: {e}");
                }
            }
        }

        let bytes =
            unsafe { std::slice::from_raw_parts(buffer_start_ptr, buffer_valid_length as usize) };

        let r = match subtype {
            VideoSubtype::Uncompressed(pixel_format) => {
                match pixel_format {
                    HandledPixelFormat::RGB24 => {
                        let stride = format.stride().unwrap_or(format.width() as i32 * 3); // TODO check if row major
                        Ok(FlatSample {
                            buffer: ImageSampleBuffer::Cow(Cow::Borrowed(bytes)),
                            layout: FlatSampleLayout {
                                offset: 0,
                                sample_type: SampleType::Plain(PixelFormat::BGR8Packed),
                                width: format.width(),
                                height: format.height(),
                                row_major: true,
                                stride,
                            },
                        })
                    }
                    HandledPixelFormat::NV12 => Ok(FlatSample {
                        buffer: ImageSampleBuffer::Cow(Cow::Borrowed(bytes)),
                        layout: FlatSampleLayout {
                            offset: 0,
                            sample_type: SampleType::FourCC(FourCC::new(b"NV12")),
                            width: format.width(),
                            height: format.height(),
                            row_major: true,
                            stride: 0,
                        },
                    }),
                    HandledPixelFormat::YUY2 => Ok(FlatSample {
                        buffer: ImageSampleBuffer::Cow(Cow::Borrowed(bytes)),
                        layout: FlatSampleLayout {
                            offset: 0,
                            sample_type: SampleType::FourCC(FourCC::new(b"YUY2")),
                            width: format.width(),
                            height: format.height(),
                            row_major: true,
                            stride: 0,
                        },
                    }),
                    _ => todo!("Uncompressed pixel format: {pixel_format:?}"),
                }
            }
            VideoSubtype::CompressedFrame(compressed_frame) => match compressed_frame {
                CompressedFrame::MJpeg => Ok(FlatSample {
                    buffer: ImageSampleBuffer::Cow(Cow::Borrowed(bytes)),
                    layout: FlatSampleLayout {
                        offset: 0,
                        sample_type: SampleType::FourCC(FourCC::new(b"MJPG")),
                        width: format.width(),
                        height: format.height(),
                        row_major: true,
                        stride: 0,
                    },
                }),
            },
        };

        inner.send_sample(Ok(r.map(Sample::ImageSample)));

        Ok(())
    }

    fn OnFlush(&self, dwstreamindex: u32) -> windows_core::Result<()> {
        if dwstreamindex != 0 {
            todo!("dwstreamindex != 0");
        }

        let mut inner = self.inner.lock().unwrap();
        inner.flushing = false;
        inner.send_flushed();
        Ok(())
    }

    fn OnEvent(
        &self,
        _dwstreamindex: u32,
        _pevent: Option<&windows::Win32::Media::MediaFoundation::IMFMediaEvent>,
    ) -> windows_core::Result<()> {
        //self.inner.lock().unwrap().send_event();
        // TODO
        Ok(())
    }
}

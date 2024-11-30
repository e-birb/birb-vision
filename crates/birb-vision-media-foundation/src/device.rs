use std::{borrow::Cow, cell::RefCell, error::Error, sync::{Arc, Mutex, Weak}};

use birb_vision_core::{anyhow::anyhow, context::{DeviceInfo, DeviceInfoEntry}, decoders::{decode_mjpg, nv12_to_rgb_image, yuyv422_to_rgb}, CameraDevice, DeviceResult, FlatSample, FlatSampleLayout, FourCC, ImageSampleBuffer, PixelFormat, Sample, SampleType, StreamEvent};
use image::{DynamicImage, RgbImage};
use serde::{Deserialize, Serialize};
use windows::{core::PWSTR, Win32::Media::MediaFoundation::{IMFAttributes, IMFMediaSource, IMFSourceReader, IMFSourceReaderCallback, IMFSourceReaderCallback_Impl, MFCreateAttributes, MFCreateSourceReaderFromMediaSource, MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_SYMBOLIC_LINK, MF_E_NOTACCEPTING, MF_READWRITE_DISABLE_CONVERTERS, MF_SOURCE_READERF_ALLEFFECTSREMOVED, MF_SOURCE_READERF_CURRENTMEDIATYPECHANGED, MF_SOURCE_READERF_ENDOFSTREAM, MF_SOURCE_READERF_ERROR, MF_SOURCE_READERF_NATIVEMEDIATYPECHANGED, MF_SOURCE_READERF_NEWSTREAM, MF_SOURCE_READERF_STREAMTICK, MF_SOURCE_READER_ASYNC_CALLBACK, MF_SOURCE_READER_FIRST_VIDEO_STREAM, MF_SOURCE_READER_FLAG}};

use crate::*;



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

pub struct MFDevice {
    _cx: MediaFoundationContext,
    info: MFDeviceInfo,
    is_streaming: Mutex<bool>,

    // Note: this SHALL be placed AFTER IMFSourceReader so that it is dropped AFTER IMFSourceReader
    _callback: Box<IMFSourceReaderCallback>,
    callback_inner: Arc<Mutex<ReaderCallbackInner>>,
    source_reader: Arc<IMFSourceReader>,
}

unsafe impl Send for MFDevice {}
unsafe impl Sync for MFDevice {}

impl MFDevice {
    pub fn info(&self) -> &MFDeviceInfo {
        &self.info
    }

    pub fn compatible_format_list(&self) -> MFResult<Vec<VideoFormat>> {
        let mut list = vec![];

        let mut index = 0;
        while let Ok(media_type) = unsafe {
            self.source_reader.GetNativeMediaType(FIRST_VIDEO_STREAM, index)
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
            self.source_reader.GetCurrentMediaType(FIRST_VIDEO_STREAM)?
        };

        VideoFormat::from_media_type(&media_type)
    }

    pub fn is_open(&self) -> bool {
        self.is_streaming.lock().unwrap().clone()
    }

    pub fn select_format(&mut self, query: impl Into<VideoFormatQuery>) -> MFResult<VideoFormat> {
        // This function if much different from the original nokhwa one: https://github.com/l1npengtul/nokhwa/blob/58454663b811f45388cc5a0cd681cb397ef51922/nokhwa-bindings-windows/src/lib.rs#L1009
        // but I think this is more correct

        let query: VideoFormatQuery = query.into();
        log::debug!("query: {:?}", query);

        let mut index = 0;
        while let Ok(media_type) = unsafe {
            self.source_reader.GetNativeMediaType(FIRST_VIDEO_STREAM, index)
        } {
            index += 1;

            let Ok(format) = VideoFormat::from_media_type(&media_type) else {
                log::warn!("Failed to parse media type {media_type:?}");
                continue;
            };

            // TODO use query.matches(&format) instead for better methods organization
            if format.satisfies(&query) {
                unsafe {
                    self.source_reader.SetCurrentMediaType(FIRST_VIDEO_STREAM, None, &media_type)?;
                }

                log::debug!("Selected format: {format:?}");

                return Ok(format);
            }
        }

        Err(MFError::Other("No matching format found".into()))
    }

    pub fn start_stream(&self) -> MFResult<()> {
        self.callback_inner.lock().unwrap().capture = true;
        unsafe {
            self.source_reader.SetStreamSelection(FIRST_VIDEO_STREAM, true)?
        };
        Self::send_read_sample(&self.source_reader);

        // The first call to ReadSample will start the stream and the callback will be called
        // with a "MF_SOURCE_READERF_STREAMTICK" flag which we ignore

        Ok(())
    }

    pub fn stop_stream(&self) -> MFResult<()> {
        self.callback_inner.lock().unwrap().capture = false;
        // TODO flush, wait for "flushed" with a timeout and set SetStreamSelection to false
        unsafe {
            //self.source_reader.SetStreamSelection(FIRST_VIDEO_STREAM, false)?
        };

        Ok(())
    }

    pub fn flush_reader(&self) -> MFResult<()> {
        unsafe {
            self.source_reader.Flush(FIRST_VIDEO_STREAM)?
        };

        Ok(())
    }
}

const FIRST_VIDEO_STREAM: u32 = MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32;

impl CameraDevice for MFDevice {
    fn get_device_info(&mut self) -> DeviceResult<DeviceInfo> {
        let mut info = DeviceInfo::new();
        info.display_name = self.info.name.clone();
        info.other.insert("symlink".into(), DeviceInfoEntry::new("symlink", self.info.symlink.clone()));

        Ok(info)
    }

    fn start_grabbing(&mut self) -> DeviceResult<()> {
        self.start_stream().unwrap(); // TODO handle error
        Ok(())
    }

    fn stop_grabbing(&mut self) -> DeviceResult<()> {
        self.stop_stream().unwrap(); // TODO handle error
        Ok(())
    }

    fn flush(&mut self) -> DeviceResult<()> {
        let mut inner = self.callback_inner.lock().unwrap();
        inner.flushing = true;
        self.flush_reader().unwrap();
        Ok(())
    }

    //async fn receive_frame(&self) -> DeviceResult<std::borrow::Cow<'_, birb_vision_core::Frame>> {
    //    let img = self.receive_and_decode_frame().unwrap(); // TODO handle error
    //    Ok(Cow::Owned(Frame::Image(img)))
    //}
    //fn poll_events(&self, ctx: &mut Context) -> Poll<DeviceResult<Event>> {
    //    let inner = &mut self.callback_inner.lock().unwrap();
    //    inner.waker = Some(ctx.waker().clone());
//
    //    if inner.format.is_none() || inner.subtype.is_none() {
    //        self.set_format_to_callback(inner)?;
    //    }
//
    //    while let Some(event) = inner.events.pop_front() {
    //        match event {
    //            Sample(Ok(frame)) => self.push_event(Event::Frame(frame)),
    //            Sample(Err(flag)) => match flag {
    //                MF_SOURCE_READERF_ERROR => {
    //                    // see https://learn.microsoft.com/en-us/windows/win32/api/mfreadwrite/ne-mfreadwrite-mf_source_reader_flag
    //                    // TODO do not make any further calls to IMFSourceReader methods.
    //                    // How to handle this?
    //                    // maybe invalidate the device with a flag?
    //                    todo!()
    //                },
    //                MF_SOURCE_READERF_ENDOFSTREAM => todo!(),
    //                MF_SOURCE_READERF_NEWSTREAM => todo!(),
    //                MF_SOURCE_READERF_NATIVEMEDIATYPECHANGED => todo!(),
    //                MF_SOURCE_READERF_CURRENTMEDIATYPECHANGED => {
    //                    // TODO this is a bit odd...
    //                    // Corrently, we do the conversion in the callback, but MAYBE this is incorrect
    //                    // and we should do it in the caller thread. However, this "current media type changed" flag
    //                    // is not alreadi well handled here since by the time we get here, the format may
    //                    // be changed again and we tell the callback a wrong format to decode the frame.
    //                    // By offloading the decoding to the caller thread, this would be even worse since there might
    //                    // be a bigger delay between the format change and the decoding due to other stuff that the caller
    //                    // thread might be doing.
    //                    // Giving the callback a reference to the source reader might be a solution, but the reader
    //                    // is not send nor sync. Of course since se set the callback using SetUnknown, the
    //                    // compiler would be unable to enforce this, but I feel like this is risky since it is not
    //                    // clear to me if the IMFSourceReader methods are thread safe AND if it is acceptable to call
    //                    // them during the callback invocation.
    //                    self.set_format_to_callback(inner)?;
    //                },
    //                MF_SOURCE_READERF_STREAMTICK => {
    //                    // "There is a gap in the stream"
    //                    self.push_event(Event::Flushed)
    //                },
    //                MF_SOURCE_READERF_ALLEFFECTSREMOVED => todo!(),
    //                _ => todo!(),
    //            },
    //            Flushed => {
    //                //println!("Flushed received");
    //                //return Poll::Pending;
    //            },
    //            Event => todo!(),
    //        }
    //    }
//
    //    self.finish_poll(inner)
    //}

    fn set_stream_callback(&mut self, f: Box<dyn Fn(StreamEvent) + Send + Sync>) -> DeviceResult {
        let mut inner = self.callback_inner.lock().unwrap();
        let inner = &mut *inner;

        if inner.format.is_none() || inner.subtype.is_none() {
            self.set_format_to_callback(inner)?;
        }

        inner.tx = f;

        Ok(())
    }

    fn grab(&mut self) -> DeviceResult<()> {
        Self::send_read_sample(&self.source_reader);
        Ok(())
    }
}

impl MFDevice {
    fn set_format_to_callback(&self, inner: &mut ReaderCallbackInner) -> DeviceResult<()> {
        let format = match self.get_current_format() {
            Ok(f) => f,
            Err(e) => return Err(anyhow!("Failed to get current format: {e}").into()), // TODO better error handling
        };
        let subtype = match format.recognize_supported_media_subtype() {
            Some(s) => s,
            //None => return Err(DeviceError::Unsupported),
            None => return Err(anyhow!("No supported media subtype").into()), // TODO better error handling
        };

        inner.format = Some(format);
        inner.subtype = Some(subtype);
        Ok(())
    }

    fn send_read_sample(source_reader: &IMFSourceReader) {
        loop {
            match unsafe {
                source_reader.ReadSample(
                    FIRST_VIDEO_STREAM,
                    0,
                    None,
                    None,
                    None,
                    None,
                )
            } {
                Ok(()) => {},
                Err(err) => {
                    // HACK MSDN says:
                    // "In Windows 7, there was a bug in the implementation of this method, which causes OnFlush to be called before the flush operation completes. A hotfix used to be available that fixed that bug."
                    // see:
                    // - https://learn.microsoft.com/en-us/windows/win32/api/mfreadwrite/nf-mfreadwrite-imfsourcereader-flush
                    // - https://learn.microsoft.com/en-us/windows/win32/api/mfreadwrite/nf-mfreadwrite-imfsourcereader-readsample#return-value
                    // since if we are here flush is false, it means that the we are in this bug case
                    // so we continue calling ReadSample until we do not get this error
                    if err.code() == MF_E_NOTACCEPTING {
                        std::thread::yield_now();
                        continue;
                    } else {
                        // TODO better error handling
                        //self.callback_inner.lock().unwrap().send_sample(Err(anyhow!("Failed to call read sample: {err}").into()));

                        //self.callback_inner.lock().unwrap().send_sample(Err(MF_SOURCE_READER_FLAG(err.code().0 as i32))); // TODO correct?
                        todo!()
                    }
                }
            }

            break;
        }
    }
}


struct ReaderCallbackInner {
    format: Option<VideoFormat>,
    subtype: Option<VideoSubtype>,
    tx: Box<dyn Fn(StreamEvent)>,
    source_reader: Weak<IMFSourceReader>,
    flushing: bool,
    capture: bool,

    //on_read_sample: Option<Box<dyn FnMut() -> windows_core::Result<()>>>,
    //on_flush: Option<Box<dyn FnMut() -> windows_core::Result<()>>>,
    //on_event: Option<Box<dyn FnMut() -> windows_core::Result<()>>>,
}

impl ReaderCallbackInner {
    fn send_event_impl(&mut self, event: StreamEvent) {
        (self.tx)(event);
    }

    fn send_sample(&mut self, sample: Result<DeviceResult<birb_vision_core::Sample>, MF_SOURCE_READER_FLAG>) {
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
                },
                MF_SOURCE_READERF_ALLEFFECTSREMOVED => todo!(),
                _ => todo!(),
            }
        };

        self.send_event_impl(StreamEvent::Sample(sample));
    }

    fn send_flushed(&mut self) {
        self.send_event_impl(StreamEvent::Flushed);
    }
}

#[windows::core::implement(IMFSourceReaderCallback)]
struct ReaderCallback {
    inner: Arc<Mutex<ReaderCallbackInner>>,
}

impl ReaderCallback {
    fn new() -> Self {
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
        lltimestamp: i64,
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

        let bytes = unsafe {
            std::slice::from_raw_parts(buffer_start_ptr, buffer_valid_length as usize)
        };

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
            },
            VideoSubtype::CompressedFrame(compressed_frame) => {
                match compressed_frame {
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
                }
            }
        };

        inner.send_sample(Ok(r.map(Sample::ImageSample)));

        Ok(())
    }

    fn OnFlush(
        &self,
        dwstreamindex: u32,
    ) -> windows_core::Result<()> {
        
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
        dwstreamindex: u32,
        pevent: Option<&windows::Win32::Media::MediaFoundation::IMFMediaEvent>,
    ) -> windows_core::Result<()> {
        //self.inner.lock().unwrap().send_event();
        // TODO
        Ok(())
    }
}

// Note: sync would have been a loop of:
// unsafe {
//    self.source_reader.ReadSample(
//        FIRST_VIDEO_STREAM,
//        0,
//        None,
//        Some(&mut stream_flags),
//        None,
//        Some(&mut imf_sample),
//    )?;
//}
// until we get a sample, but for async
// we set the last 4 arguments to None and we will get the sample in the callback.
// See:
// - https://learn.microsoft.com/en-us/windows/win32/api/mfreadwrite/nf-mfreadwrite-imfsourcereader-readsample#asynchronous-mode
// - https://learn.microsoft.com/en-us/windows/win32/medfound/using-the-source-reader-in-asynchronous-mode
// - https://chromium.googlesource.com/chromium/src/media/+/e3fa66c6b364174b9dd5d3759d160cdb8158caf7/video/capture/win/video_capture_device_mf_win.cc#223
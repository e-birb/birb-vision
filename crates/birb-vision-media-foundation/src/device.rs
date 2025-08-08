use std::{
    ffi::c_void,
    mem::MaybeUninit,
    sync::{Arc, Mutex},
};

use birb_vision_core::{
    anyhow::anyhow,
    context::{DeviceInfo, DeviceInfoEntry},
    BoolProperty, CameraDevice, CommandProperty, DeviceResult, Node, NodeId, NumericProperty,
    NumericState, Property, PropertyState, PropertyValue, Representation, StreamEvent, ValueOrRef,
};
use windows::Win32::Media::{
    DirectShow::{IAMCameraControl, IAMVideoProcAmp},
    KernelStreaming::GUID_NULL,
    MediaFoundation::{
        IMFSourceReader, IMFSourceReaderCallback, MF_E_NOTACCEPTING,
        MF_SOURCE_READER_FIRST_VIDEO_STREAM, MF_SOURCE_READER_MEDIASOURCE,
    },
};
use windows_core::Interface;

use crate::*;

mod control;
mod info;
mod reader_callback;

pub use control::*;
pub use info::MFDeviceInfo;
use reader_callback::*;

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
            self.source_reader
                .GetNativeMediaType(FIRST_VIDEO_STREAM, index)
        } {
            index += 1;

            match VideoFormat::list(&media_type) {
                Ok(framerates) => {
                    list.extend(framerates);
                }
                Err(e) => log::warn!(
                    "Failed to list video formats for media type {media_type:?}, error: {e}"
                ),
            };
        }

        Ok(list)
    }

    pub fn get_current_format(&self) -> MFResult<VideoFormat> {
        let media_type = unsafe { self.source_reader.GetCurrentMediaType(FIRST_VIDEO_STREAM)? };

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
            self.source_reader
                .GetNativeMediaType(FIRST_VIDEO_STREAM, index)
        } {
            index += 1;

            let Ok(format) = VideoFormat::from_media_type(&media_type) else {
                log::warn!("Failed to parse media type {media_type:?}");
                continue;
            };

            // TODO use query.matches(&format) instead for better methods organization
            if format.satisfies(&query) {
                unsafe {
                    self.source_reader.SetCurrentMediaType(
                        FIRST_VIDEO_STREAM,
                        None,
                        &media_type,
                    )?;
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
            self.source_reader
                .SetStreamSelection(FIRST_VIDEO_STREAM, true)?
        };
        Self::send_read_sample(&self.source_reader);

        // The first call to ReadSample will start the stream and the callback will be called
        // with a "MF_SOURCE_READERF_STREAMTICK" flag which we ignore

        Ok(())
    }

    pub fn stop_stream(&self) -> MFResult<()> {
        self.callback_inner.lock().unwrap().capture = false;
        // TODO flush, wait for "flushed" with a timeout and set SetStreamSelection to false
        //unsafe {
        //    self.source_reader.SetStreamSelection(FIRST_VIDEO_STREAM, false)?
        //};

        Ok(())
    }

    pub fn flush_reader(&self) -> MFResult<()> {
        unsafe { self.source_reader.Flush(FIRST_VIDEO_STREAM)? };

        Ok(())
    }

    pub fn get_control_range(&self, control: MFKnownControl) -> MFResult<MFControlRange> {
        let control_id = control.control_id().unwrap();

        let mut range = MFControlRange::default();

        match control_id {
            MFControlId::ProcAmp(property) => unsafe {
                self.get_media_source::<IAMVideoProcAmp>()?.GetRange(
                    property,
                    &mut range.min,
                    &mut range.max,
                    &mut range.stepping_delta,
                    &mut range.default,
                    &mut range.caps_flags,
                )?
            },
            MFControlId::CameraControl(property) => unsafe {
                self.get_media_source::<IAMCameraControl>()?.GetRange(
                    property,
                    &mut range.min,
                    &mut range.max,
                    &mut range.stepping_delta,
                    &mut range.default,
                    &mut range.caps_flags,
                )?
            },
        };

        Ok(range)
    }

    pub fn get_control_value(&self, control: MFKnownControl) -> MFResult<MFControlValue> {
        let control_id = control.control_id().unwrap();

        let mut value = MFControlValue::default();

        match control_id {
            MFControlId::ProcAmp(property) => unsafe {
                self.get_media_source::<IAMVideoProcAmp>()?.Get(
                    property,
                    &mut value.value,
                    &mut value.flags,
                )?
            },
            MFControlId::CameraControl(property) => unsafe {
                self.get_media_source::<IAMCameraControl>()?.Get(
                    property,
                    &mut value.value,
                    &mut value.flags,
                )?
            },
        };

        Ok(value)
    }

    pub fn set_control_value(
        &self,
        control: MFKnownControl,
        value: MFControlValue,
    ) -> MFResult<()> {
        let control_id = control.control_id().unwrap();

        match control_id {
            MFControlId::ProcAmp(property) => unsafe {
                self.get_media_source::<IAMVideoProcAmp>()?.Set(
                    property,
                    value.value,
                    value.flags,
                )?
            },
            MFControlId::CameraControl(property) => unsafe {
                self.get_media_source::<IAMCameraControl>()?.Set(
                    property,
                    value.value,
                    value.flags,
                )?
            },
        };

        Ok(())
    }

    fn get_media_source<T: Interface>(&self) -> MFResult<T> {
        // see https://github.com/l1npengtul/nokhwa/blob/aabdaeb0623208a31707ea838dfed555282e2890/nokhwa-bindings-windows/src/lib.rs#L836
        unsafe {
            let mut receiver: MaybeUninit<T> = MaybeUninit::uninit();
            if let Err(_why) = self.source_reader.GetServiceForStream(
                MF_SOURCE_READER_MEDIASOURCE.0 as u32,
                &GUID_NULL,
                &T::IID,
                receiver.as_mut_ptr().cast::<*mut c_void>(),
            ) {
                //return Err(NokhwaError::SetPropertyError {
                //    property: "MF_SOURCE_READER_MEDIASOURCE".to_string(),
                //    value: "IAMCameraControl".to_string(),
                //    error: why.to_string(),
                //});
                todo!();
            }
            Ok(receiver.assume_init())
        }
    }
}

const FIRST_VIDEO_STREAM: u32 = MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32;

impl CameraDevice for MFDevice {
    fn get_device_info(&mut self) -> DeviceResult<DeviceInfo> {
        let mut info = DeviceInfo::new();
        info.display_name = self.info.name.clone();
        info.other.insert(
            "symlink".into(),
            DeviceInfoEntry::new("symlink", self.info.symlink.clone()),
        );

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
    //                    // Currently, we do the conversion in the callback, but MAYBE this is incorrect
    //                    // and we should do it in the caller thread. However, this "current media type changed" flag
    //                    // is not already well handled here since by the time we get here, the format may
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

    fn all_properties(&mut self) -> DeviceResult<Vec<Node>> {
        fn to_property_node(dev: &MFDevice, control: MFKnownControl) -> DeviceResult<Option<Node>> {
            let r = dev.get_control_range(control);
            const ELEMENT_NOT_FOUND_ERROR_CODE: i32 = 0x80070490_u32 as i32; // TODO find the correct MF_E_* instead
            if let Err(MFError::WinError(err)) = &r {
                if err.code() == HRESULT(ELEMENT_NOT_FOUND_ERROR_CODE) {
                    log::warn!("Control {:?} not found", control);
                    return Ok(None);
                }
            };
            let range = r.map_err(|e| anyhow!("Failed to get control range: {e}"))?;
            let name = format!("{control:?}");
            let property = match control.kind() {
                MFKnownControlKind::Boolean => {
                    let default = range.default != 0;
                    let mut property = BoolProperty::new(NodeId::I32(control.into()));
                    property.display_name = name;
                    property.default = Some(default);
                    Property::Bool(property)
                }
                MFKnownControlKind::Range => {
                    let mut property = NumericProperty::<i64>::new(NodeId::I32(control.into()));
                    property.display_name = name;
                    property.min = Some(ValueOrRef::Value(range.min as i64));
                    property.max = Some(ValueOrRef::Value(range.max as i64));
                    property.default = Some(range.default as i64);
                    property.increment = Some(ValueOrRef::Value(range.stepping_delta as i64));
                    property.representation = Some(Representation::Linear);
                    Property::Integer(property)
                }
            };

            Ok(Some(Node::Property(property)))
        }

        let mut properties = vec![];
        let mut failed = 0;
        for control in MFKnownControl::ALL {
            match to_property_node(self, *control) {
                Ok(Some(node)) => properties.push(node),
                Ok(None) => {}
                Err(e) => {
                    log::error!(
                        "Failed to create property node for control {:?}: {}",
                        control,
                        e
                    );
                    failed += 1;
                    continue;
                }
            }
        }
        if failed > 0 {
            log::error!("Failed to create {failed} property nodes for MFDevice");
        }

        let mut default_button = CommandProperty::new(NodeId::String("reset-defaults".into()));
        default_button.display_name = "Reset Defaults".into();
        properties.push(Node::Property(Property::Command(default_button)));

        //Ok(vec![
        //    to_property_node(self, MFKnownControl::Brightness)?,
        //    to_property_node(self, MFKnownControl::Contrast)?,
        //    to_property_node(self, MFKnownControl::Hue)?,
        //    to_property_node(self, MFKnownControl::Saturation)?,
        //    to_property_node(self, MFKnownControl::Sharpness)?,
        //    to_property_node(self, MFKnownControl::Gamma)?,
        //    to_property_node(self, MFKnownControl::WhiteBalance)?,
        //    to_property_node(self, MFKnownControl::BacklightComp)?,
        //    to_property_node(self, MFKnownControl::Gain)?,
        //    to_property_node(self, MFKnownControl::Pan)?,
        //    to_property_node(self, MFKnownControl::Tilt)?,
        //    to_property_node(self, MFKnownControl::Zoom)?,
        //    to_property_node(self, MFKnownControl::Exposure)?,
        //    to_property_node(self, MFKnownControl::Iris)?,
        //    to_property_node(self, MFKnownControl::Focus)?,
        //])
        Ok(properties)
    }

    fn read_property(&mut self, id: &NodeId) -> DeviceResult<PropertyState> {
        let NodeId::I32(id) = id else {
            return Err(anyhow!("Invalid NodeId: {id:?}").into());
        };

        let control =
            MFKnownControl::try_from(*id).map_err(|_| anyhow!("Invalid control id: {id}"))?;

        let value = self
            .get_control_value(control)
            .map_err(|e| anyhow!("Failed to get control value: {e}"))?;

        let range = self
            .get_control_range(control)
            .map_err(|e| anyhow!("Failed to get control range: {e}"))?;

        let state = match control.kind() {
            MFKnownControlKind::Boolean => PropertyState::Bool(value.value != 0),
            MFKnownControlKind::Range => PropertyState::Int(NumericState {
                current: value.value as i64,
                range: range.min as i64..=range.max as i64,
            }),
        };

        Ok(state)
    }

    fn write_property(&mut self, id: &NodeId, value: PropertyValue) -> DeviceResult {
        if let NodeId::String(id) = id {
            if id == "reset-defaults" {
                let all = self.all_properties()?;
                for node in all {
                    if let Node::Property(property) = &node {
                        match property {
                            Property::Bool(prop) => {
                                if let Some(default) = prop.default {
                                    self.write_property(&node.id, PropertyValue::Bool(default))?;
                                }
                            }
                            Property::Integer(prop) => {
                                if let Some(default) = prop.default {
                                    self.write_property(&node.id, PropertyValue::Integer(default))?;
                                }
                            }
                            _ => {}
                        }
                    }
                }
                return Ok(());
            }
        }

        let NodeId::I32(id) = id else {
            return Err(anyhow!("Invalid NodeId: {id:?}").into());
        };

        let control =
            MFKnownControl::try_from(*id).map_err(|_| anyhow!("Invalid control id: {id}"))?;

        let current = self
            .get_control_value(control)
            .map_err(|e| anyhow!("Failed to get control value: {e}"))?;
        let mut new_value = current;

        match value {
            PropertyValue::Bool(value) => {
                new_value.value = if value { 1 } else { 0 };
            }
            PropertyValue::Integer(value) => {
                new_value.value = value as i32;
            }
            _ => Err(anyhow!("Unsupported property value type: {value:?}"))?,
        }

        self.set_control_value(control, new_value)
            .map_err(|e| anyhow!("Failed to set control value: {e}"))?;

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
            match unsafe { source_reader.ReadSample(FIRST_VIDEO_STREAM, 0, None, None, None, None) }
            {
                Ok(()) => {}
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

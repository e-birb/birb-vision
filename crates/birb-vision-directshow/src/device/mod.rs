use std::borrow::Cow;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::Mutex;

use birb_vision_core::{
    anyhow::anyhow,
    context::{DeviceInfo, DeviceInfoEntry},
    BoolProperty, CameraDevice, DeviceResult, FlatSample, FlatSampleLayout, ImageSampleBuffer,
    Node, NodeId, NumericProperty, NumericState, PixelFormat, Property, PropertyState,
    PropertyValue, Representation, Sample, SampleType, StreamEvent, ValueOrRef,
};
use serde::{Deserialize, Serialize};
use windows::Win32::Media::DirectShow::{
    CameraControl_Flags_Auto, CameraControl_Flags_Manual,
    IAMCameraControl, IAMVideoProcAmp, IBaseFilter, IFilterGraph2, IMediaControl,
    IMediaEventEx, IEnumPins, IPin, PIN_DIRECTION, PINDIR_INPUT, PINDIR_OUTPUT,
    VideoProcAmp_Flags_Auto, VideoProcAmp_Flags_Manual,
};
use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER};
use windows_core::Interface;

mod control;
mod sample_grabber;

pub use control::DSControl;
use sample_grabber::{
    CLSID_FilterGraph, CLSID_NullRenderer, CLSID_SampleGrabber,
    ISampleGrabber, AM_MEDIA_TYPE, MEDIASUBTYPE_RGB24, MEDIATYPE_Video,
};

use crate::*;

/// Information about a DirectShow video capture device.
///
/// Returned by [`DirectShowContext::enumerate_devices`](crate::DirectShowContext::enumerate_devices).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DSDeviceInfo {
    /// Human-readable name (e.g. "Logitech Webcam C930e").
    pub friendly_name: String,
    /// Unique device path (persistent across reboots, useful for re-identification).
    pub device_path: Option<String>,
}

impl DSDeviceInfo {
    pub fn friendly_name(&self) -> &str {
        &self.friendly_name
    }

    pub fn device_path(&self) -> Option<&str> {
        self.device_path.as_deref()
    }
}

/// A DirectShow video capture device.
///
/// Wraps a capture filter bound from the system device enumerator.  Provides
/// access to camera controls (brightness, contrast, pan, tilt, zoom, etc.)
/// via `IAMVideoProcAmp` and `IAMCameraControl`, and streams frames through
/// a DirectShow filter graph terminated with a Sample Grabber + Null Renderer.
///
/// # COM apartment
///
/// COM is initialised (apartment-threaded) on the first call to
/// [`DirectShowContext::new`].  The initialisation is tied to the calling
/// thread via a thread-local guard and automatically torn down when the
/// thread exits.
pub struct DirectShowDevice {
    _ctx: Arc<crate::ctx::CtxInner>,
    info: DSDeviceInfo,
    /// The DirectShow capture filter (IBaseFilter) obtained by binding the moniker.
    /// We store it as `IUnknown` so we can query for property interfaces on demand.
    filter: windows_core::IUnknown,
    /// Cached IAMVideoProcAmp interface, if available.
    proc_amp: Option<IAMVideoProcAmp>,
    /// Cached IAMCameraControl interface, if available.
    camera_control: Option<IAMCameraControl>,
    /// Cached list of supported property nodes.
    properties: Vec<Node>,
    callback: Arc<Mutex<Box<dyn Fn(StreamEvent) + Send + Sync>>>,
    /// Graph state, initialised lazily on first start_grabbing / grab call.
    graph_state: Mutex<Option<GraphState>>,
    /// Signal the polling thread to stop.
    stop_signal: Arc<AtomicBool>,
    /// Handle of the background frame-polling thread.
    poll_thread: Mutex<Option<std::thread::JoinHandle<()>>>,
}

/// Runtime state held while the filter graph is built and running.
struct GraphState {
    _graph: IFilterGraph2,
    media_control: IMediaControl,
    media_event: IMediaEventEx,
    sample_grabber: ISampleGrabber,
    width: u32,
    height: u32,
}

/// Helper sent to the background polling thread.
///
/// Holds the COM interface pointers needed to poll for frames
/// (`ISampleGrabber`, `IMediaEventEx`) together with the callback and
/// stop-signal.
///
/// COM interface pointers aren't `Send` by default in the windows crate,
/// so we wrap them and implement `Send` manually.  DirectShow filters are
/// free-threaded, so accessing them from a dedicated thread (after calling
/// `CoInitializeEx`) is safe.
struct GraphPoller {
    sg: ISampleGrabber,
    mev: IMediaEventEx,
    callback: Arc<Mutex<Box<dyn Fn(StreamEvent) + Send + Sync>>>,
    stop_signal: Arc<AtomicBool>,
    width: u32,
    height: u32,
}

// SAFETY: DirectShow COM objects are free-threaded and safe to access from
// any thread after CoInitializeEx has been called on that thread.
unsafe impl Send for GraphPoller {}

impl GraphPoller {
    fn run(self) {
        // Initialise COM on this thread
        let com_ok = unsafe {
            windows::Win32::System::Com::CoInitializeEx(
                None,
                windows::Win32::System::Com::COINIT_APARTMENTTHREADED
                    | windows::Win32::System::Com::COINIT_DISABLE_OLE1DDE,
            )
        }
        .is_ok();

        while !self.stop_signal.load(Ordering::SeqCst) {
            // Wait for an event from the graph (timeout 66 ms ≈ 15 fps)
            let mut ev_code = 0;
            let mut ev_param1 = 0;
            let mut ev_param2 = 0;
            let hr = unsafe {
                self.mev
                    .GetEvent(&mut ev_code, &mut ev_param1, &mut ev_param2, 66)
            };
            if hr.is_ok() {
                let _ = unsafe { self.mev.FreeEventParams(ev_code, ev_param1, ev_param2) };
            }

            // Check whether a new frame arrived
            let mut size = 0;
            let hr = unsafe { self.sg.GetCurrentBuffer(&mut size, std::ptr::null_mut()) };
            if hr.is_err() || size == 0 {
                continue;
            }

            // Allocate and read the buffer
            let mut buf: Vec<u8> = vec![0u8; size as usize];
            let hr = unsafe { self.sg.GetCurrentBuffer(&mut size, buf.as_mut_ptr() as *mut i32) };
            if hr.is_err() {
                continue;
            }
            buf.truncate(size as usize);

            let sample = build_flat_sample(&buf, self.width, self.height);

            if let Ok(cb) = self.callback.lock() {
                cb(StreamEvent::Sample(Ok(Sample::ImageSample(sample))));
            } else {
                // Mutex poisoned — the callback panicked, nothing more we can do
                break;
            }
        }

        if com_ok {
            unsafe { windows::Win32::System::Com::CoUninitialize() };
        }
    }
}

// COM interfaces are reference-counted pointers; they are thread-safe under the COM apartment model.
unsafe impl Send for DirectShowDevice {}
unsafe impl Sync for DirectShowDevice {}

impl DirectShowDevice {
    /// Create a new device from enumerated device info.
    ///
    /// Binds the device's moniker to an `IBaseFilter`, queries for
    /// `IAMVideoProcAmp` and `IAMCameraControl` interfaces, and
    /// enumerates all supported camera controls.
    ///
    /// Prefer using [`DirectShowContext::create`](crate::DirectShowContext) or
    /// the [`VisionContext`](birb_vision_core::context::VisionContext) trait instead.
    pub(crate) fn new(
        ctx: Arc<crate::ctx::CtxInner>,
        info: DSDeviceInfo,
    ) -> DSResult<Self> {
        // Bind the moniker to create the actual DirectShow capture filter
        let filter = ctx.bind_device_filter(&info)?;

        // Query for the two camera-control COM interfaces
        let proc_amp = filter.cast::<IAMVideoProcAmp>().ok();
        let camera_control = filter.cast::<IAMCameraControl>().ok();

        // Enumerate all known controls and cache the supported ones
        let properties = Self::enumerate_properties(proc_amp.as_ref(), camera_control.as_ref());

        Ok(Self {
            _ctx: ctx,
            info,
            filter: filter.into(),
            proc_amp,
            camera_control,
            properties,
            callback: Arc::new(Mutex::new(Box::new(|_| {}))),
            graph_state: Mutex::new(None),
            stop_signal: Arc::new(AtomicBool::new(false)),
            poll_thread: Mutex::new(None),
        })
    }

    /// Enumerate all known DirectShow controls, returning only those the camera supports.
    fn enumerate_properties(
        proc_amp: Option<&IAMVideoProcAmp>,
        camera_control: Option<&IAMCameraControl>,
    ) -> Vec<Node> {
        use strum::IntoEnumIterator;

        let mut nodes = Vec::new();

        for control in DSControl::iter() {
            let range = match Self::get_control_range(control, proc_amp, camera_control) {
                Ok(r) => r,
                Err(_) => continue, // property not supported by this device
            };

            let name = format!("{control:?}");
            let node_id = match control.into_node_id() {
                Ok(id) => id,
                Err(e) => {
                    log::error!("Failed to create NodeId for {control:?}: {e}");
                    continue;
                }
            };

            let access_mode = property_access_mode(range.caps_flags, control.kind());

            let property = if control.is_boolean() {
                let default = range.default != 0;
                let mut prop = BoolProperty::new(node_id);
                prop.display_name = name;
                prop.default = Some(default);
                prop.access_mode = access_mode;
                Property::Bool(prop)
            } else {
                let mut prop = NumericProperty::<i64>::new(node_id);
                prop.display_name = name;
                prop.min = Some(ValueOrRef::Value(range.min as i64));
                prop.max = Some(ValueOrRef::Value(range.max as i64));
                prop.default = Some(range.default as i64);
                prop.increment = Some(ValueOrRef::Value(range.stepping_delta.max(1) as i64));
                prop.representation = Some(Representation::Linear);
                prop.access_mode = access_mode;
                Property::Integer(prop)
            };

            nodes.push(Node::Property(property));
        }

        nodes
    }

    fn get_control_range(
        control: DSControl,
        proc_amp: Option<&IAMVideoProcAmp>,
        camera_control: Option<&IAMCameraControl>,
    ) -> DSResult<DSControlRange> {
        use control::DSControlKind;

        let mut range = DSControlRange::default();

        let hr = match control.kind() {
            DSControlKind::ProcAmp => {
                let Some(proc_amp) = proc_amp else {
                    return Err(DSError::msg("IAMVideoProcAmp not available"));
                };
                unsafe {
                    proc_amp.GetRange(
                        control.property_id(),
                        &mut range.min,
                        &mut range.max,
                        &mut range.stepping_delta,
                        &mut range.default,
                        &mut range.caps_flags,
                    )
                }
            }
            DSControlKind::CameraControl => {
                let Some(camera_control) = camera_control else {
                    return Err(DSError::msg("IAMCameraControl not available"));
                };
                unsafe {
                    camera_control.GetRange(
                        control.property_id(),
                        &mut range.min,
                        &mut range.max,
                        &mut range.stepping_delta,
                        &mut range.default,
                        &mut range.caps_flags,
                    )
                }
            }
        };

        // HRESULT 0x80070490 = E_PROP_ID_UNSUPPORTED (property not available)
        const E_PROP_ID_UNSUPPORTED: i32 = 0x80070490u32 as i32;
        if let Err(e) = &hr {
            if e.code() == windows_core::HRESULT(E_PROP_ID_UNSUPPORTED) {
                return Err(DSError::msg("Property not supported by this device"));
            }
        }

        hr.map_err(|e| DSError::msg(format!("GetRange failed: {e}")))?;

        Ok(range)
    }

    fn get_control_value(
        &self,
        control: DSControl,
    ) -> DSResult<DSControlValue> {
        use control::DSControlKind;

        let mut value = DSControlValue::default();

        match control.kind() {
            DSControlKind::ProcAmp => {
                let Some(ref proc_amp) = self.proc_amp else {
                    return Err(DSError::msg("IAMVideoProcAmp not available"));
                };
                unsafe {
                    proc_amp.Get(
                        control.property_id(),
                        &mut value.value,
                        &mut value.flags,
                    )?;
                }
            }
            DSControlKind::CameraControl => {
                let Some(ref camera_control) = self.camera_control else {
                    return Err(DSError::msg("IAMCameraControl not available"));
                };
                unsafe {
                    camera_control.Get(
                        control.property_id(),
                        &mut value.value,
                        &mut value.flags,
                    )?;
                }
            }
        }

        Ok(value)
    }

    fn set_control_value(
        &self,
        control: DSControl,
        value: DSControlValue,
    ) -> DSResult<()> {
        use control::DSControlKind;

        match control.kind() {
            DSControlKind::ProcAmp => {
                let Some(ref proc_amp) = self.proc_amp else {
                    return Err(DSError::msg("IAMVideoProcAmp not available"));
                };
                unsafe {
                    proc_amp.Set(control.property_id(), value.value, value.flags)?;
                }
            }
            DSControlKind::CameraControl => {
                let Some(ref camera_control) = self.camera_control else {
                    return Err(DSError::msg("IAMCameraControl not available"));
                };
                unsafe {
                    camera_control.Set(control.property_id(), value.value, value.flags)?;
                }
            }
        }

        Ok(())
    }

    /// Build the DirectShow filter graph: capture source → Sample Grabber → Null Renderer.
    ///
    /// Connects pins manually (avoids needing `ICaptureGraphBuilder2` which is
    /// not always registered, e.g. under Wine).
    fn build_graph(&self, filter: &windows_core::IUnknown) -> DSResult<GraphState> {
        // --- Filter Graph ---
        let graph: IFilterGraph2 = unsafe {
            CoCreateInstance(&CLSID_FilterGraph, None, CLSCTX_INPROC_SERVER)?
        };

        // --- Add the capture filter to the graph ---
        let base_filter: IBaseFilter = filter
            .cast()
            .map_err(|e| DSError::msg(format!("Failed to cast IUnknown to IBaseFilter: {e}")))?;

        unsafe {
            graph.AddFilter(
                &base_filter,
                &windows::core::HSTRING::from(&self.info.friendly_name),
            )?;
        }

        // --- Create and add the Sample Grabber filter ---
        let sample_grabber: ISampleGrabber = unsafe {
            CoCreateInstance(&CLSID_SampleGrabber, None, CLSCTX_INPROC_SERVER)?
        };

        let grabber_filter: IBaseFilter = sample_grabber
            .cast()
            .map_err(|e| DSError::msg(format!("Failed to cast ISampleGrabber to IBaseFilter: {e}")))?;

        unsafe {
            graph.AddFilter(&grabber_filter, &windows::core::HSTRING::from("Sample Grabber"))?;

            // Set media type to RGB24 so we get simple 24-bit BGR frames
            let mut mt: AM_MEDIA_TYPE = std::mem::zeroed();
            mt.majortype = MEDIATYPE_Video;
            mt.subtype = MEDIASUBTYPE_RGB24;
            sample_grabber.SetMediaType(&mt as *const AM_MEDIA_TYPE)?;
        }

        // --- Create and add the Null Renderer ---
        let null_renderer: IBaseFilter = unsafe {
            CoCreateInstance(&CLSID_NullRenderer, None, CLSCTX_INPROC_SERVER)?
        };

        unsafe {
            graph.AddFilter(&null_renderer, &windows::core::HSTRING::from("Null Renderer"))?;
        }

        // --- Connect capture filter output → sample grabber input ---
        let capture_pin = find_pin(&base_filter, PINDIR_OUTPUT)
            .ok_or_else(|| DSError::msg("No output pin on capture filter"))?;
        let grabber_in = find_pin(&grabber_filter, PINDIR_INPUT)
            .ok_or_else(|| DSError::msg("No input pin on sample grabber"))?;

        unsafe {
            graph.Connect(&capture_pin, &grabber_in)
                .map_err(|e| DSError::msg(format!("Failed to connect capture → sample grabber: {e}")))?;
        }

        // --- Connect sample grabber output → null renderer input ---
        let grabber_out = find_pin(&grabber_filter, PINDIR_OUTPUT)
            .ok_or_else(|| DSError::msg("No output pin on sample grabber"))?;
        let null_in = find_pin(&null_renderer, PINDIR_INPUT)
            .ok_or_else(|| DSError::msg("No input pin on null renderer"))?;

        unsafe {
            graph.Connect(&grabber_out, &null_in)
                .map_err(|e| DSError::msg(format!("Failed to connect sample grabber → null renderer: {e}")))?;
        }

        // --- Read back the connected media type to get dimensions ---
        let mut connected_mt: AM_MEDIA_TYPE = unsafe { std::mem::zeroed() };
        unsafe {
            sample_grabber.GetConnectedMediaType(&mut connected_mt as *mut AM_MEDIA_TYPE)?;
        }

        let (width, height) = parse_video_dimensions(&connected_mt);

        if !connected_mt.pbFormat.is_null() {
            unsafe {
                let _ = windows::Win32::System::Com::CoTaskMemFree(Some(connected_mt.pbFormat.cast()));
            }
        }

        // --- Configure sample grabber ---
        unsafe {
            sample_grabber.SetBufferSamples(true)?;
            sample_grabber.SetOneShot(false)?;
        }

        // --- Media Control ---
        let media_control: IMediaControl = graph
            .cast()
            .map_err(|e| DSError::msg(format!("Failed to get IMediaControl: {e}")))?;

        // --- Media Event ---
        let media_event: IMediaEventEx = graph
            .cast()
            .map_err(|e| DSError::msg(format!("Failed to get IMediaEventEx: {e}")))?;

        Ok(GraphState {
            _graph: graph,
            media_control,
            media_event,
            sample_grabber,
            width,
            height,
        })
    }

    /// Helper: build or reuse the graph, returning a lock guard on graph_state.
    fn ensure_graph(&self) -> DSResult<std::sync::MutexGuard<'_, Option<GraphState>>> {
        let mut gs = self.graph_state.lock().map_err(|e| {
            DSError::msg(format!("graph_state mutex poisoned: {e}"))
        })?;
        if gs.is_none() {
            *gs = Some(self.build_graph(&self.filter)?);
        }
        Ok(gs)
    }
}

/// Parse video dimensions from a connected `AM_MEDIA_TYPE`.
///
/// Reads the `BITMAPINFOHEADER` embedded inside the format block.
/// At offset 48 (`VIDEOINFOHEADER`) or 112 (`VIDEOINFOHEADER2`) we find:
///   biSize(4) | biWidth(4) | biHeight(4) | …
fn parse_video_dimensions(mt: &AM_MEDIA_TYPE) -> (u32, u32) {
    if mt.pbFormat.is_null() || mt.cbFormat < 52 {
        return (0, 0);
    }
    unsafe {
        for &bmi_offset in &[48isize, 112isize] {
            if mt.cbFormat < (bmi_offset as u32 + 12) {
                continue;
            }
            let ptr = mt.pbFormat.offset(bmi_offset);
            let bi_size = *(ptr as *const u32);      // biSize
            let bi_width = *(ptr.add(4) as *const i32);  // biWidth
            let bi_height = *(ptr.add(8) as *const i32); // biHeight
            // Sanity: biSize should be at least 40 for BITMAPINFOHEADER
            if bi_size >= 40 && bi_width > 0 && bi_height != 0 {
                return (bi_width as u32, bi_height.unsigned_abs());
            }
        }
    }
    (0, 0)
}

/// Build a `FlatSample` from a raw RGB24 buffer with given dimensions.
fn build_flat_sample(buf: &[u8], width: u32, height: u32) -> FlatSample<ImageSampleBuffer<'static>> {
    FlatSample {
        buffer: ImageSampleBuffer::Cow(Cow::Owned(buf.to_vec())),
        layout: FlatSampleLayout {
            offset: 0,
            sample_type: SampleType::Plain(PixelFormat::BGR8Packed),
            width,
            height,
            row_major: true,
            stride: (width * 3) as i32,
        },
    }
}

/// Find the first pin on `filter` with the given direction.
fn find_pin(filter: &IBaseFilter, direction: PIN_DIRECTION) -> Option<IPin> {
    unsafe {
        let enum_pins: IEnumPins = filter.EnumPins().ok()?;
        let mut pins = [None as Option<IPin>];
        loop {
            let mut fetched = 0u32;
            if enum_pins.Next(&mut pins, Some(&mut fetched)).is_err() || fetched == 0 {
                return None;
            }
            let pin = pins[0].take()?;
            if let Ok(pin_dir) = pin.QueryDirection() {
                if pin_dir == direction {
                    return Some(pin);
                }
            }
        }
    }
}

impl CameraDevice for DirectShowDevice {
    fn get_device_info(&self) -> DeviceResult<DeviceInfo> {
        let mut info = DeviceInfo::new();
        info.display_name = self.info.friendly_name.clone();
        if let Some(ref path) = self.info.device_path {
            info.other.insert(
                "path".into(),
                DeviceInfoEntry::new("Device Path", path.clone()),
            );
        }
        Ok(info)
    }

    fn set_stream_callback(&self, f: Box<dyn Fn(StreamEvent) + Send + Sync>) -> DeviceResult {
        *self.callback.lock().map_err(|e| anyhow!("{e}"))? = f;
        Ok(())
    }

    fn start_grabbing(&self) -> DeviceResult {
        // Stop any previous polling thread before building a new graph
        self.stop_signal.store(true, Ordering::SeqCst);
        if let Some(handle) = self.poll_thread.lock().map_err(|e| anyhow!("{e}"))?.take() {
            let _ = handle.join();
        }
        self.stop_signal.store(false, Ordering::SeqCst);

        // Build (or reuse) the filter graph
        let mut gs_opt = self
            .ensure_graph()
            .map_err(|e| anyhow!("{e}"))?;
        let gs = gs_opt.as_mut().ok_or_else(|| anyhow!("Graph state is None after ensure_graph"))?;

        // Start the filter graph
        unsafe {
            gs.media_control
                .Run()
                .map_err(|e| anyhow!("Failed to start filter graph: {e}"))?;
        }

        // Spawn a background thread that polls for new frames.
        //
        // COM interface pointers are not `Send`, so we wrap them in a helper
        // that we explicitly mark as Send (DirectShow objects are free-threaded).
        let callback = self.callback.clone();
        let stop_signal = self.stop_signal.clone();

        let poller = GraphPoller {
            sg: gs.sample_grabber.clone(),
            mev: gs.media_event.clone(),
            callback,
            stop_signal,
            width: gs.width,
            height: gs.height,
        };

        let handle = std::thread::Builder::new()
            .name("ds-poll".into())
            .spawn(move || {
                poller.run();
            })
            .map_err(|e| anyhow!("Failed to spawn polling thread: {e}"))?;

        *self.poll_thread.lock().map_err(|e| anyhow!("{e}"))? = Some(handle);

        Ok(())
    }

    fn stop_grabbing(&self) -> DeviceResult {
        // Signal the polling thread to stop
        self.stop_signal.store(true, Ordering::SeqCst);
        if let Some(handle) = self.poll_thread.lock().map_err(|e| anyhow!("{e}"))?.take() {
            let _ = handle.join();
        }
        self.stop_signal.store(false, Ordering::SeqCst);

        // Stop the filter graph and tear it down so the next start_grabbing
        // creates a fresh graph.
        if let Some(gs) = self.graph_state.lock().map_err(|e| anyhow!("{e}"))?.take() {
            unsafe {
                let _ = gs.media_control.Stop();
            }
            // Dropping gs here releases all COM references
        }

        Ok(())
    }

    fn grab(&self) -> DeviceResult {
        let mut gs_opt = self
            .ensure_graph()
            .map_err(|e| anyhow!("{e}"))?;
        let gs = gs_opt.as_mut().ok_or_else(|| anyhow!("Graph state is None after ensure_graph"))?;

        // If a polling thread is already running (continuous streaming), just
        // read the latest buffered frame — don't touch SetOneShot.
        let is_streaming = self.poll_thread.lock().map_err(|e| anyhow!("{e}"))?.is_some();

        if is_streaming {
            // Read the current buffer without changing one-shot mode
            let mut size = 0;
            unsafe {
                gs.sample_grabber
                    .GetCurrentBuffer(&mut size, std::ptr::null_mut())
                    .map_err(|e| anyhow!("GetCurrentBuffer (size query) failed: {e}"))?;
            }
            if size == 0 {
                return Err(anyhow!("No frame data available after grab").into());
            }

            let mut buf: Vec<u8> = vec![0u8; size as usize];
            unsafe {
                gs.sample_grabber
                    .GetCurrentBuffer(&mut size, buf.as_mut_ptr() as *mut i32)
                    .map_err(|e| anyhow!("GetCurrentBuffer (read) failed: {e}"))?;
            }
            buf.truncate(size as usize);

            // Build the sample without holding the graph lock
            let sample = build_flat_sample(&buf, gs.width, gs.height);
            drop(gs_opt);

            if let Ok(cb) = self.callback.lock() {
                cb(StreamEvent::Sample(Ok(Sample::ImageSample(sample))));
            }

            return Ok(());
        }

        // --- One-shot mode (no active stream) ---
        unsafe {
            gs.sample_grabber
                .SetOneShot(true)
                .map_err(|e| anyhow!("SetOneShot failed: {e}"))?;
            gs.media_control
                .Run()
                .map_err(|e| anyhow!("Failed to run filter graph: {e}"))?;
        }

        // Wait for the frame to arrive (IMediaEvent with short timeout)
        let mut ev_code = 0;
        let mut ev_param1 = 0;
        let mut ev_param2 = 0;
        let timeout_ms = 2000;
        let hr = unsafe {
            gs.media_event
                .GetEvent(&mut ev_code, &mut ev_param1, &mut ev_param2, timeout_ms)
        };

        if let Err(ref err) = hr {
            if err.code() == windows_core::HRESULT(0x8007000Eu32 as i32) /* WAIT_TIMEOUT */ {
                return Err(anyhow!("Grab timed out after {timeout_ms}ms — no frame received").into());
            }
        } else {
            unsafe {
                let _ = gs.media_event.FreeEventParams(ev_code, ev_param1, ev_param2);
            }
        }

        // Read the buffer
        let mut size = 0;
        unsafe {
            gs.sample_grabber
                .GetCurrentBuffer(&mut size, std::ptr::null_mut())
                .map_err(|e| anyhow!("GetCurrentBuffer (size query) failed: {e}"))?;
        }
        if size == 0 {
            return Err(anyhow!("No frame data available after grab").into());
        }

        let mut buf: Vec<u8> = vec![0u8; size as usize];
        unsafe {
            gs.sample_grabber
                .GetCurrentBuffer(&mut size, buf.as_mut_ptr() as *mut i32)
                .map_err(|e| anyhow!("GetCurrentBuffer (read) failed: {e}"))?;
        }
        buf.truncate(size as usize);

        // Build the sample without holding the graph lock
        let sample = build_flat_sample(&buf, gs.width, gs.height);
        drop(gs_opt);

        if let Ok(cb) = self.callback.lock() {
            cb(StreamEvent::Sample(Ok(Sample::ImageSample(sample))));
        }

        Ok(())
    }

    fn all_properties(&self) -> DeviceResult<Vec<Node>> {
        Ok(self.properties.clone())
    }

    fn read_property(&self, id: &NodeId) -> DeviceResult<PropertyState> {
        let node_id = DSControl::from_node_id(id)?;

        let control::DSNodeId::Control(control) = node_id;

        let value = self
            .get_control_value(control)
            .map_err(|e| anyhow!("Failed to get control value: {e}"))?;

        let range = Self::get_control_range(control, self.proc_amp.as_ref(), self.camera_control.as_ref())
            .map_err(|e| anyhow!("Failed to get control range: {e}"))?;

        let state = if control.is_boolean() {
            PropertyState::Bool(value.value != 0)
        } else {
            PropertyState::Int(NumericState {
                current: value.value as i64,
                range: range.min as i64..=range.max as i64,
            })
        };

        Ok(state)
    }

    fn write_property(&self, id: &NodeId, value: PropertyValue) -> DeviceResult {
        let node_id = DSControl::from_node_id(id)?;

        let control::DSNodeId::Control(control) = node_id;

        let flags = manual_flag_for_kind(control.kind());

        let raw = match (control.is_boolean(), value) {
            (true, PropertyValue::Bool(v)) => {
                DSControlValue {
                    value: if v { 1 } else { 0 },
                    flags,
                }
            }
            (false, PropertyValue::Integer(v)) => {
                DSControlValue {
                    value: v as i32,
                    flags,
                }
            }
            _ => return Err(anyhow!("Unexpected property value type for control {control:?}").into()),
        };

        self.set_control_value(control, raw)
            .map_err(|e| anyhow!("Failed to set control value: {e}"))?;

        Ok(())
    }
}

/// Return the `Manual` flag constant for the given control kind.
///
/// Both `VideoProcAmp_Flags_Manual` and `CameraControl_Flags_Manual` have
/// the same numeric value (`0x01`), but using the correct constant is
/// clearer and more defensive.
fn manual_flag_for_kind(kind: control::DSControlKind) -> i32 {
    match kind {
        control::DSControlKind::ProcAmp => VideoProcAmp_Flags_Manual.0,
        control::DSControlKind::CameraControl => CameraControl_Flags_Manual.0,
    }
}

/// Return the `Auto` flag constant for the given control kind.
fn auto_flag_for_kind(kind: control::DSControlKind) -> i32 {
    match kind {
        control::DSControlKind::ProcAmp => VideoProcAmp_Flags_Auto.0,
        control::DSControlKind::CameraControl => CameraControl_Flags_Auto.0,
    }
}

/// Convert DirectShow `VideoProcAmpFlags` / `CameraControlFlags` caps to `AccessMode`.
fn property_access_mode(caps_flags: i32, kind: control::DSControlKind) -> birb_vision_core::AccessMode {
    use birb_vision_core::AccessMode;

    // Sanity-check: the flag values are identical across both interfaces.
    debug_assert_eq!(VideoProcAmp_Flags_Manual.0, CameraControl_Flags_Manual.0);
    debug_assert_eq!(VideoProcAmp_Flags_Auto.0, CameraControl_Flags_Auto.0);

    let has_manual = (caps_flags & manual_flag_for_kind(kind)) != 0;
    let has_auto = (caps_flags & auto_flag_for_kind(kind)) != 0;

    match (has_manual, has_auto) {
        (true, _) => AccessMode::ReadWrite,
        (false, true) => AccessMode::ReadOnly,
        (false, false) => AccessMode::ReadWrite,
    }
}

/// Re-export the range/value types used by the device constructor.
pub use control::DSControlRange;
pub use control::DSControlValue;
pub use control::DSNodeId;

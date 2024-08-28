

use std::{borrow::Cow, sync::{Arc, Mutex}};
use clap::ValueEnum;

use enum_as_inner::EnumAsInner;
use futures::stream::BoxStream;
pub use image;
pub mod decoders;
pub mod channels;

mod frame;
mod device_properties;
//mod pixel_format;

pub use frame::*;
pub use device_properties::*;
//pub use pixel_format::*;

pub use futures;
use serde::{Deserialize, Serialize};

pub mod backend;

#[derive(Debug)]
#[derive(EnumAsInner)]
// TODO #[derive(Serialize, Deserialize)]
pub enum Event {
    Frame(DeviceResult<Frame>),
    Flushed, // TODO maybe remove
    // TODO consider not having any events that are not "common" since the user may expect them
    // but never emitted by the implementation. Another possibility would be to group them
    // in another enum for non-common/ensured events.
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(EnumAsInner)]
#[derive(Serialize, Deserialize)]
#[derive(ValueEnum)]
pub enum DeviceAccessMode {

    /// Exclusive access to the device
    #[default]
    Exclusive,

    // TODO ExclusiveWithSwitch 

    /// Control access to the device
    ///
    /// Exclusive access to control the device, but
    /// other actors may still read from the device.
    Control,

    // TODO ControlWithSwitch
    // TODO ControlSwitchEnable
    // TODO ControlSwitchEnableWithKey

    /// Monitor access to the device
    Monitor,
}

/// Ciao
///
/// # Device Access
/// - [`is_device_accessible`]
/// - [`is_open`]
/// - [`open`]
/// - [`close`]
///
/// # Acquisition
/// - [`start_grabbing`]
/// - [`stop_grabbing`]
/// - [`grab`]
/// - [`flush`]
///
/// # Streaming
/// - [`stream`]
///
/// [`is_device_accessible`]: CameraDevice::is_device_accessible
/// [`is_open`]: CameraDevice::is_open
/// [`open`]: CameraDevice::open
/// [`close`]: CameraDevice::close
/// [`start_grabbing`]: CameraDevice::start_grabbing
/// [`stop_grabbing`]: CameraDevice::stop_grabbing
/// [`grab`]: CameraDevice::grab
/// [`flush`]: CameraDevice::flush
/// [`stream`]: CameraDevice::stream
pub trait CameraDevice {
    fn is_device_accessible(&self, mode: DeviceAccessMode) -> bool;
    fn is_open(&self) -> Option<DeviceAccessMode>;
    fn open(&self, mode: DeviceAccessMode) -> DeviceResult;
    fn close(&self) -> DeviceResult;

    fn control_description(&self) -> DeviceResult<Node>;
    fn properties(&self) -> DeviceResult<Node>;
    fn get_bool_property(&self, id: &NodeId) -> DeviceResult<bool>;
    fn get_int_property(&self, id: &NodeId) -> DeviceResult<NumericValue<i64>>;
    fn get_float_property(&self, id: &NodeId) -> DeviceResult<NumericValue<f64>>;
    fn get_enum_property(&self, id: &NodeId) -> DeviceResult<EnumValue>;
    fn get_string_property(&self, id: &NodeId) -> DeviceResult<String>; // TODO Cow

    fn set_property(&self, id: &NodeId, value: &PropertyValue) -> DeviceResult;
    fn set_bool_property(&self, id: &NodeId, value: bool) -> DeviceResult;
    fn set_int_property(&self, id: &NodeId, value: i64) -> DeviceResult;
    fn set_float_property(&self, id: &NodeId, value: f64) -> DeviceResult;
    fn set_enum_property(&self, id: &NodeId, value: i64) -> DeviceResult;
    fn set_string_property(&self, id: &NodeId, value: &str) -> DeviceResult;
    fn send_command(&self, id: &NodeId) -> DeviceResult;

    fn start_grabbing(&self) -> DeviceResult;
    fn stop_grabbing(&self) -> DeviceResult; // TODO a stream object

    /// Tell the camera to read a sample from the stream
    ///
    /// This acts as a sort of "software trigger" for the camera, telling it to read a sample from the stream.
    /// The actual behavior is implementation-defined, and may not have any effect on some devices or in some
    /// configurations such as:
    /// - the camera is not streaming
    /// - the camera is in not is software-triggerable mode or is in free-run mode
    ///
    /// # Notes
    /// - This method is similar to [OpenCV's `VideoCapture::grab`](https://github.com/opencv/opencv/blob/ae4a11b0c0986809d2f938f68343c8da99286b29/modules/videoio/include/opencv2/videoio.hpp#L878), but it is not guaranteed to have effect.
    fn grab(&self) -> DeviceResult<()>;

    fn flush(&self) -> DeviceResult;

    /// Similar to [futures::stream::Stream::poll_next] but with no Pin requirement
    // TODO Is this "no Pin requirement" good?
    //fn poll_events(&self, ctx: &mut std::task::Context) -> Poll<DeviceResult<Event>>;

    fn set_stream_callback(&self, f: Box<dyn Fn(Event) + Send + Sync>) -> DeviceResult;
}

//impl<'a, T: CameraDevice + ?Sized> Future for NextFrame<'a, T> {
//    type Output = DeviceResult<Event>;
//
//    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Self::Output> {
//        self.device.poll_events(cx)
//    }
//}

pub trait CameraDeviceEx: CameraDevice {
    //fn stream(&self, buf: usize) -> DeviceResult<BoxStream<Event>> {
    //    let (tx, rx) = futures::channel::mpsc::channel(buf);
    //    let tx = Mutex::new(tx);
    //    self.set_stream_callback(Box::new(move |e| {
    //        // TODO maybe just clone instead of using the mutex?
    //        tx.lock().unwrap().try_send(e).unwrap(); // TODO handle error
    //    }))?;
    //    Ok(Box::pin(rx))
    //}

    /*fn next_event(&self) -> impl Future<Output = DeviceResult<Event>> {
        NextFrame { device: self }
    }

    fn next_frame(&self) -> impl Future<Output = DeviceResult<Frame>> {
        async {
            loop {
                match self.next_event().await? {
                    Event::Frame(frame) => return frame,
                    _ => continue,
                }
            }
        }
    }*/
}

impl<T: CameraDevice + ?Sized> CameraDeviceEx for T {}

#[derive(Debug, Clone)]
pub enum DeviceError {
    Unsupported,
    Other(Arc<dyn std::error::Error + Send + Sync>),
    OtherString(Cow<'static, str>),
}

impl DeviceError {
    pub fn other<E>(e: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        DeviceError::Other(Arc::new(e))
    }
}

impl std::fmt::Display for DeviceError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DeviceError::Unsupported => write!(f, "Operation is not supported"),
            DeviceError::Other(e) => write!(f, "Error: {}", e),
            DeviceError::OtherString(s) => write!(f, "{}", s),
        }
    }
}

impl From<&'static str> for DeviceError {
    fn from(s: &'static str) -> Self {
        DeviceError::OtherString(s.into())
    }
}

impl From<String> for DeviceError {
    fn from(s: String) -> Self {
        DeviceError::OtherString(s.into())
    }
}

// TODO conversions!

impl std::error::Error for DeviceError {}

pub type DeviceResult<T = ()> = Result<T, DeviceError>;


use std::{future::Future, sync::Mutex};
use backend::DeviceInfo;
use clap::ValueEnum;

use enum_as_inner::EnumAsInner;
use futures::channel::oneshot;
pub use image;
pub mod decoders;
pub mod channels;

pub use anyhow;
pub use thiserror;

mod sample;
mod device_properties;
//mod pixel_format;

pub use sample::*;
pub use device_properties::*;
//pub use pixel_format::*;

pub use futures;
use serde::{Deserialize, Serialize};

pub mod backend;
mod event;

pub use event::*;

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

pub trait CameraDevice {
    fn get_device_info(&self) -> DeviceResult<DeviceInfo>;

    /// All controls
    fn property(&self, id: &NodeId) -> DeviceResult<Node>;

    /// Root of the interesting properties to be exposed to the user
    fn root_property(&self) -> DeviceResult<NodeId>;

    fn get_property(&self, id: &NodeId) -> DeviceResult<PropertyState>;
    fn set_property(&self, id: &NodeId, value: PropertyValue) -> DeviceResult;

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

    fn set_stream_callback(&self, f: Box<dyn for<'a> Fn(Event<'a>) + Send + Sync>) -> DeviceResult;
}

//impl<'a, T: CameraDevice + ?Sized> Future for NextFrame<'a, T> {
//    type Output = DeviceResult<Event>;
//
//    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Self::Output> {
//        self.device.poll_events(cx)
//    }
//}

pub trait CameraDeviceEx: CameraDevice {
    fn get_one_frame<'a>(&'a self) -> impl Future<Output = DeviceResult<Sample<'static>>> + 'a {
        async move {
            let (tx, rx) = oneshot::channel();
            let tx = Mutex::new(Some(tx));

            self.set_stream_callback(Box::new(move |event| {
                match event {
                    Event::Frame(frame) => {
                        if let Some(tx) = tx.lock().unwrap().take() {
                            if let Err(e) = tx.send(frame.map(|s| s.into_owned())) {
                                log::error!("Error sending frame: {:?}", e);
                            }
                        }
                    },
                    _ => {},
                }
            }))?;

            self.grab()?;

            let frame_result = rx.await.map_err(|e| anyhow::Error::from(e))?;

            Ok(frame_result?)
        }
    }

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

//#[derive(Debug)]
//pub enum DeviceError {
//    Unsupported,
//    Other(Box<dyn std::error::Error>),
//}

//impl<E> From<E> for DeviceError
//where
//    E: Into<Box<dyn std::error::Error + 'static>> + 'static,
//{
//    fn from(value: E) -> Self {
//        DeviceError::Other(value.into())
//    }
//}

#[derive(thiserror::Error, Debug)]
pub enum DeviceError {
    #[error("Device is not accessible in the requested mode")]
    NotAccessible,

    #[error("Invalid parameter")]
    InvalidParameter,

    #[error("Operation is not supported")]
    Unsupported,

    #[error("Functionality not implemented")]
    NotImplemented,

    #[error("Buffer overflow")]
    BufferOverflow,

    #[error("Call order error, this function cannot be called at this time")]
    CallOrderError,

    #[error("No data available")]
    NoDataAvailable,

    #[error("Timeout")]
    Timeout,

    #[error("Version mismatch")]
    VersionMismatch,

    #[error("Library load error")]
    LibraryLoadError,

    #[error("Input/output error: {0}")]
    IO(#[from] std::io::Error),

    #[error("Invalid Node ID")]
    InvalidNodeId,

    #[error("Unsupported Format")]
    UnsupportedFormat,

    //#[error("Error: {0}")]
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl DeviceError {
    pub fn other<E>(e: E) -> Self
    where
        E: Into<anyhow::Error>,
    {
        DeviceError::Other(e.into())
    }
}

pub type DeviceResult<T = ()> = Result<T, DeviceError>;
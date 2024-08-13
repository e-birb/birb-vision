

use std::{borrow::Cow, future::Future, rc::Rc, task::Poll};

pub use image;
pub mod decoders;

mod frame;
//mod device_properties;
//mod pixel_format;

pub use frame::*;
//pub use device_properties::*;
//pub use pixel_format::*;

pub trait Context {
    
}

pub type FrameCallback = dyn for<'a> Fn(Cow<'a, Frame>) + Send + Sync + 'static;

pub trait CameraDevice {
    fn open(&self) -> DeviceResult;
    fn close(&self) -> DeviceResult;

    fn start_video_stream(&self) -> DeviceResult;
    fn stop_video_stream(&self) -> DeviceResult;

    fn flush(&self) -> DeviceResult;

    /// Similar to [futures::stream::Stream::poll_next] but with no Pin requirement
    // TODO Is this "no Pin requirement" good?
    fn poll_frame(&self, ctx: &mut std::task::Context) -> Poll<DeviceResult<Cow<Frame>>>;
}

// TODO consider using futures::stream::Next instead of this
pub struct NextFrame<'a, T: CameraDevice + ?Sized> { // TODO pub not needed?
    device: &'a T,
}

impl<'a, T: CameraDevice + ?Sized> Future for NextFrame<'a, T> {
    type Output = DeviceResult<Cow<'a, Frame>>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Self::Output> {
        self.device.poll_frame(cx)
    }
}

pub trait CameraDeviceEx: CameraDevice {
    fn get_frame(&self) -> impl Future<Output = DeviceResult<Cow<Frame>>> {
        NextFrame { device: self }
    }
}

impl<T: CameraDevice + ?Sized> CameraDeviceEx for T {}

#[derive(Debug, Clone)]
pub enum DeviceError {
    Unsupported,
    Other(Rc<dyn std::error::Error + Send + Sync>),
    OtherString(Cow<'static, str>),
}

impl DeviceError {
    pub fn other<E>(e: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        DeviceError::Other(Rc::new(e))
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
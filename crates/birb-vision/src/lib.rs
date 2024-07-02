

use std::{borrow::Cow, future::Future, ops::{Deref, DerefMut}, rc::Rc};

pub use image;
pub mod decoders;

mod frame;
//mod device_properties;
//mod pixel_format;

pub use frame::*;
//pub use device_properties::*;
//pub use pixel_format::*;

//pub type AsyncTask<'a, T = ()> = std::pin::Pin<Box<dyn Future<Output = T> + 'a>>;
#[must_use]
pub struct AsyncTask<'a, T = ()> {
    inner: std::pin::Pin<Box<dyn Future<Output = T> + 'a>>,
}

impl<'a, T> AsyncTask<'a, T> {
    pub fn new<F>(f: F) -> Self
    where
        F: Future<Output = T> + 'a,
    {
        Self {
            inner: Box::pin(f),
        }
    }
}

impl<'a, T> Deref for AsyncTask<'a, T> {
    type Target = std::pin::Pin<Box<dyn Future<Output = T> + 'a>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, T> DerefMut for AsyncTask<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<'a, T> From<std::pin::Pin<Box<dyn Future<Output = T> + 'a>>> for AsyncTask<'a, T> {
    fn from(inner: std::pin::Pin<Box<dyn Future<Output = T> + 'a>>) -> Self {
        Self { inner }
    }
}

impl<'a, T> Future for AsyncTask<'a, T> {
    type Output = T;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        Future::poll(self.get_mut().inner.as_mut(), cx)
    }
}

pub trait CameraDevice {
    fn open(&mut self) -> AsyncTask<DeviceResult<()>>;
    fn close(&mut self) -> AsyncTask<DeviceResult<()>>;

    fn start_video_stream(&mut self) -> AsyncTask<DeviceResult<()>>;
    fn stop_video_stream(&mut self) -> AsyncTask<DeviceResult<()>>;

    fn receive_frame(&mut self) -> AsyncTask<DeviceResult<Cow<'_, Frame>>>;
}

#[derive(Debug, Clone)]
pub enum DeviceError {
    Unsupported,
    Other(Rc<dyn std::error::Error + Send + Sync>),
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
        }
    }
}

// TODO conversions!

impl std::error::Error for DeviceError {}

pub type DeviceResult<T> = Result<T, DeviceError>;
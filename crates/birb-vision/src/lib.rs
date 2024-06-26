

use std::{borrow::Cow, future::Future, rc::Rc};

pub use image;
use image::DynamicImage;

pub type AsyncTask<'a, T = ()> = std::pin::Pin<Box<dyn Future<Output = T> + 'a>>;

mod device_properties;
mod pixel_format;
mod frame;

pub use device_properties::*;
pub use pixel_format::*;
pub use frame::*;

pub trait CameraDevice {
    fn open(&mut self) -> AsyncTask<DeviceResult<()>>;
    fn close(&mut self) -> AsyncTask<DeviceResult<()>>;

    fn start_video_stream(&mut self) -> AsyncTask<DeviceResult<()>>;
    fn stop_video_stream(&mut self) -> AsyncTask<DeviceResult<()>>;

    fn receive_frame(&mut self) -> AsyncTask<DeviceResult<Cow<'_, DynamicImage>>>;
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

impl std::error::Error for DeviceError {}

pub type DeviceResult<T> = Result<T, DeviceError>;
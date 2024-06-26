use std::borrow::Cow;

use birb_vision::{image::DynamicImage, AsyncTask, CameraDevice, DeviceError, Frame};
use nokhwa::{Buffer, FormatDecoder, NokhwaError};

pub use birb_vision;
pub use image;


pub struct NokhwaCamera {
    format_decoder: Box<dyn Fn(Buffer) -> Result<DynamicImage, NokhwaError>>,
    pub camera: nokhwa::Camera,
}

impl NokhwaCamera {
    pub fn new<Decoder: FormatDecoder>(camera: nokhwa::Camera) -> Self
    where
        DynamicImage: From<image::ImageBuffer<<Decoder as FormatDecoder>::Output, Vec<u8>>>,
    {
        Self {
            format_decoder: Box::new(|buffer| {
                buffer.decode_image::<Decoder>().map(|img| img.into())
            }),
            camera,
        }
    }
}

impl CameraDevice for NokhwaCamera {
    fn open(&mut self) -> AsyncTask<birb_vision::DeviceResult<()>> {
        AsyncTask::new(async move {
            Ok(())
        })
    }

    fn close(&mut self) -> AsyncTask<birb_vision::DeviceResult<()>> {
        AsyncTask::new(async move {
            Err(DeviceError::Unsupported)
        })
    }

    fn start_video_stream(&mut self) -> AsyncTask<birb_vision::DeviceResult<()>> {
        AsyncTask::new(async move {
            self.camera.open_stream().map_err(|e| DeviceError::other(e))
        })
    }

    fn stop_video_stream(&mut self) -> AsyncTask<birb_vision::DeviceResult<()>> {
        AsyncTask::new(async move {
            self.camera.stop_stream().map_err(|e| DeviceError::other(e))
        })
    }

    fn receive_frame(&mut self) -> AsyncTask<birb_vision::DeviceResult<std::borrow::Cow<'_, Frame>>> {
        AsyncTask::new(async move {
            let frame = self.camera.frame().map_err(|e| DeviceError::other(e))?;
            //let decoded = frame.decode_image::<RgbFormat>().map_err(|e| DeviceError::other(e))?;
            let decoded = (self.format_decoder)(frame).map_err(|e| DeviceError::other(e))?;
            Ok(Cow::Owned(Frame::Image(decoded)))
        })
    }
}
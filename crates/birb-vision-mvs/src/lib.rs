use std::{borrow::Cow, time::Duration};

use birb_vision::{image::{DynamicImage, ImageBuffer, Luma}, AsyncTask, CameraDevice, DeviceError, DeviceResult};
use mvs::{device::AccessMode, MVSDevice};



pub struct MVSCamera {
    device: MVSDevice,
}

impl MVSCamera {
    pub fn new(device: MVSDevice) -> Self {
        Self { device }
    }
}

impl CameraDevice for MVSCamera {
    fn open(&mut self) -> AsyncTask<DeviceResult<()>> {
        Box::pin(async move {
            self.device.open(
                AccessMode::Exclusive,
                None,
            ).map_err(|e| DeviceError::other(e))
        })
    }

    fn close(&mut self) -> AsyncTask<DeviceResult<()>> {
        Box::pin(async move {
            self.device.close().map_err(|e| DeviceError::other(e))
        })
    }

    fn start_video_stream(&mut self) -> AsyncTask<DeviceResult<()>> {
        Box::pin(async move {
            self.device.start_grabbing().map_err(|e| DeviceError::other(e))
        })
    }

    fn stop_video_stream(&mut self) -> AsyncTask<DeviceResult<()>> {
        Box::pin(async move {
            self.device.stop_grabbing().map_err(|e| DeviceError::other(e))
        })
    }

    fn receive_frame(&mut self) -> AsyncTask<DeviceResult<std::borrow::Cow<'_, DynamicImage>>> {
        Box::pin(async move {
            let w = self.device.get_int_value("Width").map_err(|e| DeviceError::other(e))?.current();
            let h = self.device.get_int_value("Height").map_err(|e| DeviceError::other(e))?.current();

            // TODO CHECK PIXEL FORMAT

            let mut buf = ImageBuffer::<Luma<u8>, Vec<u8>>::new(w as u32, h as u32).into_raw();

            self.device.get_one_frame_timeout(&mut buf, Duration::from_secs(1)).map_err(|e| DeviceError::other(e))?;

            let img = DynamicImage::ImageLuma8(ImageBuffer::from_raw(w as u32, h as u32, buf).unwrap());

            Ok(Cow::Owned(img))
        })
    }
}



use std::{borrow::Cow, time::Duration};

use birb_vision::{image::{DynamicImage, ImageBuffer, Luma}, AsyncTask, CameraDevice, DeviceError, DeviceResult, Frame};
use crate::prelude::*;

impl CameraDevice for MVSDevice {
    fn open(&mut self) -> AsyncTask<DeviceResult<()>> {
        AsyncTask::new(async move {
            MVSDevice::open(
                self,
                AccessMode::Exclusive,
                None,
            ).map_err(|e| DeviceError::other(e))
        })
    }

    fn close(&mut self) -> AsyncTask<DeviceResult<()>> {
        AsyncTask::new(async move {
            MVSDevice::close(self).map_err(|e| DeviceError::other(e))
        })
    }

    fn start_video_stream(&mut self) -> AsyncTask<DeviceResult<()>> {
        AsyncTask::new(async move {
            self.start_grabbing().map_err(|e| DeviceError::other(e))
        })
    }

    fn stop_video_stream(&mut self) -> AsyncTask<DeviceResult<()>> {
        AsyncTask::new(async move {
            self.stop_grabbing().map_err(|e| DeviceError::other(e))
        })
    }

    fn receive_frame(&mut self) -> AsyncTask<DeviceResult<std::borrow::Cow<'_, Frame>>> {
        AsyncTask::new(async move {
            // TODO HANDLE DIFFERENT PIXEL FORMATS

            let w = self.get_int_value("Width").map_err(|e| DeviceError::other(e))?.current();
            let h = self.get_int_value("Height").map_err(|e| DeviceError::other(e))?.current();
            //let pitch = self.get_int_value("LinePitch").map_err(|e| DeviceError::other(e))?.current();
            //assert_eq!(pitch, w, "LinePitch != Width");

            let mut buf = ImageBuffer::<Luma<u8>, Vec<u8>>::new(w as u32, h as u32).into_raw();

            self.get_one_frame_timeout(&mut buf, Duration::from_secs(1)).map_err(|e| DeviceError::other(e))?;

            let img = DynamicImage::ImageLuma8(ImageBuffer::from_raw(w as u32, h as u32, buf).unwrap());

            Ok(Cow::Owned(Frame::Image(img)))
        })
    }
}



use std::{borrow::Cow, time::Duration};

use birb_vision::{external::async_trait::async_trait, image::{DynamicImage, ImageBuffer, Luma}, CameraDevice, DeviceError, DeviceResult, Frame};
use crate::prelude::*;

#[async_trait(?Send)]
impl CameraDevice for MVSDevice {
    async fn open(&self) -> DeviceResult<()> {
        MVSDevice::open(
            self,
            AccessMode::Exclusive,
            None,
        ).map_err(|e| DeviceError::other(e))
    }

    async fn close(&self) -> DeviceResult<()> {
        MVSDevice::close(self).map_err(|e| DeviceError::other(e))
    }

    async fn start_video_stream(&self) -> DeviceResult<()> {
        self.start_grabbing().map_err(|e| DeviceError::other(e))
    }

    async fn stop_video_stream(&self) -> DeviceResult<()> {
        self.stop_grabbing().map_err(|e| DeviceError::other(e))
    }

    async fn flush(&self) -> DeviceResult<()> {
        log::error!("flush not implemented for MVSDevice");
        Ok(())
    }

    async fn receive_frame(&self) -> DeviceResult<std::borrow::Cow<'_, Frame>> {
        // TODO HANDLE DIFFERENT PIXEL FORMATS

        let w = self.get_int_value("Width").map_err(|e| DeviceError::other(e))?.current();
        let h = self.get_int_value("Height").map_err(|e| DeviceError::other(e))?.current();
        //let pitch = self.get_int_value("LinePitch").map_err(|e| DeviceError::other(e))?.current();
        //assert_eq!(pitch, w, "LinePitch != Width");

        let mut buf = ImageBuffer::<Luma<u8>, Vec<u8>>::new(w as u32, h as u32).into_raw();

        self.get_one_frame_timeout(&mut buf, Duration::from_secs(1)).map_err(|e| DeviceError::other(e))?;

        let img = DynamicImage::ImageLuma8(ImageBuffer::from_raw(w as u32, h as u32, buf).unwrap());

        Ok(Cow::Owned(Frame::Image(img)))
    }
}

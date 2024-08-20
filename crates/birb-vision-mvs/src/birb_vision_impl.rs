

use std::{borrow::Cow, time::Duration};

use birb_vision::{channels::{CallbackHandle, CallbackTx}, image::{DynamicImage, ImageBuffer, Luma}, CameraDevice, DeviceAccessMode, DeviceError, DeviceResult, Event, Frame};
use crate::{mvs_try, prelude::*};

impl CameraDevice for MVDevice {
    fn is_device_accessible(&self, mode: DeviceAccessMode) -> bool {
        todo!()
    }

    fn is_open(&self) -> Option<DeviceAccessMode> {
        todo!()
    }

    fn open(&self, mode: DeviceAccessMode) -> DeviceResult<()> {
        MVDevice::open(
            self,
            AccessMode::Exclusive,
            None,
        ).map_err(|e| DeviceError::other(e))
    }

    fn close(&self) -> DeviceResult<()> {
        MVDevice::close(self).map_err(|e| DeviceError::other(e))
    }

    fn start_grabbing(&self) -> DeviceResult<()> {
        self.start_grabbing().map_err(|e| DeviceError::other(e))
    }

    fn stop_grabbing(&self) -> DeviceResult<()> {
        self.stop_grabbing().map_err(|e| DeviceError::other(e))
    }

    fn flush(&self) -> DeviceResult<()> {
        log::error!("flush not implemented for MVSDevice");
        Ok(())
    }

    //async fn receive_frame(&self) -> DeviceResult<std::borrow::Cow<'_, Frame>> {
    //    // TODO HANDLE DIFFERENT PIXEL FORMATS
//
    //    let w = self.get_int_value("Width").map_err(|e| DeviceError::other(e))?.current();
    //    let h = self.get_int_value("Height").map_err(|e| DeviceError::other(e))?.current();
    //    //let pitch = self.get_int_value("LinePitch").map_err(|e| DeviceError::other(e))?.current();
    //    //assert_eq!(pitch, w, "LinePitch != Width");
//
    //    let mut buf = ImageBuffer::<Luma<u8>, Vec<u8>>::new(w as u32, h as u32).into_raw();
//
    //    self.get_one_frame_timeout(&mut buf, Duration::from_secs(1)).map_err(|e| DeviceError::other(e))?;
//
    //    let img = DynamicImage::ImageLuma8(ImageBuffer::from_raw(w as u32, h as u32, buf).unwrap());
//
    //    Ok(Cow::Owned(Frame::Image(img)))
    //}

    fn grab(&self) -> DeviceResult<()> {
        // TODO this function is deprecated, what should we use instead? Maybe MV_CC_SetCommandValue
        mvs_try!(self.cx => MV_CC_TriggerSoftwareExecute(self.handle)).map_err(|e| DeviceError::other(e))
    }

    fn set_stream_callback(&self, f: Box<dyn Fn(Event) + Send + Sync>) -> DeviceResult<CallbackHandle> {
        let (tx, handle) = CallbackTx::new(f);
        self.register_image_callback(move |img| {
            tx.try_call(|f| f(Event::Frame(Ok(Frame::Image(img)))));
        });
        Ok(handle)
    }
}

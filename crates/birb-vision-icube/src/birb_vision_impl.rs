use std::borrow::Cow;

use anyhow::anyhow;
use birb_vision_core::{backend::DeviceInfo, CameraDevice, DeviceError, DeviceResult, Event, FlatSample, PixelFormat, Sample, SampleType};
use icube_sdk_sys::SDK;

use crate::{iCubeContext, iCubeDevice, CallbackEventType, IntoICubeResult};


impl CameraDevice for iCubeDevice {
    fn get_device_info(&self) -> DeviceResult<DeviceInfo> {
        let mut info = DeviceInfo::new();
        self.get_name()?;
        info.display_name = "iCube".into(); // TODO get from SDK
        todo!()
    }

    fn start_grabbing(&self) -> DeviceResult {
        self.start_video_stream(false, true).map_err(|e| e.into())
    }

    fn stop_grabbing(&self) -> DeviceResult {
        self.stop_video_stream().map_err(|e| e.into())
    }

    fn grab(&self) -> DeviceResult {
        todo!()
    }

    fn set_stream_callback(&self, f: Box<dyn for<'a> Fn(Event<'a>) + Send + Sync>) -> DeviceResult {
        let device_index = self.device_index();
        self.set_callback(Box::new(move |e: CallbackEventType<'_>| {
            let mut width = 0;
            let mut height = 0;

            let size_result = 'a: {
                let ctx = match iCubeContext::new() {
                    Ok(ctx) => ctx,
                    Err(e) => break 'a Err(anyhow!("iCube callback called without teh possibility of getting a context: {e}").into()),
                };
                match ctx.sdk() {
                        SDK::V1(api) => unsafe { (api.GetSize)(device_index as _, &mut width, &mut height).v1_result() },
                        SDK::V2(api) => unsafe { (api.GetSize)(device_index, &mut width, &mut height).v2_result() },
                }.map(|_| (width, height))
            };

            match e {
                CallbackEventType::NEW_FRAME(data) => {
                    let sample: Result<Sample, DeviceError> = size_result
                        .map_err(|e| DeviceError::from(e))
                        .map(|(width, height)| Sample::FlatSample(FlatSample {
                            buffer: Cow::Borrowed(data),
                            offset: 0,
                            sample_type: SampleType::Plain(PixelFormat::RGB8Packed),
                            width: width as _,
                            height: height as _,
                            stride: width as i32 * 3,
                            row_major: true,
                        }));
                    f(Event::Sample(sample));
                },
                _ => todo!("unhandled iCube event type: {e:?}"),
            }

            todo!()
        }));

        Ok(())
    }
}

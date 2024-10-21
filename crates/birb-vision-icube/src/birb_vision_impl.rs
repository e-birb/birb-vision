use std::borrow::Cow;

use anyhow::anyhow;
use birb_vision_core::{backend::{Backend, DeviceInfo, DeviceInfoEntry}, CameraDevice, DeviceError, DeviceResult, Event, FlatSample, PixelFormat, Sample, SampleType};
use icube_sdk_sys::SDK;

use crate::{iCubeContext, iCubeDevice, CallbackEventType, IntoICubeResult};


impl CameraDevice for iCubeDevice {
    fn get_device_info(&self) -> DeviceResult<DeviceInfo> {
        let mut info = DeviceInfo::new();
        info.display_name = self.get_name()?.into(); // TODO get from SDK
        info.other.insert("device_index".into(), DeviceInfoEntry::new("Device Index", self.device_index().to_string()));
        Ok(info)
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
        }));

        Ok(())
    }
}

impl Backend for iCubeContext {
    fn available_transport_layers(&self) -> Vec<String> {
        vec![] // TODO maybe?
    }

    fn enumerate(&self, _transport_layers: &[String]) -> anyhow::Result<Vec<DeviceInfo>> {
        self.init_device_list(|device_indices| {
            let mut devices = vec![];
            for device_index in device_indices {
                let device = device_index.open()?;
                devices.push(device.get_device_info()?);
            }
            Ok(devices)
        })
    }

    fn create(&self, info: &DeviceInfo) -> anyhow::Result<Option<Box<dyn CameraDevice>>> {
        self.init_device_list(|device_indices| {
            for idx in device_indices {
                let matches = if let Some(device_index) = info.other.get("device_index") {
                    device_index.value == idx.sdk_index().to_string()
                } else if info.display_name == idx.name().as_ref().map(|s| s.clone()).map_err(|e| anyhow!("Could not read iCube device name: {e}"))? {
                    true
                } else {
                    false
                };

                if matches {
                    let device = idx.open()?;
                    let device = Box::new(device) as Box<dyn CameraDevice>;
                    return Ok(Some(device));
                }
            }

            Ok(None)
        })
    }
}
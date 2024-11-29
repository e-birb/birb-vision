use std::borrow::Cow;

use anyhow::anyhow;
use birb_vision_core::{context::{VisionContext, DeviceInfo, DeviceInfoEntry}, CameraDevice, DeviceError, DeviceResult, FlatSample, FlatSampleLayout, ImageSampleBuffer, Node, NodeId, PixelFormat, Sample, SampleType, StreamEvent, StringProperty};
use icube_sdk_sys::SDK;

use crate::{iCubeContext, iCubeDevice, CallbackEventType, IntoICubeResult};

/// Properties
mod common_property {
    use std::borrow::Cow;
    use birb_vision_core::NodeId;

    macro_rules! decl {
        ($(
            $name:ident = $id:literal;
        )*) => {
            $(
                pub const $name: NodeId = NodeId::String(Cow::Borrowed(concat!("common-prop--", $id)));
            )*
        };
    }

    decl! {
        NAME = "Name";
        VERSION = "Version";
        FIRMWARE_VERSION = "Firmware Version";
        SERIAL_NUMBER = "Serial Number";
        FPGA_VERSION = "FPGA Version";
        ROI_PROPERTY = "ROI"; // TODO a "compound" property whose value is a set of other properties
        RESOLUTION = "Resolution";
        RESOLUTION_RANGE = "Resolution Range";
        RESOLUTION_MODE = "Resolution Mode";
        BIN_SKIP = "Binning and Skipping";
        TRIGGER_MODE = "Trigger Mode";
        EXPOSURE = "Exposure";
        // TODO auto stuff?

        CAMERA_SPECIFIC_PARAMS = "Camera Specific Parameters"; // TODO this will also contain commands (set_param_one_push)
    }
}

impl CameraDevice for iCubeDevice {
    fn get_device_info(&mut self) -> DeviceResult<DeviceInfo> {
        let mut info = DeviceInfo::new();
        info.display_name = self.get_name()?.into(); // TODO get from SDK
        info.other.insert("device_index".into(), DeviceInfoEntry::new("Device Index", self.device_index().to_string()));
        Ok(info)
    }

    fn all_properties(&mut self) -> DeviceResult<Vec<birb_vision_core::Node>> {
        let mut properties: Vec<Node> = Vec::<Node>::new();

        use common_property::*;

        // name
        properties.push(StringProperty::new_const(
            NAME,
            "Name",
        ).into());

        // version
        properties.push(StringProperty::new_const(
            VERSION,
            "SDK Version",
        ).into());

        // firmware version
        properties.push(StringProperty::new_const(
            FIRMWARE_VERSION,
            "Firmware Version",
        ).into());

        // serial number
        if let SDK::V2(_) = self.handle.ctx.sdk() {
            properties.push(StringProperty::new_const(
                SERIAL_NUMBER,
                "Serial Number",
            ).into());
        }

        // FPGA version
        properties.push(StringProperty::new_const(
            FPGA_VERSION,
            "FPGA Version",
        ).into());

        Ok(properties)
    }

    fn read_property(&mut self, id: &NodeId) -> DeviceResult<birb_vision_core::PropertyState> {
        use common_property::*;

        if id == &NAME {
            let version = self.get_name()?;
            Ok(birb_vision_core::PropertyState::String(version))
        } else if id == &VERSION {
            let version = self.get_version()?;
            Ok(birb_vision_core::PropertyState::String(version))
        } else if id == &FIRMWARE_VERSION {
            let version = self.get_firmware_version()?;
            Ok(birb_vision_core::PropertyState::String(version))
        } else if id == &SERIAL_NUMBER {
            let serial_number = self.get_serial_number()?;
            Ok(birb_vision_core::PropertyState::String(serial_number))
        } else if id == &FPGA_VERSION {
            let version = self.get_fpga_version()?;
            Ok(birb_vision_core::PropertyState::String(version))
        } else {
            Err(DeviceError::NotAccessible)
        }
    }

    fn start_grabbing(&mut self) -> DeviceResult {
        self.start_video_stream(false, true).map_err(|e| e.into())
    }

    fn stop_grabbing(&mut self) -> DeviceResult {
        self.stop_video_stream().map_err(|e| e.into())
    }

    fn grab(&mut self) -> DeviceResult {
        // TODO implement
        Err(DeviceError::NotImplemented)
    }

    fn set_stream_callback(&mut self, f: Box<dyn for<'a> Fn(StreamEvent<'a>) + Send + Sync>) -> DeviceResult {
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
                        .map(|(width, height)| Sample::ImageSample(FlatSample {
                            buffer: ImageSampleBuffer::Cow(Cow::Borrowed(data)),
                            layout: FlatSampleLayout {
                                offset: 0,
                                sample_type: SampleType::Plain(PixelFormat::RGB8Packed),
                                width: width as _,
                                height: height as _,
                                stride: width as i32 * 3,
                                row_major: true,
                            },
                        }));
                    f(StreamEvent::Sample(sample));
                },
                _ => todo!("unhandled iCube event type: {e:?}"),
            }
        }));

        Ok(())
    }
}

impl VisionContext for iCubeContext {
    fn available_transport_layers(&self) -> Vec<String> {
        vec![] // TODO maybe?
    }

    fn enumerate(&self, _transport_layers: &[String]) -> anyhow::Result<Vec<DeviceInfo>> {
        self.init_device_list(|device_indices| {
            let mut devices = vec![];
            for device_index in device_indices {
                let mut device = device_index.open()?;
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
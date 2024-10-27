use std::borrow::Cow;

use anyhow::anyhow;
use birb_vision_core::{backend::{Backend, DeviceInfo, DeviceInfoEntry}, AccessMode, CameraDevice, DeviceError, DeviceResult, Event, FlatSample, FlatSampleLayout, GroupNode, ImageSampleBuffer, Node, NodeVariant, PixelFormat, PropertyVariant, Sample, SampleType};
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
        ROOT = "Root";

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
    fn get_device_info(&self) -> DeviceResult<DeviceInfo> {
        let mut info = DeviceInfo::new();
        info.display_name = self.get_name()?.into(); // TODO get from SDK
        info.other.insert("device_index".into(), DeviceInfoEntry::new("Device Index", self.device_index().to_string()));
        Ok(info)
    }

    fn all_properties(&self) -> DeviceResult<Vec<birb_vision_core::Node>> {
        let mut properties = Vec::new();

        use common_property::*;

        // name
        let mut node = Node::new_with_id(NAME);
        node.display_name = "Name".into();
        node.variant = NodeVariant::Property(PropertyVariant::String(Default::default()));
        node.access_mode = AccessMode::ReadOnly;
        properties.push(node);

        // version
        let mut node = Node::new_with_id(VERSION);
        node.display_name = "SDK Version".into();
        node.variant = NodeVariant::Property(PropertyVariant::String(Default::default()));
        node.access_mode = AccessMode::ReadOnly;
        properties.push(node);

        // firmware version
        let mut node = Node::new_with_id(FIRMWARE_VERSION);
        node.display_name = "Firmware Version".into();
        node.variant = NodeVariant::Property(PropertyVariant::String(Default::default()));
        node.access_mode = AccessMode::ReadOnly;
        properties.push(node);

        // serial number
        if let SDK::V2(_) = self.handle.ctx.sdk() {
            let mut node = Node::new_with_id(SERIAL_NUMBER);
            node.display_name = "Serial Number".into();
            node.variant = NodeVariant::Property(PropertyVariant::String(Default::default()));
            node.access_mode = AccessMode::ReadOnly;
            properties.push(node);
        }

        // FPGA version
        let mut node = Node::new_with_id(FPGA_VERSION);
        node.display_name = "FPGA Version".into();
        node.variant = NodeVariant::Property(PropertyVariant::String(Default::default()));
        node.access_mode = AccessMode::ReadOnly;

        // root node
        let mut node = Node::new_with_id(ROOT);
        node.display_name = "Root".into();
        node.variant = NodeVariant::Group(GroupNode {
            children: properties.iter().map(|n| n.id.clone()).collect(),
        });
        properties.push(node);

        Ok(properties)
    }

    fn root_property(&self) -> DeviceResult<Option<birb_vision_core::NodeId>> {
        Ok(Some(common_property::ROOT))
    }

    fn read_property(&self, node: &Node) -> DeviceResult<birb_vision_core::PropertyState> {
        use common_property::*;

        if node.id == NAME {
            let version = self.get_name()?;
            Ok(birb_vision_core::PropertyState::String(version))
        } else if node.id == VERSION {
            let version = self.get_version()?;
            Ok(birb_vision_core::PropertyState::String(version))
        } else if node.id == FIRMWARE_VERSION {
            let version = self.get_firmware_version()?;
            Ok(birb_vision_core::PropertyState::String(version))
        } else if node.id == SERIAL_NUMBER {
            let serial_number = self.get_serial_number()?;
            Ok(birb_vision_core::PropertyState::String(serial_number))
        } else if node.id == FPGA_VERSION {
            let version = self.get_fpga_version()?;
            Ok(birb_vision_core::PropertyState::String(version))
        } else {
            Err(DeviceError::NotAccessible)
        }
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
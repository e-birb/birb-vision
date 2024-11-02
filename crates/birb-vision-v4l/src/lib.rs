use std::{borrow::Cow, cell::RefCell, collections::HashMap, ops::Deref, path::Path, sync::{Arc, Mutex}, time::Duration};

use birb_vision_core::{anyhow::{self, anyhow}, backend::{Backend, DeviceInfo, DeviceInfoEntry}, CameraDevice, DeviceResult, StreamEvent, FlatSample, FlatSampleLayout, FourCC, GroupNode, Node, NodeId, PropertyState, PropertyValue, Sample, ImageSampleBuffer, SampleType};
use v4l::{io::traits::CaptureStream, video::Capture, Control, Device};

use birb_vision_core::DeviceError::*;
mod control_compat;

pub struct V4lDevice {
    info: Arc<DeviceInfo>,
    dev: Mutex<Device>,
    current_format: Arc<Mutex<v4l::Format>>,

    callback: Arc<Mutex<Box<dyn Fn(StreamEvent) + Send + Sync>>>,
    thread: RefCell<Option<std::thread::JoinHandle<()>>>,
    stream: RefCell<Option<Arc<Mutex<v4l::io::mmap::Stream<'static>>>>>,

    // TODO I'm not sure if Device is actually Send and Sync
    // (even though `v4l::Device` struct is). I'm just going to
    // assume it is not for now, better safe than sorry.
    _marker: *mut (),

    properties: HashMap<NodeId, Node>,
}

impl V4lDevice {
    pub fn from_path(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let node = v4l::context::enum_devices()
            .into_iter()
            .find(|node| node.path() == path.as_ref());

        let info = node_to_info(&node.unwrap());

        let dev = Device::with_path(path)?;

        let format = dev.format().unwrap();
        //dev.set_format(&Format::new(format.width, format.height, V4lFourCC::new(b"YUYV"))).unwrap();

        let mut root: Node = GroupNode::new("birb-vision-v4l::Root").into();
        root.display_name = "V4L2".into();
        let mut root_children = Vec::new();

        let mut properties = HashMap::new();

        for control in dev.query_controls().unwrap() {
            let Some(node) = control_compat::parse(control) else {
                continue;
            };
            root_children.push(node.id.clone());
            let prev = properties.insert(node.id.clone(), node);
            if let Some(_) = prev {
                // TODO handle error
                todo!();
            }
        }

        Ok(Self {
            info: Arc::new(info),
            dev: Mutex::new(dev),
            current_format: Arc::new(Mutex::new(format)),
            thread: RefCell::new(None),
            callback: Arc::new(Mutex::new(Box::new(|_| {}))),
            stream: RefCell::new(None),
            _marker: std::ptr::null_mut(),
            properties,
        })
    }
}

impl CameraDevice for V4lDevice {
    fn get_device_info(&self) -> DeviceResult<DeviceInfo> {
        Ok(self.info.deref().clone())
    }

    fn all_properties(&self) -> DeviceResult<Vec<Node>> {
        Ok(self
            .properties
            .iter()
            .map(|(_, v)| v.clone())
            .collect()
        )
    }

    fn read_property(&self, node: &Node) -> DeviceResult<PropertyState> {
        let id = &node.id; // TODO since now we take node as a parameter, the V4lDevice::properties machinery is not necessary anymore
        let node = self.properties.get(id).ok_or(anyhow!("Control {id:?} not found"))?;

        // read the value from the device
        let value = self.dev
            .lock().unwrap()
            .control(*id.as_i32().ok_or(InvalidNodeId)? as _)?
            .value;

        control_compat::node_value_to_property_state(node, value)
    }

    fn write_property(&self, node: &Node, value: PropertyValue) -> DeviceResult {
        let id = &node.id; // TODO since now we take node as a parameter, the V4lDevice::properties machinery is not necessary anymore
        self.dev.lock().unwrap().set_control(Control {
            id: *id.as_i32().ok_or(InvalidNodeId)? as _,
            value: control_compat::property_value_to_v4l(value)?,
        })?;

        Ok(())
    }

    fn start_grabbing(&self) -> DeviceResult {
        if self.stream.borrow().is_some() { // TODO also check thread, maybe wait if necessary (only one none)?
            return Ok(());
        }

        let mut dev = self.dev.lock().unwrap();

        let mut s = v4l::io::mmap::Stream::with_buffers(&mut dev, v4l::buffer::Type::VideoCapture, 4)?;
        s.set_timeout(Duration::from_secs(2));

        

        let stream = Arc::new(Mutex::new(s));

        let j = std::thread::spawn({
            let stream = Arc::downgrade(&stream);
            let callback = self.callback.clone();
            let format = self.current_format.clone();
            move || while let Some(stream) = stream.upgrade() {
                let mut stream = stream.lock().unwrap();
                let Ok((data, meta)) = stream.next() else {
                    break;
                };
                let format = format.lock().unwrap().clone();

                // TODO use the stride!!!
                let layout = FlatSampleLayout {
                    offset: 0,
                    sample_type: SampleType::FourCC(FourCC(format.fourcc.repr)),
                    width: format.width,
                    height: format.height,
                    stride: format.stride as _,
                    row_major: true,
                };
                let sample = FlatSample {
                    buffer: ImageSampleBuffer::Cow(Cow::Borrowed(data)),
                    layout,
                };
                let event = StreamEvent::Sample(Ok(Sample::ImageSample(sample)));
                callback.lock().unwrap()(event);
            }
        });

        self.stream.replace(Some(stream));
        self.thread.replace(Some(j));

        Ok(())
    }
    fn stop_grabbing(&self) -> DeviceResult {
        self.stream.replace(None);
        if let Some(j) = self.thread.replace(None) {
            j.join().unwrap();
        }
        Ok(())
    }

    fn grab(&self) -> DeviceResult<()> {
        // TODO
        Ok(())
    }

    fn flush(&self) -> DeviceResult {
        // TODO
        Ok(())
    }

    fn set_stream_callback(&self, f: Box<dyn Fn(StreamEvent) + Send + Sync>) -> DeviceResult {
        *self.callback.lock().unwrap() = f;
        Ok(())
    }
}

pub struct V4lContext;

impl V4lContext {
    pub fn new() -> Self {
        Self
    }
}

impl Backend for V4lContext {
    fn available_transport_layers(&self) -> Vec<String> {
        vec![]
    }

    fn enumerate(&self, _transport_layers: &[String]) -> anyhow::Result<Vec<DeviceInfo>> {
        Ok(
            v4l::context::enum_devices()
                .into_iter()
                .filter(|node| Device::with_path(node.path()).map(|dev| dev.format().is_ok()).unwrap_or(false))
                .map(|node| node_to_info(&node))
                .collect()
            )
    }

    fn create(&self, info: &DeviceInfo) -> anyhow::Result<Option<Box<dyn CameraDevice>>> {
        for node in v4l::context::enum_devices() {
            if node.path().to_string_lossy() == info.other.get("path").unwrap().value.as_str() && node.name() == Some(info.display_name.to_string()) {
                return Ok(Some(Box::new(V4lDevice::from_path(node.path())?)));
            }
        }
        Ok(None)
    }
}

fn node_to_info(node: &v4l::context::Node) -> DeviceInfo {
    let path = node.path().to_string_lossy().to_string();
    let mut info = DeviceInfo::new();
    info.display_name = path.clone();
    info.other.insert("path".into(), DeviceInfoEntry::new("Path", path));
    if let Some(display_name) = node.name() {
        info.display_name = display_name.to_string();
    }
    info
}
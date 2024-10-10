use std::{cell::RefCell, collections::HashMap, error::Error, ops::Deref, path::Path, sync::{Arc, Mutex}, time::{Duration, Instant}};

use birb_vision_core::{backend::{Backend, DeviceInfo, DeviceInfoEntry}, decoders::yuyv422_to_rgb, image::{DynamicImage, RgbImage}, CameraDevice, DeviceAccessMode, DeviceError, DeviceResult, EnumValue, Event, Node, NodeId, NodeVariant, NumericValue, PropertyState, PropertyValue, PropertyVariant, Sample};
use v4l::{control::Value, io::traits::CaptureStream, video::Capture, Control, Device, FourCC};

use birb_vision_core::DeviceError::*;
mod control_compat;

pub struct V4lDevice {
    info: Arc<DeviceInfo>,
    dev: Mutex<Device>,
    current_format: Arc<Mutex<v4l::Format>>,

    callback: Arc<Mutex<Box<dyn Fn(Event) + Send + Sync>>>,
    thread: RefCell<Option<std::thread::JoinHandle<()>>>,
    stream: RefCell<Option<Arc<Mutex<v4l::io::mmap::Stream<'static>>>>>,

    // TODO I'm not sure if Device is actually Send and Sync
    // (even though `v4l::Device` struct is). I'm just going to
    // assume it is not for now, better safe than sorry.
    _marker: *mut (),

    properties: HashMap<NodeId, Node>,
    root_property: NodeId,
}

impl V4lDevice {
    pub fn from_path(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let node = v4l::context::enum_devices()
            .into_iter()
            .find(|node| node.path() == path.as_ref());

        let info = node_to_info(&node.unwrap());

        let dev = Device::with_path(path)?;

        let format = dev.format().unwrap();
        //dev.set_format(&Format::new(format.width, format.height, FourCC::new(b"YUYV"))).unwrap();

        let mut properties = HashMap::new();

        for control in dev.query_controls().unwrap() {
            let Some(node) = control_compat::parse(control) else {
                continue;
            };
            let prev = properties.insert(node.id.clone(), node);
            if let Some(_) = prev {
                // TODO handle error
                todo!();
            }
        }

        let mut root = Node::new_with_id("birb-vision-v4l::Root");
        root.display_name = "V4L2".into();
        root.variant.as_group_mut().expect("root was not a group").children = properties.keys().cloned().collect::<Vec<_>>().into();
        let root_id = root.id.clone();
        properties.insert(root_id.clone(), root);

        Ok(Self {
            info: Arc::new(info),
            dev: Mutex::new(dev),
            current_format: Arc::new(Mutex::new(format)),
            thread: RefCell::new(None),
            callback: Arc::new(Mutex::new(Box::new(|_| {}))),
            stream: RefCell::new(None),
            _marker: std::ptr::null_mut(),
            properties,
            root_property: root_id,
        })
    }
}

impl CameraDevice for V4lDevice {
    fn get_device_info(&self) -> DeviceResult<DeviceInfo> {
        Ok(self.info.deref().clone())
    }

    fn property(&self, id: &NodeId) -> DeviceResult<Node> {
        self.properties.get(id).cloned().ok_or(DeviceError::InvalidNodeId)
    }

    fn root_property(&self) -> DeviceResult<NodeId> {
        Ok(self.root_property.clone())
    }

    fn get_property(&self, id: &NodeId) -> DeviceResult<PropertyState> {
        let control = self.dev.lock().unwrap().control(*id.as_i32().ok_or(InvalidNodeId)? as _)?;

        let node = self.properties.get(id).expect(&format!("Control {id:?} not found"));

        match control.value {
            Value::None => todo!(),
            Value::Integer(current) => match &node.variant {
                NodeVariant::Property(PropertyVariant::Integer(property)) => Ok(PropertyState::Int(NumericValue {
                    current,
                    range: (property.min.unwrap_or(0))..=(property.max.unwrap_or(0)),
                })),
                NodeVariant::Property(PropertyVariant::Enum(property)) => {
                    Ok(PropertyState::Enum(EnumValue {
                        current,
                        support: property.entries.iter().map(|e| e.discriminant).collect::<Vec<_>>().into(),
                    }))
                },
                _ => todo!(),
            },
            Value::Boolean(current) => Ok(PropertyState::Bool(current)),
            Value::String(current) => Ok(PropertyState::String(current)),
            _ => todo!(),
        }
    }

    fn set_property(&self, id: &NodeId, value: birb_vision_core::PropertyValue) -> DeviceResult {
        let dev = self.dev.lock().unwrap(); // TODO handle error
        let value = match value {
            PropertyValue::Bool(value) => Value::Boolean(value),
            PropertyValue::Int(value) => Value::Integer(value),
            PropertyValue::Float(_) => todo!(), // maybe unsupported?
            PropertyValue::Enum(value) => Value::Integer(value),
            PropertyValue::String(value) => Value::String(value.clone()),
            PropertyValue::Command => Value::None,
        };

        dev.set_control(Control {
            id: *id.as_i32().ok_or(InvalidNodeId)? as _,
            value,
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

        use birb_vision_core::image;

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
                let image: DeviceResult<DynamicImage> = if format.fourcc == FourCC::new(b"YUYV") {
                    let start = Instant::now();
                    let data = yuyv422_to_rgb(data, false).unwrap();
                    println!("Converted in {:?}", start.elapsed());
                    let img = DynamicImage::ImageRgb8(RgbImage::from_raw(format.width as u32, format.height as u32, data).unwrap());
                    Ok(img)
                } else if format.fourcc == FourCC::new(b"RGB3") {
                    let img = DynamicImage::ImageRgb8(RgbImage::from_raw(format.width as u32, format.height as u32, data.to_vec()).unwrap());
                    Ok(img)
                } else if format.fourcc == FourCC::new(b"MJPG") {
                    let start = Instant::now();
                    //let img = birb_vision_core::decoders::decode_mjpg(data).unwrap();
                    //let img = DynamicImage::ImageRgb8(img);
                    let img = image::load_from_memory(&data).unwrap();
                    println!("Converted mjpeg in {:?}", start.elapsed());
                    Ok(img)
                } else {
                    log::error!("Unsupported format: {}", format.fourcc);
                    Err(UnsupportedFormat)
                };
                drop(stream);
                let event = Event::Frame(image.map(Sample::Image));
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

    fn set_stream_callback(&self, f: Box<dyn Fn(Event) + Send + Sync>) -> DeviceResult {
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

    fn enumerate(&self, _transport_layers: &[String]) -> Result<Vec<DeviceInfo>, Box<dyn Error>> {
        Ok(
            v4l::context::enum_devices()
                .into_iter()
                .filter(|node| Device::with_path(node.path()).map(|dev| dev.format().is_ok()).unwrap_or(false))
                .map(|node| node_to_info(&node))
                .collect()
            )
    }

    fn create(&self, info: &DeviceInfo) -> Result<Option<Box<dyn CameraDevice>>, Box<dyn Error>> {
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
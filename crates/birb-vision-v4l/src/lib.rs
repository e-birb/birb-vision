use std::{cell::RefCell, collections::HashMap, error::Error, path::Path, sync::{Arc, Mutex}, time::{Duration, Instant}};

use birb_vision_core::{backend::{Backend, DeviceInfo, DeviceInfoEntry}, decoders::yuyv422_to_rgb, image::{DynamicImage, RgbImage}, BoolProperty, CameraDevice, Child, DeviceAccessMode, DeviceResult, EnumEntry, EnumProperty, EnumValue, Event, Frame, GroupNode, Node, NodeId, NodeVariant, NumericProperty, NumericValue, PropertyVariant, Representation, StringProperty};
use v4l::{control::{MenuItem, Value}, io::traits::CaptureStream, video::Capture, Control, Device, Format, FourCC};

use birb_vision_core::DeviceError::*;

pub struct V4lDevice {
    dev: Mutex<Device>,
    controls: RefCell<HashMap<NodeId, v4l::control::Description>>,
    current_format: Arc<Mutex<v4l::Format>>,

    callback: Arc<Mutex<Box<dyn Fn(Event) + Send + Sync>>>,
    thread: RefCell<Option<std::thread::JoinHandle<()>>>,
    stream: RefCell<Option<Arc<Mutex<v4l::io::mmap::Stream<'static>>>>>,

    // TODO I'm not sure if Device is actually Send and Sync
    // (even though `v4l::Device` struct is). I'm just going to
    // assume it is not for now, better safe than sorry.
    _marker: *mut (),
}

impl V4lDevice {
    pub fn from_v4l_device(dev: Device) -> Self {
        let format = dev.format().unwrap();
        //dev.set_format(&Format::new(format.width, format.height, FourCC::new(b"YUYV"))).unwrap();

        Self {
            dev: Mutex::new(dev),
            controls: RefCell::new(HashMap::new()),
            current_format: Arc::new(Mutex::new(format)),
            thread: RefCell::new(None),
            callback: Arc::new(Mutex::new(Box::new(|_| {}))),
            stream: RefCell::new(None),
            _marker: std::ptr::null_mut(),
        }
    }

    pub fn from_path(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let dev = Device::with_path(path)?;
        Ok(Self::from_v4l_device(dev))
    }
}

impl CameraDevice for V4lDevice {
    fn is_device_accessible(&self, mode: DeviceAccessMode) -> bool {
        // TODO
        true
    }

    fn is_open(&self) -> Option<DeviceAccessMode> {
        // TODO
        None
    }

    fn open(&self, mode: DeviceAccessMode) -> DeviceResult {
        // TODO maybe no-op
        Ok(())
    }

    fn close(&self) -> DeviceResult {
        // TODO maybe no-op
        Ok(())
    }

    fn control_description(&self) -> DeviceResult<birb_vision_core::Node> {
        use v4l::control::Type;

        let controls = self.dev.lock().unwrap().query_controls().unwrap();
        let mut nodes = Vec::<Node>::new();
        for control in controls {
            let value = self.dev.lock().unwrap().control(control.id);
            let mut node: Node = Node::new_with_id(control.id as i32);
            node.display_name = control.name.clone().into();
            let variant: Option<NodeVariant> = match control.typ {
                Type::Integer | Type::Integer64 => {
                    let mut variant = NumericProperty::<i64>::default();
                    variant.min = control.minimum.into();
                    variant.max = control.maximum.into();
                    variant.default = control.default.into();
                    variant.increment = (control.step as i64).into();
                    variant.representation = Some(if control.name.to_lowercase().starts_with("exposure") {
                        Representation::Logarithmic
                    } else {
                        Representation::Linear
                    });
                    Some(PropertyVariant::Integer(variant).into())
                },
                Type::Boolean => {
                    let mut variant = BoolProperty::default();
                    variant.default = Some(control.default != 0);
                    Some(PropertyVariant::Boolean(variant).into())
                },
                Type::Menu => {
                    let mut variant = EnumProperty::default();
                    variant.entries = control
                        .items
                        .as_ref().unwrap()
                        .iter()
                        .map(|(id, item)| {
                            let MenuItem::Name(name) = item else {
                                panic!("Expected name");
                            };
                            EnumEntry {
                                discriminant: *id as i64,
                                name: name.clone().into(),
                                help: None,
                            }
                        })
                        .collect::<Vec<_>>()
                        .into();
                    Some(PropertyVariant::Enum(variant).into())
                },
                Type::Button => {
                    Some(PropertyVariant::Command.into())
                },
                Type::CtrlClass => {
                    // TODO
                    None
                },
                Type::String => {
                    let variant = StringProperty::default();
                    Some(PropertyVariant::String(variant).into())
                },
                Type::Bitmask => {
                    // TODO
                    None
                },
                Type::IntegerMenu => {
                    // TODO
                    None
                },
                Type::U8 | Type::U16 | Type::U32 => {
                    // TODO
                    None
                },
                Type::Area => {
                    // TODO
                    None
                },
            };
            if let Some(variant) = variant {
                node.variant = variant;
                self.controls.borrow_mut().insert((control.id as i32).into(), control);
                nodes.push(node);
            }
        }

        let children = nodes
            .into_iter()
            .map(|n| Child::Node(n))
            .collect::<Vec<Child>>();

        let mut root = Node::new("Controls");
        root.id = Some("root CONTROLS smdkwmfpeewopfmewpfmwepompwre".into());
        root.variant = NodeVariant::Group(GroupNode {
            children: children.into(),
        });

        Ok(root)
    }

    fn properties(&self) -> DeviceResult<Node> {
        self.control_description()
    }

    fn get_bool_property(&self, id: &NodeId) -> DeviceResult<bool> {
        let control = self.dev.lock().unwrap().control(*id.as_i32().ok_or(InvalidNodeId)? as _).unwrap();
        let Value::Boolean(value) = control.value else {
            panic!("Expected boolean value");
        };
        Ok(value)
    }
    fn get_int_property(&self, id: &NodeId) -> DeviceResult<NumericValue<i64>> {
        let controls = self.controls.borrow();
        let desc = controls.get(id).expect(&format!("Control {id:?} not found"));
        let control = self.dev.lock().unwrap().control(*id.as_i32().ok_or(InvalidNodeId)? as _).unwrap();
        let Value::Integer(value) = control.value else {
            panic!("Expected integer value");
        };
        Ok(NumericValue {
            current: value,
            range: (desc.minimum as i64)..=(desc.maximum as i64),
        })
    }
    fn get_float_property(&self, id: &NodeId) -> DeviceResult<NumericValue<f64>> {
        let controls = self.controls.borrow();
        let desc = controls.get(id).unwrap();
        let control = self.dev.lock().unwrap().control(*id.as_i32().ok_or(InvalidNodeId)? as _).unwrap();
        let Value::Integer(value) = control.value else {
            panic!("Expected integer value");
        };
        Ok(NumericValue {
            current: value as f64,
            range: (desc.minimum as f64)..=(desc.maximum as f64),
        })
    }
    fn get_enum_property(&self, id: &NodeId) -> DeviceResult<EnumValue> {
        let control = self.dev.lock().unwrap().control(*id.as_i32().ok_or(InvalidNodeId)? as _).unwrap();
        let Value::Integer(value) = control.value else {
            panic!("Expected integer value");
        };
        let controls = self.controls.borrow();
        let desc = controls.get(id).unwrap();
        let entries = desc.items.as_ref().unwrap().iter().map(|(id, item)| {
            let MenuItem::Name(name) = item else {
                panic!("Expected name");
            };
            EnumEntry {
                discriminant: *id as i64,
                name: name.clone().into(),
                help: None,
            }
        }).collect::<Vec<_>>();
        Ok(EnumValue {
            current: value,
            support: entries.iter().map(|e| e.discriminant).collect::<Vec<_>>().into(),
        })
    }
    fn get_string_property(&self, id: &NodeId) -> DeviceResult<String> {
        let control = self.dev.lock().unwrap().control(*id.as_i32().ok_or(InvalidNodeId)? as _).unwrap();
        let Value::String(value) = control.value else {
            panic!("Expected string value");
        };
        Ok(value)
    }

    fn set_property(&self, id: &NodeId, value: &birb_vision_core::PropertyValue) -> DeviceResult {
        todo!()
    }

    fn set_bool_property(&self, id: &NodeId, value: bool) -> DeviceResult {
        self.dev.lock().unwrap().set_control(Control { id: *id.as_i32().ok_or(InvalidNodeId)? as _, value: Value::Boolean(value) })?;
        Ok(())
    }
    fn set_int_property(&self, id: &NodeId, value: i64) -> DeviceResult {
        self.dev.lock().unwrap().set_control(Control { id: *id.as_i32().ok_or(InvalidNodeId)? as _, value: Value::Integer(value) })?;
        Ok(())
    }
    fn set_float_property(&self, id: &NodeId, value: f64) -> DeviceResult {
        // TODO error
        unimplemented!()
    }
    fn set_enum_property(&self, id: &NodeId, value: i64) -> DeviceResult {
        self.set_int_property(id, value)
    }
    fn set_string_property(&self, id: &NodeId, value: &str) -> DeviceResult {
        self.dev.lock().unwrap().set_control(Control { id: *id.as_i32().ok_or(InvalidNodeId)? as _, value: Value::String(value.to_string()) }).unwrap();
        Ok(())
    }
    fn send_command(&self, id: &NodeId) -> DeviceResult {
        self.dev.lock().unwrap().set_control(Control { id: *id.as_i32().ok_or(InvalidNodeId)? as _, value: Value::None }).unwrap();
        Ok(())
    }

    fn start_grabbing(&self) -> DeviceResult {
        if self.stream.borrow().is_some() { // TODO also check thread, maybe wait if necessary (only one none)?
            return Ok(());
        }

        let mut dev = self.dev.lock().unwrap();

        let mut s = v4l::io::mmap::Stream::with_buffers(&mut dev, v4l::buffer::Type::VideoCapture, 4).unwrap();
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
                let image: Option<DynamicImage> = if format.fourcc == FourCC::new(b"YUYV") {
                    let start = Instant::now();
                    let data = yuyv422_to_rgb(data, false).unwrap();
                    println!("Converted in {:?}", start.elapsed());
                    let img = DynamicImage::ImageRgb8(RgbImage::from_raw(format.width as u32, format.height as u32, data).unwrap());
                    Some(img)
                } else if format.fourcc == FourCC::new(b"RGB3") {
                    let img = DynamicImage::ImageRgb8(RgbImage::from_raw(format.width as u32, format.height as u32, data.to_vec()).unwrap());
                    Some(img)
                } else if format.fourcc == FourCC::new(b"MJPG") {
                    let start = Instant::now();
                    //let img = birb_vision_core::decoders::decode_mjpg(data).unwrap();
                    //let img = DynamicImage::ImageRgb8(img);
                    let img = image::load_from_memory(&data).unwrap();
                    println!("Converted mjpeg in {:?}", start.elapsed());
                    Some(img)
                } else {
                    panic!("Unsupported format: {}", format.fourcc);
                    None
                };
                drop(stream);
                if let Some(image) = image {
                    let event = Event::Frame(Ok(Frame::Image(image)));
                    callback.lock().unwrap()(event);
                }
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
    fn enumerate(&self) -> Result<Vec<DeviceInfo>, Box<dyn Error>> {
        Ok(
            v4l::context::enum_devices()
                .into_iter()
                .filter(|node| Device::with_path(node.path()).map(|dev| dev.format().is_ok()).unwrap_or(false))
                .map(node_to_info)
                .collect()
            )
    }

    fn find(&self, info: &DeviceInfo) -> Result<Vec<DeviceInfo>, Box<dyn Error>> {
        todo!()
    }

    fn create(&self, info: &DeviceInfo) -> Result<Option<Box<dyn CameraDevice>>, Box<dyn Error>> {
        for node in v4l::context::enum_devices() {
            if node.path().to_string_lossy() == info.other.get("path").unwrap().value.as_str() && node.name() == Some(info.display_name.to_string()) {
                let dev = Device::with_path(node.path())?;
                return Ok(Some(Box::new(V4lDevice::from_v4l_device(dev))));
            }
        }
        Ok(None)
    }
}

fn node_to_info(node: v4l::context::Node) -> DeviceInfo {
    let path = node.path().to_string_lossy().to_string();
    let mut info = DeviceInfo::new();
    info.display_name = path.clone();
    info.other.insert("path".into(), DeviceInfoEntry::new("Path", path));
    if let Some(display_name) = node.name() {
        info.display_name = display_name.to_string();
    }
    info
}
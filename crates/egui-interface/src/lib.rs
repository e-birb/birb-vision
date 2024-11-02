use std::{collections::{HashMap, HashSet}, ops::{Deref, DerefMut}, sync::{mpsc::Sender, Arc, Weak}, time::Instant};

use birb_vision::core::{backend::DeviceInfo, image::DynamicImage, AccessMode, CameraDevice, EnumEntry, Node, NodeId, Property as BirbProperty, PropertyState, PropertyValue, Representation, StreamEvent, ValueOrRef};
use egui::{load::SizedTexture, mutex::Mutex, Color32, ColorImage, DragValue, FontData, FontDefinitions, FontFamily, Grid, Image, ImageData, RichText, ScrollArea, Slider, TextBuffer, TextureFilter, TextureHandle, TextureOptions, Ui, Window};
use material_icons::Icon;
use regex::Regex;

mod selector;

pub use selector::CameraManager;

/// UI providing control over a camera.
pub struct CameraControl {
    state: Option<Arc<Mutex<State>>>,
    controls_window: bool,
    zoom: f32,
    fps: f32,
}

/// The state that is shared with the camera control thread.
pub struct State {
    device_info: Option<Arc<DeviceInfo>>,
    props: Option<Properties>,
    filter: String,
    filter_error: String,
    selected: HashSet<NodeId>,
    updated: HashSet<NodeId>,
    image: Option<ImageData>,
    texture_handle: Option<TextureHandle>,
    thread: Option<std::thread::JoinHandle<()>>,
    update: Box<dyn Fn() + Send + Sync>,
    tx: Sender<Command>,
    last_frame: Instant,
}

enum Command {
    Write,
    StartGrabbing,
    StopGrabbing,
}

impl State {
    fn filter_re(&mut self) -> Option<Regex> {
        if self.filter.is_empty() {
            return None;
        }

        let r = Regex::new(&self.filter.to_lowercase()); // TODO lowercase as an option
        match r {
            Ok(re) => Some(re),
            Err(e) => {
                self.filter_error = format!("{e}");
                None
            }
        }
    }
}

impl State {
    fn new() -> Self {
        State {
            device_info: None,
            props: None,
            filter: String::new(),
            filter_error: String::new(),
            selected: Default::default(),
            updated: HashSet::new(),
            image: None,
            texture_handle: None,
            thread: None,
            update: Box::new(|| {}),
            tx: std::sync::mpsc::channel().0, // TODO shit
            last_frame: Instant::now(),
        }
    }
}

#[derive(Clone)]
struct StateRef(Weak<Mutex<State>>);

impl StateRef {
    fn on_state<R>(&self, f: impl FnOnce(&State) -> R) -> Option<R> {
        self.0.upgrade().map(|state| {
            let state = state.lock();
            let r = f(&state);
            r
        })
    }

    fn on_state_mut<R>(&self, f: impl FnOnce(&mut State) -> R) -> Option<R> {
        self.0.upgrade().map(|state| {
            let mut state = state.lock();
            let r = f(&mut state);
            (state.update)();
            r
        })
    }
}

impl CameraControl {
    pub fn new() -> Self {
        CameraControl {
            state: None,
            controls_window: false,
            zoom: 1.0,
            fps: 0.0,
        }
    }

    pub fn current_camera_info(&self) -> Option<Arc<DeviceInfo>> {
        self.state.as_ref()?.lock().device_info.clone()
    }

    pub fn init(
        &mut self,
        init: impl FnOnce() -> Box<dyn CameraDevice> + Send + 'static,
    ) {
        let mut state = State::new();
        let (tx, rx) = std::sync::mpsc::channel();
        state.tx = tx;
        let state = Arc::new(Mutex::new(state));

        let thread = std::thread::spawn({
            let state = StateRef(Arc::downgrade(&state));

            move || {
                let camera = init();
                let info = camera.get_device_info().unwrap();
                state.on_state_mut(|state| {
                    state.device_info = Some(Arc::new(info));
                });
                //camera.start_grabbing().unwrap();
                camera.set_stream_callback({
                    let state = state.clone();
                    Box::new(move |e| {
                        match e {
                            StreamEvent::Sample(sample) => {
                                let Ok(sample) = sample else {
                                    return;
                                };
                                let img: DynamicImage = match sample.try_decode() {
                                    Ok(result) => match result {
                                        Ok(img) => img,
                                        Err(e) => todo!("could not decode sample: {e}"),
                                    },
                                    Err(sample) => todo!("could not convert sample {sample:?} to image"),
                                };
                                let img = img.to_rgb8();
                                let img = ColorImage::from_rgb([img.width() as usize, img.height() as usize], &img.into_raw());
                                state.on_state_mut(move |s| {
                                    s.image = Some(img.into());
                                });
                                //println!("Sent in {:?}", start.elapsed());
                            },
                            _ => {

                            },
                        }
                    })
                }).unwrap();

                let mut properties = Properties::parse(&*camera);
                properties.update_all_nodes(&*camera);

                state.on_state_mut(|state| {
                    let re = state.filter_re();
                    let mut selected_nodes = HashSet::new();
                    if let Some(root_id) = &properties.root {
                        let (_, root) = properties.leafs.get(root_id).unwrap();
                        root.filter(&properties, &mut selected_nodes, &re);
                        // unnecessary? selected_nodes.insert(root_id.clone());
                    }

                    state.props = properties.into();
                    state.selected = selected_nodes;
                });

                loop {
                    std::thread::yield_now();
                    let Ok(command) = rx.recv() else {
                        break;
                    };
                    match command {
                        Command::Write => {
                            if state.on_state_mut(|state| {
                                let props = state.props.as_mut().unwrap();
                                props.write_all_nodes(&*camera);
                            }).is_none() {
                                break;
                            }
                        },
                        Command::StartGrabbing => {
                            camera.start_grabbing().unwrap();
                        },
                        Command::StopGrabbing => {
                            camera.stop_grabbing().unwrap();
                        }
                    }
                }
            }
        });

        state.lock().thread = Some(thread);

        self.state = Some(state.clone());
    }

    fn show_view(&mut self, ui: &mut egui::Ui) {
        let Some(state) = self.state.as_ref() else {
            ui.centered_and_justified(|ui| {
                ui.label("No camera");
            });
            return;
        };
        let mut state = state.lock();
        let tx = state.tx.clone();
        state.update = Box::new({
            let cx = ui.ctx().clone();
            move || cx.request_repaint()
        });

        if let Some(img) = state.image.take() {
            self.fps = 1.0 / state.last_frame.elapsed().as_secs_f32();
            state.last_frame = Instant::now();
            let mut options = TextureOptions::default();
            options.magnification = TextureFilter::Nearest;
            state.texture_handle = ui.ctx().load_texture("frame", img, options).into();
        }

        let mut fit_to_viewport = false;

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                if let Some(info) = &state.device_info {
                    ui.label(&info.display_name);
                }
            });
            ui.horizontal(|ui| {
                if ui.button(icon(Icon::PlayArrow))
                    .on_hover_text("Start Grabbing")
                    .clicked() {
                    tx.send(Command::StartGrabbing).unwrap();
                }
                if ui.button(icon(Icon::Camera))
                    .on_hover_text("Grab Frame")
                    .clicked() {
                    // TODO
                }
                if ui.button(icon(Icon::Stop))
                    .on_hover_text("Stop Grabbing")
                    .clicked() {
                    tx.send(Command::StopGrabbing).unwrap();
                }
                if ui.button(icon(Icon::Tune))
                    .on_hover_text("Controls")
                    .clicked() {
                    self.controls_window = true;
                }
                ui.add(Slider::new(&mut self.zoom, 0.1..=5.0).logarithmic(true).text("Zoom"));
                if ui
                    .button(icon(Icon::ZoomOutMap))
                    .on_hover_text("Fit to viewport")
                    .clicked() {
                    fit_to_viewport = true;
                }
                ui.label(format!("fps: {:.2}", self.fps));
            });
            ui.separator();
            if let Some(texture_handle) = state.texture_handle.as_ref() {
                let texture = SizedTexture::from_handle(texture_handle);
                if fit_to_viewport {
                    let available = ui.available_size();
                    let zoom_x = available.x / texture.size.x;
                    let zoom_y = available.y / texture.size.y;
                    self.zoom = zoom_x.min(zoom_y);
                }
                //let available = ui.available_size();
                // OLD:
                // let image = Image::new(texture)
                //     .max_width(available.x)
                //     .max_height(available.y)
                //     .shrink_to_fit()
                //     .maintain_aspect_ratio(true)
                //     .fit_to_original_size(4.0);
                // ui.add(image);
                //let (rect, response) = ui.allocate_exact_size(available, Sense::drag());
                //let p = ui.painter();
                //p.image(texture.id, rect, self.zoom, Color32::WHITE);
                ScrollArea::both().show(ui, |ui| {
                    let image = Image::new(texture)
                        .fit_to_exact_size(texture.size * self.zoom);
                    ui.add(image);
                });
            }
        });
    }

    fn controls_window(&mut self, ctx: &egui::Context) {
        if !self.controls_window {
            return;
        }

        if self.controls_window {
            Window::new("Controls") // TODO camera name for ID
                .open(&mut self.controls_window)
                .min_width(300.0)
                .min_height(300.0)
                .show(ctx, |ui| {
                    let Some(state) = self.state.as_ref() else { return; };
                    let mut state = state.lock();
                    let tx = state.tx.clone();
                    state.update = Box::new({
                        let cx = ui.ctx().clone();
                        move || cx.request_repaint()
                    });

                    ui.vertical(|ui| {
                        ui.set_max_width(300.0);
                        ui.horizontal(|ui| {
                            ui.label(icon(Icon::FilterList)).on_hover_ui(|ui| {
                                ui.label("Filter nodes by name using a regex. Examples:");
                                ui.code("^Exposure");
                            });
                            if ui.text_edit_singleline(&mut state.filter).changed() {
                                let re = state.filter_re();
                                if let Some(props) = &state.props {
                                    let mut selected = HashSet::new();
                                    if let Some(root_id) = &props.root {
                                        let (_, root) = props.leafs.get(root_id).unwrap();
                                        root.filter(&props, &mut selected, &re);
                                        state.selected = selected;
                                    }
                                }
                            }
                            if !state.filter_error.is_empty() {
                                ui
                                    .label(RichText::new("Invalid regex").color(Color32::RED))
                                    .on_hover_ui(|ui| {
                                        ui.code(&state.filter_error);
                                    });
                            }
                        });
                        ui.separator();

                        egui::ScrollArea::vertical().show(ui, |ui| {
                            let selected = state.selected.clone();
                            if let Some(props) = state.props.as_mut() {
                                //println!("OK 1");
                                if let Some(root) = props.root.clone() {
                                    props.show_property(ui, &selected, &root, &tx);
                                }
                            }
                        });
                    });
                });
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        add_fonts(ui.ctx());
        self.show_view(ui);
        self.controls_window(ui.ctx())
    }
}

impl Drop for CameraControl {
    fn drop(&mut self) {
        let Some(state) = self.state.as_ref() else {
            return;
        };

        let Some(j) = state.lock().thread.take() else {
            return;
        };

        self.state = None;
        if let Err(e) = j.join() {
            // TODO use catch_unwind inside the thread instead of this shit
            let e = if let Some(e) = e.downcast_ref::<String>() {
                e.clone()
            } else if let Some(e) = e.downcast_ref::<&str>() {
                e.to_string()
            } else {
                format!("{:?}", e)
            };
            log::error!("Error in joining preview thread: {}", e);
        }
    }
}

struct Properties {
    root: Option<NodeId>,
    leafs: HashMap<NodeId, (Node, Property)>,
}

impl Properties {
    fn parse(camera: &dyn CameraDevice) -> Self {
        let mut leafs: HashMap<NodeId, (Node, Property)> = HashMap::new();
        let nodes = camera.all_properties().unwrap();
        for node in nodes {
            let property = Self::handle_node(&node, camera);
            leafs.insert(node.id.clone(), (node,property)); // TODO check result
        }
        Self {
            root: camera.user_root_property().unwrap(),
            leafs,
        }
    }

    fn handle_node(node: &Node, camera: &dyn CameraDevice) -> Property {
        match &node {
            Node::Group(g) => {
                let mut group = Group {
                    basic: BasicProperty {
                        display_name: node.display_name.as_str().to_string(),
                        access_mode: AccessMode::ReadWrite,
                    },
                    children: Vec::new(),
                };
                for c in g.children.iter() {
                    group.children.push(c.clone());
                }
                Property::Group(group)
            },
            Node::Property(property) => {
                match property {
                    BirbProperty::Bool(b) => {
                        let prop = BoolProp {
                            basic: BasicProperty {
                                display_name: node.display_name.as_str().to_string(),
                                access_mode: property.access_mode,
                            },
                            value: b.value.as_ref().map(|v| v.as_value().cloned()).flatten().unwrap_or(false), // TODO handle reference
                            requested: b.value.as_ref().map(|v| v.as_value().cloned()).flatten().unwrap_or(false), // TODO handle reference
                        };
                        Property::Bool(prop)
                    },
                    BirbProperty::Integer(i) => {
                        let prop = IntProp {
                            basic: BasicProperty {
                                display_name: node.display_name.as_str().to_string(),
                                access_mode: i.access_mode,
                            },
                            value: i.value.unwrap_or(0),
                            requested: i.value.unwrap_or(0),
                            representation: i.representation.unwrap_or(Representation::PureNumber),
                            min: i.min.as_ref().map(|v| v.as_value().cloned()).flatten().unwrap_or(0), // TODO handle reference
                            max: i.max.as_ref().map(|v| v.as_value().cloned()).flatten().unwrap_or(0), // TODO handle reference
                            unit: i.unit.as_ref().map(|s| s.to_string()),
                        };
                        Property::Integer(prop)
                    },
                    BirbProperty::Float(f) => {
                        let prop = FloatProp {
                            basic: BasicProperty {
                                display_name: node.display_name.as_str().to_string(),
                                access_mode: f.access_mode,
                            },
                            value: f.value.unwrap_or(0.0),
                            requested: f.value.unwrap_or(0.0),
                            representation: f.representation.unwrap_or(Representation::PureNumber),
                            min: f.min.as_ref().map(|v| v.as_value().cloned()).flatten().unwrap_or(0.0), // TODO handle reference
                            max: f.max.as_ref().map(|v| v.as_value().cloned()).flatten().unwrap_or(0.0), // TODO handle reference
                            unit: f.unit.as_ref().map(|s| s.to_string()),
                        };
                        Property::Float(prop)
                    },
                    BirbProperty::Enum(e) => {
                        let prop = EnumProp {
                            basic: BasicProperty {
                                display_name: node.display_name.as_str().to_string(),
                                access_mode: e.access_mode,
                            },
                            value: e.value.as_ref().map(|v| v.as_value().cloned()).flatten().unwrap_or(0), // TODO handle reference
                            requested: e.value.as_ref().map(|v| v.as_value().cloned()).flatten().unwrap_or(0), // TODO handle reference
                            entries: e.entries.clone().into_owned(),
                        };
                        Property::Enum(prop)
                    },
                    BirbProperty::String(s) => {
                        let prop = StringProp {
                            basic: BasicProperty {
                                display_name: node.display_name.as_str().to_string(),
                                access_mode: s.access_mode,
                            },
                            value: "TODO".to_string(),
                            requested: "TODO".to_string(),
                            max_length: s.max_length as _, // TODO
                        };
                        Property::String(prop)
                    },
                    BirbProperty::Command(c) => {
                        let prop = CommandProp {
                            basic: BasicProperty {
                                display_name: node.display_name.as_str().to_string(),
                                access_mode: c.access_mode,
                            },
                            requested: false,
                        };
                        Property::Command(prop)
                    },
                }
            }
            _ => todo!(),
        }
    }

    pub fn update_node(&mut self, camera: &dyn CameraDevice, id: &NodeId) {
        let Some((node, property)) = self.leafs.get_mut(id) else {
            log::error!("Node not found: {id:?}");
            return;
        };

        match property {
            Property::Group(_) => {},
            Property::Bool(b) => {
                let v = match camera.read_property(node) {
                    Ok(v) => v,
                    Err(e) => {
                        log::error!("Error reading BOOL property: {id:?}: {e}");
                        return;
                    }
                };
                let v = match v {
                    PropertyState::Bool(v) => v,
                    _ => {
                        log::error!("Error reading BOOL property: wrong type");
                        return;
                    },
                };
                b.set_value(v);
            },
            Property::Integer(i) => {
                let v = match camera.read_property(node) {
                    Ok(v) => v,
                    Err(e) => {
                        log::error!("Error reading INT property: {id:?}: {e}");
                        return;
                    }
                };
                let v = match v {
                    PropertyState::Int(v) => v,
                    _ => {
                        log::error!("Error reading INT property: wrong type");
                        return;
                    },
                };
                i.set_value(v.current);
                i.min = *v.range.start();
                i.max = *v.range.end();
            },
            Property::Float(f) => {
                let v = match camera.read_property(node) {
                    Ok(v) => v,
                    Err(e) => {
                        log::error!("Error reading FLOAT property: {id:?}: {e}");
                        return;
                    }
                };
                let v = match v {
                    PropertyState::Float(v) => v,
                    _ => {
                        log::error!("Error reading FLOAT property: wrong type");
                        return;
                    },
                };
                f.set_value(v.current);
                f.min = *v.range.start();
                f.max = *v.range.end();
            },
            Property::Enum(e) => {
                let v = match camera.read_property(node) {
                    Ok(v) => v,
                    Err(e) => {
                        log::error!("Error reading ENUM property: {id:?}: {e}");
                        return;
                    }
                };
                let v = match v {
                    PropertyState::Enum(v) => v,
                    _ => {
                        log::error!("Error reading ENUM property: wrong type");
                        return;
                    },
                };
                // TODO use support
                e.set_value(v.current);
            },
            Property::String(e) => {
                let v = match camera.read_property(node) {
                    Ok(v) => v,
                    Err(e) => {
                        log::error!("Error reading STRING property: {id:?}: {e}");
                        return;
                    }
                };
                let v = match v {
                    PropertyState::String(v) => v,
                    _ => {
                        log::error!("Error reading STRING property: wrong type");
                        return;
                    },
                };
                // TODO use support
                e.set_value(v);
            },
            Property::Command(_) => {},
        }
    }

    pub fn update_all_nodes(&mut self, camera: &dyn CameraDevice) {
        for id in self.leafs.keys().cloned().collect::<Vec<_>>() {
            self.update_node(camera, &id);
        }
    }

    pub fn write_node(&mut self, camera: &dyn CameraDevice, id: &NodeId, force: bool)-> bool {
        let Some((node, property)) = self.leafs.get_mut(id) else {
            log::error!("Node not found: {id:?}");
            return false;
        };

        match property {
            Property::Group(_) => {
                return false;
            },
            Property::Bool(b) => {
                if !force && b.value == b.requested {
                    return false;
                }
                let v = b.requested;
                // HACK: remove and proper erro handling in case of write/read failures:
                b.requested = b.value;
                if let Err(e) = camera.write_property(node, PropertyValue::Bool(v)) {
                    log::error!("Error writing BOOL property: {id:?}: {e}");
                }
            },
            Property::Integer(i) => {
                if !force && i.value == i.requested {
                    return false;
                }
                let v = i.requested;
                // HACK: remove and proper erro handling in case of write/read failures:
                i.requested = i.value;
                if let Err(e) = camera.write_property(node, PropertyValue::Integer(v)) {
                    log::error!("Error writing INT property: {id:?}: {e}");
                }
            },
            Property::Float(f) => {
                if !force && f.value == f.requested {
                    return false;
                }
                let v = f.requested;
                // HACK: remove and proper erro handling in case of write/read failures:
                f.requested = f.value;
                if let Err(e) = camera.write_property(node, PropertyValue::Float(v)) {
                    log::error!("Error writing FLOAT property: {id:?}: {e}");
                }
            },
            Property::Enum(e) => {
                if !force && e.value == e.requested {
                    return false;
                }
                let v = e.requested;
                // HACK: remove and proper erro handling in case of write/read failures:
                e.requested = e.value;
                if let Err(e) = camera.write_property(node, PropertyValue::Enum(v)) {
                    log::error!("Error writing ENUM property: {id:?}: {e}");
                }
            },
            Property::String(e) => {
                if !force && e.value == e.requested {
                    return false;
                }
                let v = e.requested.clone();
                // HACK: remove and proper erro handling in case of write/read failures:
                e.requested = e.value.clone();
                if let Err(e) = camera.write_property(node, PropertyValue::String(v)) {
                    log::error!("Error writing STRING property: {id:?}: {e}");
                }
            },
            Property::Command(c) => {
                if !force && !c.requested {
                    return false;
                }
                c.requested = false;
                if let Err(e) = camera.write_property(node, PropertyValue::Command) {
                    log::error!("Error sending COMMAND property: {id:?}: {e}");
                }
            },
        }

        true
    }

    fn write_all_nodes(&mut self, camera: &dyn CameraDevice){
        for id in self.leafs.keys().cloned().collect::<Vec<_>>() {
            if self.write_node(camera, &id, false) {
                self.update_node(camera, &id);
            }
        }
    }

    pub fn show_property(&mut self, ui: &mut Ui, selected: &HashSet<NodeId>, id: &NodeId, sender: &Sender<Command>) {
        let Some((node, property)) = self.leafs.get_mut(id) else {
            ui.label(RichText::new(format!("&{:?}", id)).color(Color32::RED));
            return;
        };

        match property {
            Property::Group(ref g) => {
                let display_name = g.display_name.clone();
                let children = g.children.clone();
                Self::show_group(self, ui, selected, &display_name, children, sender);
            },
            Property::Bool(b) => Self::show_bool(ui, b, sender),
            Property::Integer(i) => Self::show_int(ui, i, sender),
            Property::Float(f) => Self::show_float(ui, f, sender),
            Property::Enum(e) => Self::show_enum(ui, e, sender),
            Property::String(s) => Self::show_string(ui, s, sender),
            Property::Command(c) => Self::show_command(ui, c, sender),
        }
    }

    pub fn show_group(&mut self, ui: &mut Ui, selected: &HashSet<NodeId>, display_name: &str, children: impl IntoIterator<Item = NodeId>, sender: &Sender<Command>) {
        ui.collapsing(display_name, |ui| {
            Grid::new(display_name).striped(true).show(ui, |ui| {
                for id in children {
                    if !selected.contains(&id) {
                        continue;
                    }
                    self.show_property(ui, selected, &id, sender);
                    ui.end_row();
                }
            });
        });
    }

    pub fn show_bool(ui: &mut Ui, b: &mut BoolProp, sender: &Sender<Command>) {
        ui.add_enabled_ui(b.access_mode.writable(), |ui| {
            if ui.checkbox(&mut b.requested, b.basic.display_name.as_str()).changed() {
                sender.send(Command::Write).unwrap();
            };
        });
    }

    pub fn show_int(ui: &mut Ui, i: &mut IntProp, sender: &Sender<Command>) {
        // TODO unit
        match i.representation {
            Representation::Hex => {
                ui.add_enabled_ui(i.access_mode.writable(), |ui| {
                    ui.horizontal(|ui| {
                        ui.label("0x");
                        if ui.add(DragValue::new(&mut i.requested).hexadecimal(4, true, false)).changed() {
                            sender.send(Command::Write).unwrap();
                        }
                        ui.label(i.basic.display_name.as_str())
                    });
                });
            },
            Representation::PureNumber => {
                ui.add_enabled_ui(i.access_mode.writable(), |ui| {
                    ui.horizontal(|ui| {
                        if ui.add(DragValue::new(&mut i.requested)).changed() {
                            sender.send(Command::Write).unwrap();
                        }
                        ui.label(i.basic.display_name.as_str())
                    });
                });
            },
            Representation::Linear => {
                ui.add_enabled_ui(i.access_mode.writable(), |ui| {
                    if ui.add(egui::Slider::new(&mut i.requested, i.min..=i.max).text(i.basic.display_name.as_str())).changed() {
                        sender.send(Command::Write).unwrap();
                    }
                });
            },
            Representation::Logarithmic => {
                ui.add_enabled_ui(i.access_mode.writable(), |ui| {
                    if ui.add(egui::Slider::new(&mut i.requested, i.min..=i.max).text(i.basic.display_name.as_str()).logarithmic(true)).changed() {
                        sender.send(Command::Write).unwrap();
                    }
                });
            },
            Representation::Boolean => {
                ui.label(RichText::new(format!("invalid representation: {:?}", i.representation)));
            },
        }
    }

    pub fn show_float(ui: &mut Ui, f: &mut FloatProp, sender: &Sender<Command>) {
        // TODO unit
        match f.representation {
            Representation::PureNumber => {
                ui.add_enabled_ui(f.access_mode.writable(), |ui| {
                    ui.horizontal(|ui| {
                        if ui.add(DragValue::new(&mut f.requested)).changed() {
                            sender.send(Command::Write).unwrap();
                        }
                        ui.label(f.basic.display_name.as_str())
                    });
                });
            },
            Representation::Linear if f.min.is_finite() && f.max.is_finite() => {
                ui.add_enabled_ui(f.access_mode.writable(), |ui| {
                    ui.horizontal(|ui| {
                        if ui.add(egui::Slider::new(&mut f.requested, f.min..=f.max).text(f.basic.display_name.as_str())).changed() {
                            sender.send(Command::Write).unwrap();
                        }
                        ui.label(f.basic.display_name.as_str())
                    });
                });
            },
            Representation::Logarithmic => {
                ui.add_enabled_ui(f.access_mode.writable(), |ui| {
                    if ui.add(egui::Slider::new(&mut f.requested, f.min..=f.max).text(f.basic.display_name.as_str()).logarithmic(true)).changed() {
                        sender.send(Command::Write).unwrap();
                    }
                });
            },
            Representation::Hex | Representation::Boolean | Representation::Linear => {
                ui.add_enabled_ui(f.access_mode.writable(), |ui| {
                    if ui.label(RichText::new(format!("invalid representation: {:?}", f.representation))).changed() {
                        sender.send(Command::Write).unwrap();
                    };
                });
            },
        }
    }

    pub fn show_enum(ui: &mut Ui, e: &mut EnumProp, sender: &Sender<Command>) {
        let selected_text = if let Some(entry) = e.entries.iter().find(|entry| entry.discriminant == e.requested) {
            entry.name.to_string()
        } else {
            "???".to_string()
        };

        ui.add_enabled_ui(e.access_mode.writable(), |ui| {
            egui::ComboBox::from_label(&e.display_name)
                .selected_text(selected_text)
                .show_ui(ui, |ui| {
                    for entry in e.entries.iter() {
                        ui.selectable_value(&mut e.requested, entry.discriminant, entry.name.as_str());
                    }
                });
        });

        if e.value != e.requested {
            sender.send(Command::Write).unwrap();
        }
    }

    pub fn show_string(ui: &mut Ui, s: &mut StringProp, sender: &Sender<Command>) {
        ui.add_enabled_ui(s.access_mode.writable(), |ui| {
            ui.horizontal(|ui| {
                if ui.text_edit_singleline(&mut s.requested).changed() {
                    sender.send(Command::Write).unwrap();
                }
                ui.label(&s.display_name);
            });
        });
    }

    pub fn show_command(ui: &mut Ui, c: &mut CommandProp, sender: &Sender<Command>) {
        ui.add_enabled_ui(c.access_mode.writable(), |ui| {
            if ui.button(c.display_name.as_str()).clicked() {
                c.requested = true;
                sender.send(Command::Write).unwrap();
            }
        });
    }
}

enum Property {
    Group(Group),
    Bool(BoolProp),
    Integer(IntProp),
    Float(FloatProp),
    Enum(EnumProp),
    String(StringProp),
    Command(CommandProp),
}

impl Property {
    fn children(&self) -> &[NodeId] {
        match self {
            Property::Group(g) => &g.children,
            _ => &[],
        }
    }

    fn filter(
        &self,
        props: &Properties,
        selected_set: &mut HashSet<NodeId>,
        re: &Option<Regex>,
        // TODO leaf_only: bool,
    ) -> bool {
        let matches = if let Some(re) = re.as_ref() {
            re.is_match(&self.display_name.to_lowercase()) // TODO lowercase as an option
        } else {
            true
        };

        let re_for_child = if matches {
            &None
        } else {
            re
        };

        let mut any_child_selected = false;
        for child_id in self.children() {
            //println!("child id: {:?}", child_id);
            let Some((_, child)) = props.leafs.get(child_id) else {
                continue;
            };
            let selected = child.filter(props, selected_set, re_for_child);
            if selected {
                selected_set.insert(child_id.clone());
            }
            any_child_selected |= selected;
        }

        matches | any_child_selected
    }
}

impl Deref for Property {
    type Target = BasicProperty;

    fn deref(&self) -> &Self::Target {
        match self {
            Property::Group(g) => &g.basic,
            Property::Bool(b) => &b.basic,
            Property::Integer(i) => &i.basic,
            Property::Float(f) => &f.basic,
            Property::Enum(e) => &e.basic,
            Property::String(s) => &s.basic,
            Property::Command(c) => &c.basic,
        }
    }
}

impl DerefMut for Property {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Property::Group(g) => &mut g.basic,
            Property::Bool(b) => &mut b.basic,
            Property::Integer(i) => &mut i.basic,
            Property::Float(f) => &mut f.basic,
            Property::Enum(e) => &mut e.basic,
            Property::String(s) => &mut s.basic,
            Property::Command(c) => &mut c.basic,
        }
    }
}

macro_rules! impl_basic {
    ($name:ident) => {
        impl Deref for $name {
            type Target = BasicProperty;

            fn deref(&self) -> &Self::Target {
                &self.basic
            }
        }

        impl std::ops::DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.basic
            }
        }
    };
    ($name:ident, $t:ty) => {
        impl_basic!($name);

        impl $name {
            fn set_value(&mut self, value: $t) {
                self.value = value.clone();
                self.requested = value;
            }
        }
    };
}

pub struct BasicProperty {
    display_name: String,
    access_mode: AccessMode,
}

struct Group {
    basic: BasicProperty,
    children: Vec<NodeId>,
}
impl_basic!(Group);

struct BoolProp {
    basic: BasicProperty,
    value: bool,
    requested: bool,
}
impl_basic!(BoolProp, bool);

struct IntProp {
    basic: BasicProperty,
    value: i64,
    requested: i64,
    representation: Representation,
    min: i64,
    max: i64,
    unit: Option<String>,
}
impl_basic!(IntProp, i64);

struct FloatProp {
    basic: BasicProperty,
    value: f64,
    requested: f64,
    representation: Representation,
    min: f64,
    max: f64,
    unit: Option<String>,
}
impl_basic!(FloatProp, f64);

pub struct EnumProp {
    basic: BasicProperty,
    value: i64,
    requested: i64,
    entries: Vec<EnumEntry>,
}
impl_basic!(EnumProp, i64);

pub struct StringProp {
    basic: BasicProperty,
    value: String,
    requested: String,
    max_length: usize,
}
impl_basic!(StringProp, String);

pub struct CommandProp {
    basic: BasicProperty,
    requested: bool,
}
impl_basic!(CommandProp);


fn icon(icon: impl ToString) -> RichText {
    RichText::new(icon.to_string()).size(20.0)
}

pub fn add_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    // TODO MaterialIcons provided font is outdated
    fonts.font_data.insert("MaterialIcons".into(), FontData::from_static(material_icons::FONT));
    fonts.families.get_mut(&FontFamily::Proportional).unwrap().insert(2, "MaterialIcons".into());
    ctx.set_fonts(fonts);
}
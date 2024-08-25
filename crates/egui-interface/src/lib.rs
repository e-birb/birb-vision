use std::{collections::{HashMap, HashSet}, ops::{Deref, DerefMut}, sync::{Arc, Weak}};

use birb_vision::{CameraDevice, Child, EnumEntry, Node, NodeId, NodeVariant, PropertyVariant, Representation};
use egui::{mutex::Mutex, Color32, ComboBox, DragValue, RichText, TextBuffer, Ui};
use regex::Regex;


pub struct Preview {
    state: Option<Arc<Mutex<PreviewState>>>,
}

pub struct PreviewState {
    props: Option<Properties>,
    filter: String,
    filter_error: String,
    selected: HashSet<NodeId>,
    updated: HashSet<NodeId>,
    thread: Option<std::thread::JoinHandle<()>>,
    update: Box<dyn Fn() + Send + Sync>,
}

impl PreviewState {
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

impl PreviewState {
    fn new() -> Self {
        PreviewState {
            props: None,
            filter: String::new(),
            filter_error: String::new(),
            selected: Default::default(),
            updated: HashSet::new(),
            thread: None,
            update: Box::new(|| {}),
        }
    }
}

struct StateRef(Weak<Mutex<PreviewState>>);

impl StateRef {
    fn on_state<R>(&self, f: impl FnOnce(&PreviewState) -> R) -> Option<R> {
        self.0.upgrade().map(|state| {
            let state = state.lock();
            let r = f(&state);
            r
        })
    }

    fn on_state_mut<R>(&self, f: impl FnOnce(&mut PreviewState) -> R) -> Option<R> {
        self.0.upgrade().map(|state| {
            let mut state = state.lock();
            let r = f(&mut state);
            (state.update)();
            r
        })
    }
}

impl Preview {
    pub fn new() -> Self {
        Preview {
            state: None,
        }
    }

    pub fn init(
        &mut self,
        init: impl FnOnce() -> Box<dyn CameraDevice> + Send + 'static,
    ) {
        let mut state = PreviewState::new();
        let state = Arc::new(Mutex::new(state));

        let thread = std::thread::spawn({
            let state = StateRef(Arc::downgrade(&state));

            move || {
                let mut camera = init();
                camera.open(Default::default()).unwrap();

                let properties = camera.control_description().unwrap();
                let ui_properties = camera.properties().unwrap();
                let mut properties = Properties::parse(&properties);
                properties.root = ui_properties.id.clone().unwrap().into();
                properties.update_all_nodes(&*camera);

                state.on_state_mut(|state| {
                    let re = state.filter_re();
                    let mut selected_nodes = HashSet::new();
                    let root_id = properties.root.as_ref().unwrap();
                    let root = properties.leafs.get(root_id).unwrap();
                    root.filter(&properties, &mut selected_nodes, &re);
                    // unnecessary? selected_nodes.insert(root_id.clone());

                    state.props = properties.into();
                    state.selected = selected_nodes;
                });

                loop {
                    std::thread::yield_now();

                    if state.on_state_mut(|state| {
                        let props = state.props.as_mut().unwrap();
                        props.write_all_nodes(&*camera);
                    }).is_none() {
                        break;
                    }
                }
            }
        });

        state.lock().thread = Some(thread);

        self.state = Some(state.clone());
    }

    pub fn show(&self, ui: &mut egui::Ui) {
        if let Some(state) = self.state.as_ref() {
            let mut state = state.lock();
            state.update = Box::new({
                let cx = ui.ctx().clone();
                move || cx.request_repaint()
            });

            if state.props.is_some() {
                ui.label("Hello World!");

                ui.horizontal(|ui| {
                    ui.label("filter");
                    if ui.text_edit_singleline(&mut state.filter).changed() {
                        let re = state.filter_re();
                        if let Some(props) = &state.props {
                            let mut selected = HashSet::new();
                            let root_id = props.root.as_ref().unwrap();
                            let root = props.leafs.get(root_id).unwrap();
                            root.filter(&props, &mut selected, &re);
                            state.selected = selected;
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
                
                // Self::show_node(ui, state.props.as_ref().unwrap());
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let selected = state.selected.clone();
                    if let Some(props) = state.props.as_mut() {
                        //println!("OK 1");
                        if let Some(root) = props.root.clone() {
                            //println!("OK 2");
                            props.show_property(ui, &selected, &root);
                        }
                    }
                });
            }
        }
    }
}

impl Drop for Preview {
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
    leafs: HashMap<NodeId, Property>,
}

impl Properties {
    fn parse(node: &Node) -> Self {
        let mut leafs = HashMap::new();
        let root = Self::handle_node(node, &mut leafs);
        Self {
            root,
            leafs,
        }
    }

    fn handle_node(node: &Node, leafs: &mut HashMap<NodeId, Property>) -> Option<NodeId> {
        let id = node.id.clone();

        match &node.variant {
            NodeVariant::Group(g) => {
                let mut group = Group {
                    basic: BasicProperty {
                        display_name: node.display_name.as_str().to_string(),
                    },
                    children: Vec::new(),
                };
                for c in g.children.iter() {
                    match c {
                        Child::Ref(id) => {
                            group.children.push(id.clone());
                        },
                        Child::Node(n) => {
                            if let Some(id) = Self::handle_node(n, leafs) {
                                group.children.push(id);
                            }
                        }
                    }
                }
                if let Some(id) = &id {
                    leafs.insert(id.clone(), Property::Group(group));
                }
            },
            NodeVariant::Property(property) => {
                match property {
                    PropertyVariant::Boolean(b) => {
                        let prop = BoolProp {
                            basic: BasicProperty {
                                display_name: node.display_name.as_str().to_string(),
                            },
                            value: b.value.unwrap_or(false),
                            requested: b.value.unwrap_or(false),
                        };
                        if let Some(id) = &id {
                            leafs.insert(id.clone(), Property::Bool(prop));
                        }
                    },
                    PropertyVariant::Integer(i) => {
                        let prop = IntProp {
                            basic: BasicProperty {
                                display_name: node.display_name.as_str().to_string(),
                            },
                            value: i.value.unwrap_or(0),
                            requested: i.value.unwrap_or(0),
                            representation: i.representation.unwrap_or(Representation::PureNumber),
                            min: i.min.unwrap_or(0),
                            max: i.max.unwrap_or(0),
                            unit: i.unit.as_ref().map(|s| s.to_string()),
                        };
                        if let Some(id) = &id {
                            leafs.insert(id.clone(), Property::Int(prop));
                        }
                    },
                    PropertyVariant::Float(f) => {
                        let prop = FloatProp {
                            basic: BasicProperty {
                                display_name: node.display_name.as_str().to_string(),
                            },
                            value: f.value.unwrap_or(0.0),
                            requested: f.value.unwrap_or(0.0),
                            representation: f.representation.unwrap_or(Representation::PureNumber),
                            min: f.min.unwrap_or(0.0),
                            max: f.max.unwrap_or(0.0),
                            unit: f.unit.as_ref().map(|s| s.to_string()),
                        };
                        if let Some(id) = &id {
                            leafs.insert(id.clone(), Property::Float(prop));
                        }
                    },
                    PropertyVariant::Enum(e) => {
                        let prop = EnumProp {
                            basic: BasicProperty {
                                display_name: node.display_name.as_str().to_string(),
                            },
                            value: e.value.unwrap_or(0),
                            requested: e.value.unwrap_or(0),
                            entries: e.entries.clone().into_owned(),
                        };
                        if let Some(id) = &id {
                            leafs.insert(id.clone(), Property::Enum(prop));
                        }
                    },
                    PropertyVariant::String(s) => {
                        let prop = StringProp {
                            basic: BasicProperty {
                                display_name: node.display_name.as_str().to_string(),
                            },
                            value: "TODO".to_string(),
                            requested: "TODO".to_string(),
                            max_length: s.max_length as _, // TODO
                        };
                        if let Some(id) = &id {
                            leafs.insert(id.clone(), Property::String(prop));
                        }
                    },
                    PropertyVariant::Command => {
                        let prop = CommandProp {
                            basic: BasicProperty {
                                display_name: node.display_name.as_str().to_string(),
                            },
                        };
                        if let Some(id) = &id {
                            leafs.insert(id.clone(), Property::Command(prop));
                        }
                    },
                }
            }
            _ => todo!(),
        }

        id
    }

    pub fn update_node(&mut self, camera: &dyn CameraDevice, id: &NodeId) {
        let Some(node) = self.leafs.get_mut(id) else {
            log::error!("Node not found: {id:?}");
            return;
        };

        match node {
            Property::Group(_) => {},
            Property::Bool(b) => {
                let v = match camera.get_bool_property(id) {
                    Ok(v) => v,
                    Err(e) => {
                        log::error!("Error reading BOOL property: {id:?}: {e}");
                        return;
                    }
                };
                b.set_value(v);
            },
            Property::Int(i) => {
                let v = match camera.get_int_property(id) {
                    Ok(v) => v,
                    Err(e) => {
                        log::error!("Error reading INT property: {id:?}: {e}");
                        return;
                    }
                };
                i.set_value(v.current);
                i.min = *v.range.start();
                i.max = *v.range.end();
            },
            Property::Float(f) => {
                let v = match camera.get_float_property(id) {
                    Ok(v) => v,
                    Err(e) => {
                        log::error!("Error reading FLOAT property: {id:?}: {e}");
                        return;
                    }
                };
                f.set_value(v.current);
                f.min = *v.range.start();
                f.max = *v.range.end();
            },
            Property::Enum(e) => {
                let v = match camera.get_enum_property(id) {
                    Ok(v) => v,
                    Err(e) => {
                        log::error!("Error reading ENUM property: {id:?}: {e}");
                        return;
                    }
                };
                // TODO use support
                e.set_value(v.current);
            },
            Property::String(e) => {
                let v = match camera.get_string_property(id) {
                    Ok(v) => v,
                    Err(e) => {
                        log::error!("Error reading STRING property: {id:?}: {e}");
                        return;
                    }
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
        let Some(node) = self.leafs.get_mut(id) else {
            log::error!("Node not found: {id:?}");
            return false;
        };

        match node {
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
                if let Err(e) = camera.set_bool_property(id, v) {
                    log::error!("Error writing BOOL property: {id:?}: {e}");
                }
            },
            Property::Int(i) => {
                if !force && i.value == i.requested {
                    return false;
                }
                let v = i.requested;
                // HACK: remove and proper erro handling in case of write/read failures:
                i.requested = i.value;
                if let Err(e) = camera.set_int_property(id, v) {
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
                if let Err(e) = camera.set_float_property(id, v) {
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
                if let Err(e) = camera.set_enum_property(id, v) {
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
                if let Err(e) = camera.set_string_property(id, &v) {
                    log::error!("Error writing STRING property: {id:?}: {e}");
                }
            },
            Property::Command(_) => {
                if !force {
                    return false;
                }
                if let Err(e) = camera.send_command(id) {
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

    pub fn show_property(&mut self, ui: &mut Ui, selected: &HashSet<NodeId>, id: &NodeId) {
        let Some(property) = self.leafs.get_mut(id) else {
            ui.label(RichText::new("&".to_string() + id.0.as_str()).color(Color32::RED));
            return;
        };

        match property {
            Property::Group(ref g) => {
                let display_name = g.display_name.clone();
                let children = g.children.clone();
                Self::show_group(self, ui, selected, &display_name, children);
            },
            Property::Bool(b) => Self::show_bool(ui, b),
            Property::Int(i) => Self::show_int(ui, i),
            Property::Float(f) => Self::show_float(ui, f),
            Property::Enum(e) => Self::show_enum(ui, e),
            Property::String(s) => Self::show_string(ui, s),
            Property::Command(c) => Self::show_command(ui, c),
        }
    }

    pub fn show_group(&mut self, ui: &mut Ui, selected: &HashSet<NodeId>, display_name: &str, children: impl IntoIterator<Item = NodeId>) {
        ui.collapsing(display_name, |ui| {
            for id in children {
                if !selected.contains(&id) {
                    continue;
                }
                self.show_property(ui, selected, &id)
            }
        });
    }

    pub fn show_bool(ui: &mut Ui, b: &mut BoolProp) {
        ui.checkbox(&mut b.requested, b.basic.display_name.as_str());
    }

    pub fn show_int(ui: &mut Ui, i: &mut IntProp) {
        // TODO unit
        match i.representation {
            Representation::Hex => {
                ui.horizontal(|ui| {
                    ui.label("0x");
                    ui.add(DragValue::new(&mut i.requested).hexadecimal(4, true, false));
                    ui.label(i.basic.display_name.as_str())
                });
            },
            Representation::PureNumber => {
                ui.horizontal(|ui| {
                    ui.add(DragValue::new(&mut i.requested));
                    ui.label(i.basic.display_name.as_str())
                });
            },
            Representation::Linear => {
                ui.add(egui::Slider::new(&mut i.requested, i.min..=i.max).text(i.basic.display_name.as_str()));
            },
            Representation::Logarithmic => {
                ui.add(egui::Slider::new(&mut i.requested, i.min..=i.max).text(i.basic.display_name.as_str()).logarithmic(true));
            },
            Representation::Boolean => {
                ui.label(RichText::new(format!("invalid representation: {:?}", i.representation)));
            },
        }
    }

    pub fn show_float(ui: &mut Ui, f: &mut FloatProp) {
        // TODO unit
        match f.representation {
            Representation::PureNumber => {
                ui.horizontal(|ui| {
                    ui.add(DragValue::new(&mut f.requested));
                    ui.label(f.basic.display_name.as_str())
                });
            },
            Representation::Linear if f.min.is_finite() && f.max.is_finite() => {
                ui.horizontal(|ui| {
                    ui.add(egui::Slider::new(&mut f.requested, f.min..=f.max).text(f.basic.display_name.as_str()));
                    ui.label(f.basic.display_name.as_str())
                });
            },
            Representation::Logarithmic => {
                ui.add(egui::Slider::new(&mut f.requested, f.min..=f.max).text(f.basic.display_name.as_str()).logarithmic(true));
            },
            Representation::Hex | Representation::Boolean | Representation::Linear => {
                ui.label(RichText::new(format!("invalid representation: {:?}", f.representation)));
            },
        }
    }

    pub fn show_enum(ui: &mut Ui, e: &mut EnumProp) {
        let selected_text = if let Some(entry) = e.entries.iter().find(|entry| entry.discriminant == e.requested) {
            entry.name.to_string()
        } else {
            "???".to_string()
        };

        egui::ComboBox::from_label(&e.display_name)
            .selected_text(selected_text)
            .show_ui(ui, |ui| {
                for entry in e.entries.iter() {
                    ui.selectable_value(&mut e.requested, entry.discriminant, entry.name.as_str());
                }
            }
        );
    }

    pub fn show_string(ui: &mut Ui, s: &mut StringProp) {
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut s.requested);
            ui.label(&s.display_name);
        });
    }

    pub fn show_command(ui: &mut Ui, c: &CommandProp) {
        if ui.button(c.display_name.as_str()).clicked() {
            // TODO
        }
    }
}

enum Property {
    Group(Group),
    Bool(BoolProp),
    Int(IntProp),
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
            println!("child id: {:?}", child_id);
            let Some(child) = props.leafs.get(child_id) else {
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
            Property::Int(i) => &i.basic,
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
            Property::Int(i) => &mut i.basic,
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
}
impl_basic!(CommandProp);

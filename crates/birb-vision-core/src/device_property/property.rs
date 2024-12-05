use std::ops::{Deref, DerefMut};

use enum_as_inner::EnumAsInner;
use serde::{Deserialize, Serialize};

use super::{NodeId, NodeInfo};

mod bool;
mod numeric;
mod string;
mod enum_property;
mod command;

pub use bool::*;
pub use numeric::*;
pub use string::*;
pub use enum_property::*;
pub use command::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Serialize, Deserialize)]
#[derive(EnumAsInner)]
pub enum AccessMode {
    ReadOnly,
    WriteOnly,
    ReadWrite,
}

impl AccessMode {
    pub fn readable(&self) -> bool {
        match self {
            AccessMode::ReadOnly | AccessMode::ReadWrite => true,
            AccessMode::WriteOnly => false,
        }
    }
    pub fn writable(&self) -> bool {
        match self {
            AccessMode::WriteOnly | AccessMode::ReadWrite => true,
            AccessMode::ReadOnly => false,
        }
    }
}

#[derive(Debug, Clone)]
#[derive(EnumAsInner)]
#[derive(Serialize, Deserialize)]
pub enum Property {
    Bool(BoolProperty),
    Integer(NumericProperty<i64>),
    Float(NumericProperty<f64>),
    Enum(EnumProperty),
    String(StringProperty),
    Command(CommandProperty),
}

impl Deref for Property {
    type Target = ControlInfo;

    fn deref(&self) -> &Self::Target {
        match self {
            Property::Bool(p) => &p.info,
            Property::Integer(p) => &p.info,
            Property::Float(p) => &p.info,
            Property::Enum(p) => &p.info,
            Property::String(p) => &p.info,
            Property::Command(p) => &p.info,
        }
    }
}

impl DerefMut for Property {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Property::Bool(p) => &mut p.info,
            Property::Integer(p) => &mut p.info,
            Property::Float(p) => &mut p.info,
            Property::Enum(p) => &mut p.info,
            Property::String(p) => &mut p.info,
            Property::Command(p) => &mut p.info,
        }
    }
}

#[derive(Debug, Clone)]
#[derive(EnumAsInner)]
#[derive(Serialize, Deserialize)]
pub enum PropertyState {
    Bool(bool),
    Int(NumericState<i64>),
    Float(NumericState<f64>),
    Enum(EnumState<'static>), // TODO ...
    String(String),
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[derive(EnumAsInner)]
#[derive(Serialize, Deserialize)]
pub enum PropertyValue {
    Bool(bool),
    Integer(i64),
    Float(f64),
    Enum(i64),
    String(String),
    Command,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Serialize, Deserialize)]
#[derive(EnumAsInner)]
pub enum ValueOrRef<T> {
    Value(T),
    Ref(NodeId),
}

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct ControlInfo {
    pub node_info: NodeInfo,
    pub access_mode: AccessMode,
    pub is_locked_ref: Option<NodeId>,
    pub address: Option<ValueOrRef<u64>>,
    pub port_ref: Option<NodeId>,
    pub streamable: bool,
}

impl Deref for ControlInfo {
    type Target = NodeInfo;
    fn deref(&self) -> &Self::Target {
        &self.node_info
    }
}

impl DerefMut for ControlInfo {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.node_info
    }
}

impl ControlInfo {
    pub fn new(id: impl Into<NodeId>) -> Self {
        Self {
            node_info: NodeInfo::new_with_id(id),
            access_mode: AccessMode::ReadWrite,
            is_locked_ref: None,
            address: None,
            port_ref: None,
            streamable: false,
        }
    }

    pub fn new_const(id: impl Into<NodeId>) -> Self {
        let mut s = Self::new(id);
        s.access_mode = AccessMode::ReadOnly;
        s
    }
}
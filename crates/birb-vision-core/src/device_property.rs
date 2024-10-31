use std::{borrow::Cow, fmt::Debug, ops::{Deref, DerefMut, RangeInclusive}};

use enum_as_inner::EnumAsInner;
use serde::{Deserialize, Serialize};

mod id;

pub use id::*;

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct NodeInfo {
    /// A Friendly name for the property
    pub display_name: String,

    /// The property id
    ///
    /// # Notes
    /// - If [`None`], the property is not directly accessible. This is the case for groups.
    pub id: NodeId,

    pub tooltip: Option<String>,
    pub description: Option<String>,

    pub visibility: Option<Visibility>,
}

impl NodeInfo {
    pub fn new_with_id(id: impl Into<NodeId>) -> Self {
        let id = id.into();
        let name = format!("{:?}", id);
        NodeInfo {
            display_name: name.clone(),
            id,
            tooltip: None,
            description: None,
            visibility: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Node {
    pub info: NodeInfo,

    pub access_mode: AccessMode,

    pub is_locked_ref: Option<NodeId>,

    pub address: Option<u64>,
    pub address_ref: Option<NodeId>,

    pub port_ref: Option<NodeId>,

    pub streamable: bool,

    /// The variant of the node
    pub variant: NodeVariant,
}

impl Deref for Node {
    type Target = NodeInfo;
    fn deref(&self) -> &Self::Target {
        &self.info
    }
}

impl DerefMut for Node {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.info
    }
}

impl Node {
    pub fn new_with_id(id: impl Into<NodeId>) -> Self {
        Node {
            info: NodeInfo::new_with_id(id),
            access_mode: AccessMode::ReadWrite,
            is_locked_ref: None,
            address: None,
            address_ref: None,
            port_ref: None,
            streamable: false,
            variant: NodeVariant::Group(GroupNode {
                children: Cow::Borrowed(&[]),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Serialize, Deserialize)]
pub enum Visibility {
    Beginner,
    Expert,
    Guru,
    Invisible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
pub enum NodeVariant {
    Group(GroupNode),
    Property(PropertyVariant),
    Port, // TODO ID
}

impl From<GroupNode> for NodeVariant {
    fn from(group: GroupNode) -> Self {
        NodeVariant::Group(group)
    }
}

impl From<BoolProperty> for NodeVariant {
    fn from(prop: BoolProperty) -> Self {
        NodeVariant::Property(PropertyVariant::Bool(prop))
    }
}

impl From<PropertyVariant> for NodeVariant {
    fn from(prop: PropertyVariant) -> Self {
        NodeVariant::Property(prop)
    }
}

#[derive(Debug, Clone)]
#[derive(EnumAsInner)]
pub enum PropertyVariant {
    Bool(BoolProperty),
    Integer(NumericProperty<i64>),
    Float(NumericProperty<f64>),
    Enum(EnumProperty),
    String(StringProperty),
    Command, // TODO command might have data!
}

impl From<EnumProperty> for NodeVariant {
    fn from(prop: EnumProperty) -> Self {
        NodeVariant::Property(PropertyVariant::Enum(prop))
    }
}

impl From<StringProperty> for NodeVariant {
    fn from(prop: StringProperty) -> Self {
        NodeVariant::Property(PropertyVariant::String(prop))
    }
}

#[derive(Debug, Clone)]
pub struct GroupNode {
    pub children: Cow<'static, [NodeId]>,
}

#[derive(Debug, Clone)]
pub struct BoolProperty {
    pub value: Option<bool>,
    pub value_ref: Option<NodeId>,
    pub default: Option<bool>,
}

impl Default for BoolProperty {
    fn default() -> Self {
        BoolProperty {
            value: None,
            value_ref: None,
            default: None,
        }
    }
}

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct NumericProperty<T> {
    pub value: Option<T>,
    pub min: Option<T>,
    pub min_ref: Option<NodeId>,
    pub max: Option<T>,
    pub max_ref: Option<NodeId>,
    pub increment: Option<T>,
    pub increment_ref: Option<NodeId>,
    pub default: Option<T>,
    pub unit: Option<Cow<'static, str>>,
    pub slope: Slope,
    pub representation: Option<Representation>,
}

impl<T> Default for NumericProperty<T> {
    fn default() -> Self {
        NumericProperty {
            value: None,
            min: None,
            min_ref: None,
            max: None,
            max_ref: None,
            increment: None,
            increment_ref: None,
            default: None,
            unit: None,
            slope: Slope::Increasing,
            representation: None,
        }
    }
}

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct EnumProperty {
    pub value: Option<i64>,
    pub value_ref: Option<NodeId>,
    pub entries: Cow<'static, [EnumEntry]>,
}

impl Default for EnumProperty {
    fn default() -> Self {
        EnumProperty {
            value: None,
            value_ref: None,
            entries: Cow::Borrowed(&[]),
        }
    }
}

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct NumericValue<T> {
    pub current: T,
    pub range: RangeInclusive<T>, // TODO support not range
}

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct EnumValue<'a> {
    pub current: i64,
    pub support: Cow<'a, [i64]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Serialize, Deserialize)]
pub enum Slope {
    Increasing,
    Decreasing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Serialize, Deserialize)]
pub enum Representation {
    Boolean,
    PureNumber,
    Hex,
    Linear,
    Logarithmic,
}

#[derive(Debug, Clone)]
pub enum NumericSupport<T: Clone + 'static> {
    Range(RangeInclusive<T>),
    Set(Cow<'static, [T]>)
}

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct EnumEntry {
    pub discriminant: i64,
    pub name: Cow<'static, str>,
    pub help: Option<Cow<'static, str>>,
}

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct StringProperty {
    pub max_length: u32,
    pub default: Option<String>,
}

impl Default for StringProperty {
    fn default() -> Self {
        StringProperty {
            max_length: u32::MAX,
            default: None,
        }
    }
}

#[derive(Debug, Clone)]
#[derive(EnumAsInner)]
#[derive(Serialize, Deserialize)]
pub enum PropertyState {
    Bool(bool),
    Int(NumericValue<i64>),
    Float(NumericValue<f64>),
    Enum(EnumValue<'static>), // TODO ...
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
use std::{borrow::Cow, fmt::{Debug, Formatter}, ops::{Deref, RangeInclusive}};

use enum_as_inner::EnumAsInner;
use serde::{Deserialize, Serialize};

mod id;

pub use id::*;

#[derive(Debug, Clone)]
pub struct Node {
    /// A Friendly name for the property
    pub display_name: Cow<'static, str>,

    /// The property id
    ///
    /// # Notes
    /// - If [`None`], the property is not directly accessible. This is the case for groups.
    pub id: Option<NodeId>,

    pub tooltip: Option<Cow<'static, str>>,
    pub description: Option<Cow<'static, str>>,

    /// The visibility of the property
    pub visibility: Option<Visibility>,

    pub imposed_access_mode: Option<AccessMode>,
    pub access_mode: Option<AccessMode>,

    pub is_locked_ref: Option<NodeId>,

    pub address: Option<u64>,
    pub address_ref: Option<NodeId>, // TODO unify with a variant, maybe ValueOrRef or something similar to also replace Child

    pub port_ref: Option<NodeId>,

    pub streamable: bool,

    /// The variant of the node
    pub variant: NodeVariant,
}

impl Node {
    pub fn new(display_name: impl Into<Cow<'static, str>>) -> Self {
        Node {
            display_name: display_name.into(),
            id: None,
            tooltip: None,
            description: None,
            visibility: None,
            imposed_access_mode: None,
            access_mode: None,
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

    pub fn new_with_id(name: impl Into<Cow<'static, str>>) -> Self {
        let name = name.into();
        Node {
            display_name: name.clone(),
            id: Some(name.into()),
            tooltip: None,
            description: None,
            visibility: None,
            imposed_access_mode: None,
            access_mode: None,
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

#[derive(Debug, Clone)]
pub enum Visibility {
    Beginner,
    Expert,
    Guru,
    Invisible,
}

#[derive(Debug, Clone)]
pub enum AccessMode {
    ReadOnly,
    WriteOnly,
    ReadWrite,
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
        NodeVariant::Property(PropertyVariant::Boolean(prop))
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
    Boolean(BoolProperty),
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
    pub children: Cow<'static, [Child]>,
}

#[derive(Clone)]
#[derive(EnumAsInner)]
//#[derive(Serialize, Deserialize)]
pub enum Child {
    Node(Node),
    Ref(NodeId),
}

impl From<Node> for Child {
    fn from(node: Node) -> Self {
        Child::Node(node)
    }
}

impl From<NodeId> for Child {
    fn from(id: NodeId) -> Self {
        Child::Ref(id)
    }
}

impl Debug for Child {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Child::Node(node) => node.fmt(f),
            Child::Ref(id) => write!(f, "Ref({:?})", id),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BoolProperty {
    pub value: Option<bool>,
    pub value_ref: Option<NodeId>, // TODO unify with a variant, maybe ValueOrRef or something similar to also replace Child
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
pub struct EnumProperty {
    pub value: Option<i64>,
    pub value_ref: Option<NodeId>, // TODO unify with a variant, maybe ValueOrRef or something similar to also replace Child
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
pub struct NumericValue<T> {
    pub current: T,
    pub range: RangeInclusive<T>, // TODO support not range
}

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
pub struct EnumEntry {
    pub discriminant: i64,
    pub name: Cow<'static, str>,
    pub help: Option<Cow<'static, str>>,
}

#[derive(Debug, Clone)]
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
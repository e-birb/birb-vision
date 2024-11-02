use std::{borrow::Cow, fmt::Debug, ops::{Deref, DerefMut}};

use enum_as_inner::EnumAsInner;
use serde::{Deserialize, Serialize};

mod id;
mod property;

pub use id::*;
pub use property::*;

#[derive(Debug, Clone)]
#[derive(EnumAsInner)]
pub enum Node {
    Group(GroupNode),
    Property(Property),
    // TODO maybe is a property? Port, // TODO 
    // TODO other meta nodes? like a "description" node?
}

impl Deref for Node {
    type Target = NodeInfo;

    fn deref(&self) -> &Self::Target {
        match self {
            Node::Group(group) => &group.info,
            Node::Property(prop) => prop.deref(),
        }
    }
}

impl DerefMut for Node {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Node::Group(group) => &mut group.info,
            Node::Property(prop) => prop.deref_mut(),
        }
    }
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Serialize, Deserialize)]
pub enum Visibility {
    Beginner,
    Expert,
    Guru,
    Invisible,
}

#[derive(Debug, Clone)]
pub struct GroupNode {
    pub info: NodeInfo,
    pub children: Cow<'static, [NodeId]>,
}

impl Deref for GroupNode {
    type Target = NodeInfo;

    fn deref(&self) -> &Self::Target {
        &self.info
    }
}

impl DerefMut for GroupNode {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.info
    }
}

impl GroupNode {
    pub fn new(id: impl Into<NodeId>) -> Self {
        Self {
            info: NodeInfo::new_with_id(id),
            children: Cow::Borrowed(&[]),
        }
    }
}

impl From<GroupNode> for Node {
    fn from(group: GroupNode) -> Self {
        Node::Group(group)
    }
}

impl From<BoolProperty> for Node {
    fn from(prop: BoolProperty) -> Self {
        Node::Property(Property::Bool(prop))
    }
}

impl From<Property> for Node {
    fn from(prop: Property) -> Self {
        Node::Property(prop)
    }
}

impl From<EnumProperty> for Node {
    fn from(prop: EnumProperty) -> Self {
        Node::Property(Property::Enum(prop))
    }
}

impl From<StringProperty> for Node {
    fn from(prop: StringProperty) -> Self {
        Node::Property(Property::String(prop))
    }
}
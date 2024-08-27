use std::{borrow::Cow, fmt::{Debug, Formatter}};

use enum_as_inner::EnumAsInner;
use serde::{Deserialize, Serialize};

use super::Node;


#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Serialize, Deserialize)]
#[derive(EnumAsInner)]
pub enum NodeId {
    String(Cow<'static, str>),
    I32(i32), // TODO maybe 64 or more? Or maybe all in types + Arc<dyn Any + Hash + ...>
}
impl NodeId {
    pub fn as_str(&self) -> Option<&str> {
        self.as_string().map(|s| s.as_ref())
    }
}

impl From<&'static str> for NodeId {
    fn from(s: &'static str) -> Self {
        NodeId::String(Cow::Borrowed(s))
    }
}

impl From<String> for NodeId {
    fn from(s: String) -> Self {
        NodeId::String(Cow::Owned(s))
    }
}

impl From<Cow<'static, str>> for NodeId {
    fn from(s: Cow<'static, str>) -> Self {
        NodeId::String(s)
    }
}

impl From<i32> for NodeId {
    fn from(value: i32) -> Self {
        NodeId::I32(value)
    }
}

impl Debug for NodeId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeId::String(s) => s.fmt(f),
            NodeId::I32(n) => n.fmt(f),
        }
    }
}
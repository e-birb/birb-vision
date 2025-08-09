use std::{borrow::Cow, fmt::{Debug, Formatter}};

use anyhow::anyhow;
use enum_as_inner::EnumAsInner;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub use serde_json::Value as Json;
pub use serde_json;

use crate::DeviceResult;

/// A unique identifier for a node in the device property tree.
#[derive(Clone, PartialEq, Eq, Hash)] // TODO PartialOrd, Ord
#[derive(Serialize, Deserialize)]
#[derive(EnumAsInner)]
#[non_exhaustive]
pub enum NodeId {
    String(Cow<'static, str>),
    I32(i32), // TODO maybe 64 or more? Or maybe all in types + Arc<dyn Any + Hash + ...>
    Json(Json),
}

impl NodeId {
    pub fn as_str(&self) -> Option<&str> {
        self.as_string().map(|s| s.as_ref())
    }

    pub fn try_serialyze_value<T: Serialize>(value: T) -> DeviceResult<Self> {
        let json = serde_json::to_value(value)
            .map_err(|e| anyhow!("Failed to serialize NodeId: {e}"))?;
        Ok(NodeId::Json(json))
    }

    pub fn try_deserialize_value<T: DeserializeOwned>(self) -> DeviceResult<T> {
        let NodeId::Json(json) = self else {
            return Err(anyhow!("Invalid NodeId: {self:?}").into());
        };
        let value = serde_json::from_value(json)
            .map_err(|e| anyhow!("Failed to deserialize NodeId: {e}"))?;
        Ok(value)
    }
}

impl From<Json> for NodeId {
    fn from(value: Json) -> Self {
        NodeId::Json(value)
    }
}

impl From<&'static str> for NodeId {
    fn from(s: &'static str) -> Self {
        NodeId::String(s.into())
    }
}

impl From<String> for NodeId {
    fn from(s: String) -> Self {
        NodeId::String(s.into())
    }
}

impl From<Cow<'static, str>> for NodeId {
    fn from(s: Cow<'static, str>) -> Self {
        NodeId::String(s.into())
    }
}

impl From<i32> for NodeId {
    fn from(value: i32) -> Self {
        NodeId::I32(value.into())
    }
}

impl Debug for NodeId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeId::String(s) => s.fmt(f),
            NodeId::I32(n) => n.fmt(f),
            NodeId::Json(j) => j.fmt(f),
        }
    }
}
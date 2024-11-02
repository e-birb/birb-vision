use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::NodeId;

use super::ControlInfo;


#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct StringProperty {
    pub info: ControlInfo,
    pub max_length: u32,
    pub default: Option<String>,
}

impl Deref for StringProperty {
    type Target = ControlInfo;

    fn deref(&self) -> &Self::Target {
        &self.info
    }
}

impl DerefMut for StringProperty {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.info
    }
}

impl StringProperty {
    pub fn new(id: impl Into<NodeId>) -> Self {
        Self {
            info: ControlInfo::new(id),
            max_length: std::u32::MAX,
            default: None,
        }
    }

    pub fn new_const(id: impl Into<NodeId>, value: impl Into<String>) -> Self {
        let default: String = value.into();
        Self {
            info: ControlInfo::new_const(id),
            max_length: default.len() as u32,
            default: Some(default),
        }
    }
}
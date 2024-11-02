use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::NodeId;

use super::{ControlInfo, ValueOrRef};


#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct BoolProperty {
    pub info: ControlInfo,
    pub value: Option<ValueOrRef<bool>>,
    pub default: Option<bool>,
}

impl Deref for BoolProperty {
    type Target = ControlInfo;

    fn deref(&self) -> &Self::Target {
        &self.info
    }
}

impl DerefMut for BoolProperty {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.info
    }
}

impl BoolProperty {
    pub fn new(id: impl Into<NodeId>) -> Self {
        Self {
            info: ControlInfo::new(id),
            value: None,
            default: None,
        }
    }
}
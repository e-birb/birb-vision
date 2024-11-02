use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::NodeId;

use super::ControlInfo;



#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct CommandProperty {
    pub info: ControlInfo,
}

impl Deref for CommandProperty {
    type Target = ControlInfo;

    fn deref(&self) -> &Self::Target {
        &self.info
    }
}

impl DerefMut for CommandProperty {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.info
    }
}

impl CommandProperty {
    pub fn new(id: impl Into<NodeId>) -> Self {
        Self {
            info: ControlInfo::new(id),
        }
    }
}
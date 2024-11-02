use std::{borrow::Cow, ops::{Deref, DerefMut}};

use serde::{Deserialize, Serialize};

use crate::NodeId;

use super::{ControlInfo, ValueOrRef};


#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct EnumProperty {
    pub info: ControlInfo,
    pub value: Option<ValueOrRef<i64>>,
    pub entries: Cow<'static, [EnumEntry]>,
}

impl Deref for EnumProperty {
    type Target = ControlInfo;

    fn deref(&self) -> &Self::Target {
        &self.info
    }
}

impl DerefMut for EnumProperty {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.info
    }
}

impl EnumProperty {
    pub fn new(id: impl Into<NodeId>) -> Self {
        Self {
            info: ControlInfo::new(id),
            value: None,
            entries: Cow::Borrowed(&[]),
        }
    }
}

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct EnumState<'a> {
    pub current: i64,
    pub support: Cow<'a, [i64]>,
}

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct EnumEntry {
    pub discriminant: i64,
    pub name: Cow<'static, str>,
    pub help: Option<Cow<'static, str>>,
}
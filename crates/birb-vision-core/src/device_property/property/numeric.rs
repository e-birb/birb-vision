use std::{borrow::Cow, ops::{Deref, DerefMut, RangeInclusive}};

use serde::{Deserialize, Serialize};

use crate::NodeId;

use super::{ControlInfo, ValueOrRef};


#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct NumericProperty<T> {
    pub info: ControlInfo,
    pub value: Option<T>,
    pub min: Option<ValueOrRef<T>>,
    pub max: Option<ValueOrRef<T>>,
    pub increment: Option<ValueOrRef<T>>,
    pub default: Option<T>,
    pub unit: Option<Cow<'static, str>>,
    pub slope: Slope,
    pub representation: Option<Representation>,
}

impl<T> Deref for NumericProperty<T> {
    type Target = ControlInfo;

    fn deref(&self) -> &Self::Target {
        &self.info
    }
}

impl<T> DerefMut for NumericProperty<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.info
    }
}

impl<T> NumericProperty<T> {
    pub fn new(id: impl Into<NodeId>) -> Self {
        Self {
            info: ControlInfo::new(id),
            value: None,
            min: None,
            max: None,
            increment: None,
            default: None,
            unit: None,
            slope: Slope::Increasing,
            representation: None,
        }
    }
}

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct NumericState<T> {
    pub current: T,
    pub range: RangeInclusive<T>, // TODO support not range
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

// TODO use this
//#[derive(Debug, Clone)]
//pub enum NumericSupport<T: Clone + 'static> {
//    Range(RangeInclusive<T>),
//    Set(Cow<'static, [T]>)
//}
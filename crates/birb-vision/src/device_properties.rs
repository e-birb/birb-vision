use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Node {
    //Group(NodeList),
    //Text(String),
    pub friendly_name: String,
    pub id: Option<PropertyId>,
    pub help: Option<String>,
    pub variant: NodeVariant,
}

#[derive(Debug, Clone)]
pub enum NodeVariant {
    Group(GroupNode),
}

#[derive(Debug, Clone)]
pub struct GroupNode {
    pub name: String,
    pub comment: Option<String>,
    pub children: NodeList,
}

#[derive(Debug, Clone)]
pub struct NodeList {
    pub nodes: Vec<Arc<Node>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PropertyId(String);

impl PropertyId {
    pub fn from_str(s: &str) -> Self {
        PropertyId(s.to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for PropertyId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for PropertyId {
    fn from(s: &str) -> Self {
        PropertyId(s.to_string())
    }
}

impl From<String> for PropertyId {
    fn from(s: String) -> Self {
        PropertyId(s)
    }
}

#[derive(Debug, Clone)]
pub enum PropertyValue {
    Bool(BoolValue),
    Int(IntValue),
    Float(FloatValue),
}

#[derive(Debug, Clone)]
pub struct BoolValue {
    pub value: bool,
}

#[derive(Debug, Clone)]
pub struct IntValue {
    pub value: i64,
    pub min: Option<i64>,
    pub max: Option<i64>,
    pub step: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct FloatValue {
    pub value: f64,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub step: Option<f64>,
}
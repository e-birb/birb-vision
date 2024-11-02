use std::borrow::Cow;

use birb_vision_core::{AccessMode, BoolProperty, CommandProperty, EnumEntry, EnumProperty, GroupNode, Node, NodeId, NumericProperty, Property, Representation, StringProperty, ValueOrRef, Visibility};
use roxmltree::Node as XmlNode;

pub const ROOT_ID: NodeId = NodeId::String(Cow::Borrowed("Camera actual Root")); // TODO unnecessary?
pub const USER_ROOT_ID: NodeId = NodeId::String(Cow::Borrowed("Root"));

fn parse_child_or_property(xml_node: XmlNode, current: &mut Node, list: &mut Vec<Node>) -> Option<NodeId> {
    fn todo_node(xml_node: XmlNode) -> Option<NodeId> {
        log::warn!("TODO: {}", xml_node.tag_name().name());
        None
    }

    let append = |node: Node, list: &mut Vec<Node>| {
        let id = node.id.clone();
        list.push(node);
        id
    };

    match xml_node.tag_name().name() {
        "" => None,
        "RegisterDescription" => Some(append(parse_root(xml_node, ROOT_ID, list), list)),
        "Category" | "Group" => Some(append(parse_group(xml_node, list), list)),
        "pFeature" => Some(NodeId::String(xml_node.text().unwrap().to_string().into())),
        "Port" => Some(append(parse_port(xml_node, list), list)),
        "Integer" => Some(append(parse_integer(xml_node, list), list)),
        "Float" => Some(append(parse_float(xml_node, list), list)),
        "IntReg" => todo_node(xml_node),
        "StructReg" => todo_node(xml_node),
        "Enumeration" => Some(append(parse_enum(xml_node, list), list)),
        "StringReg" => Some(append(parse_string(xml_node, list), list)),
        "Boolean" => Some(append(parse_bool(xml_node, list), list)),
        //"Command" => Some(Child::Node(parse_command(xml_node))),
        "MaskedIntReg" => todo_node(xml_node),
        "IntSwissKnife" => todo_node(xml_node),
        "SwissKnife" => todo_node(xml_node),
        "Converter" => todo_node(xml_node),
        "Register" => todo_node(xml_node), // !!!
        "Command" => Some(append(parse_command(xml_node, list), list)),

        "ToolTip" => {
            current.tooltip = Some(xml_node.text().unwrap().to_string().into());
            None
        },
        "Description" => {
            current.description = Some(xml_node.text().unwrap().to_string().into());
            None
        },
        "DisplayName" => {
            current.display_name = xml_node.text().unwrap().to_string().into();
            None
        },
        "Visibility" => {
            let value = match xml_node.text().unwrap() {
                "Beginner" => Visibility::Beginner,
                "Expert" => Visibility::Expert,
                "Guru" => Visibility::Guru,
                "Invisible" => Visibility::Invisible,
                other => {
                    todo!("Unknown visibility: {}", other);
                },
            };
            current.visibility = Some(value);
            None
        },
        "ImposedAccessMode" | "AccessMode" => {
            let value = match xml_node.text().unwrap() {
                "RW" => AccessMode::ReadWrite,
                "RO" => AccessMode::ReadOnly,
                "WO" => AccessMode::WriteOnly,
                other => {
                    todo!("Unknown access mode: {}", other);
                },
            };
            // TODO maybe imposed access mode should have priority
            if let Node::Property(current) = current {
                current.access_mode = value;
            } else {
                log::warn!("Non property node ({:?}) cannot have access mode", current.id);
            }
            None
        },
        "pIsImplemented" => {
            // TODO
            None
        },
        "pIsLocked" => {
            if let Node::Property(current) = current {
                current.is_locked_ref = Some(xml_node.text().unwrap().to_string().into());
            } else {
                log::warn!("Non property node ({:?}) cannot have is_locked_ref", current.id);
            }
            None
        },
        "Address" => {
            if let Node::Property(current) = current {
                current.address = Some(ValueOrRef::Value(0)); // TODO parse
            } else {
                log::warn!("Non property node ({:?}) cannot have address", current.id);
            }
            None
        },
        "pAddress" => {
            if let Node::Property(current) = current {
                current.address = Some(ValueOrRef::Ref(xml_node.text().unwrap().to_string().into()));
            } else {
                log::warn!("Non property node ({:?}) cannot have address", current.id);
            }
            None
        },
        "pPort" => {
            if let Node::Property(current) = current {
                current.port_ref = Some(xml_node.text().unwrap().to_string().into());
            } else {
                log::warn!("Non property node ({:?}) cannot have port_ref", current.id);
            }
            None
        },
        "Streamable" => {
            if let Node::Property(current) = current {
                current.streamable = true;
            } else {
                log::warn!("Non property node ({:?}) cannot have streamable", current.id);
            }
            None
        },
        "pIsAvailable" => {
            // TODO
            None
        },
        "pSelected" => {
            // TODO
            None
        },
        "IntConverter" => {
            // TODO
            None
        },

        other => {
            //return None;
            log::error!("Unknown node type: {} inside {}", other, current.display_name);
            #[cfg(debug_assertions)]
            panic!("Unknown node type: {}", other);
            #[cfg(not(debug_assertions))]
            return None;
        },
    }
}

pub fn parse_root(xml_root: XmlNode, root_id: NodeId, list: &mut Vec<Node>) -> Node {
    let mut node: Node = GroupNode::new(root_id).into();

    let mut children = Vec::new();
    for child in xml_root.children() {
        if let Some(child_node) = parse_child_or_property(child, &mut node, list) {
            children.push(child_node);
        }
    }

    node.as_group_mut().unwrap().children = children.into();

    node
}

fn parse_group(xml_node: XmlNode, list: &mut Vec<Node>) -> Node {
    let name = xml_node.attribute("Name").map(|n| n.to_string()).unwrap_or_else(|| "comment=".to_string() + &xml_node.attribute("Comment").unwrap()).to_string();
    println!("Group NAME: {}", name);
    let mut node: Node = GroupNode::new(name).into();

    let mut children = Vec::new();
    for child in xml_node.children() {
        if let Some(child_node) = parse_child_or_property(child, &mut node, list) {
            children.push(child_node);
        }
    }

    node.as_group_mut().unwrap().children = children.into();

    node
}

fn parse_port(xml_node: XmlNode, list: &mut Vec<Node>) -> Node {
    let name = xml_node.attribute("Name").unwrap().to_string();
    let mut node: Node = GroupNode::new(name).into();

    for child in xml_node.children() {
        match child.tag_name().name() {
            "" => {},
            "EventID" => {
                // TODO
                continue;
            },
            _ => {},
        }

        if parse_child_or_property(child, &mut node, list).is_some() {
            panic!("Port cannot have children");
        }
    }

    node
}

fn parse_int_value_from_string(string: &str) -> i64 {
    if string.starts_with("0x") {
        i64::from_str_radix(&string[2..], 16).unwrap()
    } else {
        string.parse().unwrap()
    }
}

fn parse_int_value(xml_node: XmlNode) -> i64 {
    let string = xml_node.text().unwrap();
    parse_int_value_from_string(string)
}

pub fn parse_integer(xml_node: XmlNode, list: &mut Vec<Node>) -> Node {
    let name = xml_node.attribute("Name").unwrap().to_string();
    let is_exposure = name == "ExposureTime";
    let mut node: Node = Property::Integer(NumericProperty::new(name)).into();

    for child in xml_node.children() {
        match child.tag_name().name() {
            "" => {},
            "Unit" => {
                node.as_property_mut().unwrap().as_integer_mut().unwrap().unit = Some(child.text().unwrap().to_string().into());
                continue;
            },
            "Value" => {
                node.as_property_mut().unwrap().as_integer_mut().unwrap().value = Some(parse_int_value(child));
                continue;
            }
            "pValue" => {
                // TODO
                continue;
            },
            "Representation" => {
                let repr = match child.text().unwrap() {
                    "Boolean" => Representation::Boolean,
                    "HexNumber" => Representation::Hex,
                    "Linear" => if is_exposure { Representation::Logarithmic } else { Representation::Linear },
                    "Logarithmic" => Representation::Logarithmic,
                    "PureNumber" => Representation::PureNumber,
                    other => {
                        todo!("Unknown representation: {:?}", other);
                    },
                };
                node.as_property_mut().unwrap().as_integer_mut().unwrap().representation = Some(repr);
                continue;
            },
            "Min" => {
                node.as_property_mut().unwrap().as_integer_mut().unwrap().min = Some(ValueOrRef::Value(parse_int_value(child)));
                continue;
            },
            "pMin" => {
                node.as_property_mut().unwrap().as_integer_mut().unwrap().min = Some(ValueOrRef::Ref(child.text().unwrap().to_string().into()));
                continue;
            },
            "Max" => {
                node.as_property_mut().unwrap().as_integer_mut().unwrap().max = Some(ValueOrRef::Value(parse_int_value(child)));
                continue;
            },
            "pMax" => {
                node.as_property_mut().unwrap().as_integer_mut().unwrap().max = Some(ValueOrRef::Ref(child.text().unwrap().to_string().into()));
                continue;
            },
            "Inc" => {
                node.as_property_mut().unwrap().as_integer_mut().unwrap().increment = Some(ValueOrRef::Value(parse_int_value(child)));
                continue;
            },
            "pInc" => {
                node.as_property_mut().unwrap().as_integer_mut().unwrap().increment = Some(ValueOrRef::Ref(child.text().unwrap().to_string().into()));
                continue;
            },
            _ => {},
        }

        if parse_child_or_property(child, &mut node, list).is_some() {
            panic!("Integer cannot have children");
        }
    }

    node
}

pub fn parse_float(xml_node: XmlNode, list: &mut Vec<Node>) -> Node {
    let name = xml_node.attribute("Name").unwrap().to_string();
    let is_exposure = name == "ExposureTime";
    let mut node: Node = Property::Float(NumericProperty::new(name)).into();

    for child in xml_node.children() {
        match child.tag_name().name() {
            "" => {},
            "Unit" => {
                node.as_property_mut().unwrap().as_float_mut().unwrap().unit = Some(child.text().unwrap().to_string().into());
                continue;
            },
            "Value" => {
                node.as_property_mut().unwrap().as_float_mut().unwrap().value = Some(child.text().unwrap().parse().unwrap());
                continue;
            }
            "pValue" => {
                // TODO
                continue;
            },
            "Representation" => {
                let repr = match child.text().unwrap() {
                    "Boolean" => Representation::Boolean,
                    "HexNumber" => Representation::Hex,
                    "Linear" => if is_exposure { Representation::Logarithmic } else { Representation::Linear },
                    "Logarithmic" => Representation::Logarithmic,
                    "PureNumber" => Representation::PureNumber,
                    other => {
                        todo!("Unknown representation: {:?}", other);
                    },
                };
                node.as_property_mut().unwrap().as_float_mut().unwrap().representation = Some(repr);
                continue;
            },
            "Min" => {
                node.as_property_mut().unwrap().as_float_mut().unwrap().min = Some(ValueOrRef::Value(child.text().unwrap().parse().unwrap()));
                continue;
            },
            "pMin" => {
                node.as_property_mut().unwrap().as_float_mut().unwrap().min = Some(ValueOrRef::Ref(child.text().unwrap().to_string().into()));
                continue;
            },
            "Max" => {
                node.as_property_mut().unwrap().as_float_mut().unwrap().max = Some(ValueOrRef::Value(child.text().unwrap().parse().unwrap()));
                continue;
            },
            "pMax" => {
                node.as_property_mut().unwrap().as_float_mut().unwrap().max = Some(ValueOrRef::Ref(child.text().unwrap().to_string().into()));
                continue;
            },
            "Inc" => {
                node.as_property_mut().unwrap().as_float_mut().unwrap().increment = Some(ValueOrRef::Value(child.text().unwrap().parse().unwrap()));
                continue;
            },
            "pInc" => {
                node.as_property_mut().unwrap().as_float_mut().unwrap().increment = Some(ValueOrRef::Ref(child.text().unwrap().to_string().into()));
                continue;
            },
            _ => {},
        }

        if parse_child_or_property(child, &mut node, list).is_some() {
            panic!("Integer cannot have children");
        }
    }

    node
}

pub fn parse_enum(xml_node: XmlNode, list: &mut Vec<Node>) -> Node {
    let name = xml_node.attribute("Name").unwrap().to_string();
    let mut node: Node = Property::Enum(EnumProperty::new(name)).into();

    let mut entries = Vec::new();
    for child in xml_node.children() {
        match child.tag_name().name() {
            "" => {},
            "Value" => {
                node.as_property_mut().unwrap().as_enum_mut().unwrap().value = Some(ValueOrRef::Value(parse_int_value(child)));
                continue;
            },
            "pValue" => {
                node.as_property_mut().unwrap().as_enum_mut().unwrap().value = Some(ValueOrRef::Ref(child.text().unwrap().to_string().into()));
                continue;
            },
            "EnumEntry" => {
                let name = child.attribute("Name").unwrap().to_string();
                let value = child.children().filter(|c| c.tag_name().name() == "Value").next().unwrap();
                let value = parse_int_value(value);
                let entry = EnumEntry {
                    discriminant: value,
                    name: name.into(),
                    help: None,
                };
                entries.push(entry);
                continue;
            }
            _ => {},
        }

        if parse_child_or_property(child, &mut node, list).is_some() {
            panic!("Integer cannot have children");
        }
    }

    node.as_property_mut().unwrap().as_enum_mut().unwrap().entries = entries.into();
    node
}

fn parse_string(xml_node: XmlNode, list: &mut Vec<Node>) -> Node {
    let name = xml_node.attribute("Name").unwrap().to_string();
    let mut node: Node = Property::String(StringProperty::new(name)).into();

    for child in xml_node.children() {
        match child.tag_name().name() {
            "Length" => {
                node.as_property_mut().unwrap().as_string_mut().unwrap().max_length = parse_int_value(child) as _;
                continue;
            },
            _ => {},
        }

        if parse_child_or_property(child, &mut node, list).is_some() {
            panic!("Integer cannot have children");
        }
    }

    node
}

fn parse_bool(xml_node: XmlNode, list: &mut Vec<Node>) -> Node {
    let name = xml_node.attribute("Name").unwrap().to_string();
    let mut node: Node = Property::Bool(BoolProperty::new(name)).into();

    for child in xml_node.children() {
        match child.tag_name().name() {
            "" => {},
            "Value" => {
                panic!("TODO: parse bool value {:?}", child.text().unwrap());
                node.as_property_mut().unwrap().as_bool_mut().unwrap().value = Some(ValueOrRef::Value(false)); // TODO parse
                continue;
            }
            "pValue" => {
                node.as_property_mut().unwrap().as_bool_mut().unwrap().value = Some(ValueOrRef::Ref(child.text().unwrap().to_string().into()));
                continue;
            },
            "OnValue" => {
                // TODO
                continue;
            },
            "OffValue" => {
                // TODO
                continue;
            },
            _ => {},
        }

        if parse_child_or_property(child, &mut node, list).is_some() {
            panic!("Integer cannot have children");
        }
    }

    node
}

fn parse_command(xml_node: XmlNode, list: &mut Vec<Node>) -> Node {
    let name = xml_node.attribute("Name").unwrap().to_string();
    let mut node: Node = Property::Command(CommandProperty::new(name)).into();

    for child in xml_node.children() {
        match child.tag_name().name() {
            "CommandValue" => {
                // TODO
                continue;
            },
            "pCommandValue" => {
                // TODO
                continue;
            },
            "Value" => {
                // TODO
                continue;
            },
            "pValue" => {
                // TODO
                continue;
            },
            _ => {},
        }

        if parse_child_or_property(child, &mut node, list).is_some() {
            panic!("Integer cannot have children");
        }
    }

    node
}
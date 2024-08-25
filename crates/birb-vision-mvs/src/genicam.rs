use birb_vision::{AccessMode, BoolProperty, Child, EnumEntry, EnumProperty, GroupNode, Node, NodeId, NodeVariant, NumericProperty, PropertyVariant, Representation, StringProperty, Visibility};
use roxmltree::Node as XmlNode;

fn parse_child_or_property(xml_node: XmlNode, current: &mut Node) -> Option<Child> {
    fn todo_node(xml_node: XmlNode) -> Option<Child> {
        log::warn!("TODO: {}", xml_node.tag_name().name());
        None
    }

    match xml_node.tag_name().name() {
        "" => None,
        "RegisterDescription" => Some(Child::Node(parse_root(xml_node))),
        "Category" | "Group" => Some(Child::Node(parse_group(xml_node))),
        "pFeature" => Some(Child::Ref(xml_node.text().unwrap().to_string().into())),
        "Port" => Some(Child::Node(parse_port(xml_node))),
        "Integer" => Some(Child::Node(parse_integer(xml_node))),
        "Float" => Some(Child::Node(parse_float(xml_node))),
        "IntReg" => todo_node(xml_node),
        "StructReg" => todo_node(xml_node),
        "Enumeration" => Some(Child::Node(parse_enum(xml_node))),
        "StringReg" => Some(Child::Node(parse_string(xml_node))),
        "Boolean" => Some(Child::Node(parse_bool(xml_node))),
        //"Command" => Some(Child::Node(parse_command(xml_node))),
        "MaskedIntReg" => todo_node(xml_node),
        "IntSwissKnife" => todo_node(xml_node),
        "SwissKnife" => todo_node(xml_node),
        "Converter" => todo_node(xml_node),
        "Register" => todo_node(xml_node), // !!!
        "Command" => Some(Child::Node(parse_command(xml_node))),

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
        "ImposedAccessMode" => {
            let value = match xml_node.text().unwrap() {
                "RW" => AccessMode::ReadWrite,
                "RO" => AccessMode::ReadOnly,
                "WO" => AccessMode::WriteOnly,
                other => {
                    todo!("Unknown access mode: {}", other);
                },
            };
            current.imposed_access_mode = Some(value);
            None
        },
        "AccessMode" => {
            let value = match xml_node.text().unwrap() {
                "RW" => AccessMode::ReadWrite,
                "RO" => AccessMode::ReadOnly,
                "WO" => AccessMode::WriteOnly,
                other => {
                    todo!("Unknown access mode: {}", other);
                },
            };
            current.access_mode = Some(value);
            None
        },
        "pIsImplemented" => {
            // TODO
            None
        },
        "pIsLocked" => {
            current.is_locked_ref = Some(NodeId(xml_node.text().unwrap().to_string().into()));
            None
        },
        "Address" => {
            current.address = Some(0); // TODO parse
            None
        },
        "pAddress" => {
            current.address_ref = Some(NodeId(xml_node.text().unwrap().to_string().into()));
            None
        },
        "pPort" => {
            current.port_ref = Some(NodeId(xml_node.text().unwrap().to_string().into()));
            None
        },
        "Streamable" => {
            current.streamable = true;
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

pub fn parse_root(xml_node: XmlNode) -> Node {
    let mut node = Node::new("Device");

    let mut children = Vec::new();
    for child in xml_node.children() {
        if let Some(child_node) = parse_child_or_property(child, &mut node) {
            children.push(child_node);
        }
    }

    node.variant = GroupNode {
        children: children.into(),
    }.into();
    node
}

fn parse_group(xml_node: XmlNode) -> Node {
    let name = xml_node.attribute("Name").map(|n| n.to_string()).unwrap_or_else(|| "comment=".to_string() + &xml_node.attribute("Comment").unwrap()).to_string();
    let mut node = Node::new_with_id(name);

    let mut children = Vec::new();
    for child in xml_node.children() {
        if let Some(child_node) = parse_child_or_property(child, &mut node) {
            children.push(child_node);
        }
    }

    node.variant = GroupNode {
        children: children.into(),
    }.into();

    node
}

fn parse_port(xml_node: XmlNode) -> Node {
    let name = xml_node.attribute("Name").unwrap().to_string();
    let mut node = Node::new_with_id(name);

    for child in xml_node.children() {
        match child.tag_name().name() {
            "" => {},
            "EventID" => {
                // TODO
                continue;
            },
            _ => {},
        }

        if parse_child_or_property(child, &mut node).is_some() {
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

pub fn parse_integer(xml_node: XmlNode) -> Node {
    let name = xml_node.attribute("Name").unwrap().to_string();
    let is_exposure = name == "ExposureTime";
    let mut node = Node::new_with_id(name);
    let mut prop = NumericProperty::default();

    for child in xml_node.children() {
        match child.tag_name().name() {
            "" => {},
            "Unit" => {
                prop.unit = Some(child.text().unwrap().to_string().into());
                continue;
            },
            "Value" => {
                prop.value = Some(parse_int_value(child));
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
                prop.representation = Some(repr);
                continue;
            },
            "Min" => {
                prop.min = Some(parse_int_value(child));
                continue;
            },
            "pMin" => {
                prop.min_ref = Some(NodeId(child.text().unwrap().to_string().into()));
                continue;
            },
            "Max" => {
                prop.max = Some(parse_int_value(child));
                continue;
            },
            "pMax" => {
                prop.max_ref = Some(NodeId(child.text().unwrap().to_string().into()));
                continue;
            },
            "Inc" => {
                prop.increment = Some(parse_int_value(child));
                continue;
            },
            "pInc" => {
                prop.increment_ref = Some(NodeId(child.text().unwrap().to_string().into()));
                continue;
            },
            _ => {},
        }

        if parse_child_or_property(child, &mut node).is_some() {
            panic!("Integer cannot have children");
        }
    }

    node.variant = NodeVariant::Property(PropertyVariant::Integer(prop));
    node
}

pub fn parse_float(xml_node: XmlNode) -> Node {
    let name = xml_node.attribute("Name").unwrap().to_string();
    let is_exposure = name == "ExposureTime";
    let mut node = Node::new_with_id(name);
    let mut prop = NumericProperty::<f64>::default();

    for child in xml_node.children() {
        match child.tag_name().name() {
            "" => {},
            "Unit" => {
                prop.unit = Some(child.text().unwrap().to_string().into());
                continue;
            },
            "Value" => {
                prop.value = Some(child.text().unwrap().parse().unwrap());
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
                prop.representation = Some(repr);
                continue;
            },
            "Min" => {
                prop.min = Some(child.text().unwrap().parse().unwrap());
                continue;
            },
            "pMin" => {
                prop.min_ref = Some(NodeId(child.text().unwrap().to_string().into()));
                continue;
            },
            "Max" => {
                prop.max = Some(child.text().unwrap().parse().unwrap());
                continue;
            },
            "pMax" => {
                prop.max_ref = Some(NodeId(child.text().unwrap().to_string().into()));
                continue;
            },
            "Inc" => {
                prop.increment = Some(child.text().unwrap().parse().unwrap());
                continue;
            },
            "pInc" => {
                prop.increment_ref = Some(NodeId(child.text().unwrap().to_string().into()));
                continue;
            },
            _ => {},
        }

        if parse_child_or_property(child, &mut node).is_some() {
            panic!("Integer cannot have children");
        }
    }

    node.variant = NodeVariant::Property(PropertyVariant::Float(prop));
    node
}

pub fn parse_enum(xml_node: XmlNode) -> Node {
    let name = xml_node.attribute("Name").unwrap().to_string();
    let mut node = Node::new_with_id(name);
    let mut variant = EnumProperty {
        value: None,
        value_ref: None,
        entries: vec![].into(),
    };

    let mut entries = Vec::new();
    for child in xml_node.children() {
        match child.tag_name().name() {
            "" => {},
            "Value" => {
                variant.value = Some(parse_int_value(child));
                continue;
            },
            "pValue" => {
                variant.value_ref = Some(NodeId(child.text().unwrap().to_string().into()));
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

        if parse_child_or_property(child, &mut node).is_some() {
            panic!("Integer cannot have children");
        }
    }

    variant.entries = entries.into();
    node.variant = variant.into();
    node
}

fn parse_string(xml_node: XmlNode) -> Node {
    let name = xml_node.attribute("Name").unwrap().to_string();
    let mut node = Node::new_with_id(name);
    let mut prop = StringProperty::default();

    for child in xml_node.children() {
        match child.tag_name().name() {
            "Length" => {
                prop.max_length = parse_int_value(child) as _;
                continue;
            },
            _ => {},
        }

        if parse_child_or_property(child, &mut node).is_some() {
            panic!("Integer cannot have children");
        }
    }

    node.variant = NodeVariant::Property(PropertyVariant::String(prop));
    node
}

fn parse_bool(xml_node: XmlNode) -> Node {
    let name = xml_node.attribute("Name").unwrap().to_string();
    let mut node = Node::new_with_id(name);
    let mut prop = BoolProperty::default();

    for child in xml_node.children() {
        match child.tag_name().name() {
            "" => {},
            "Value" => {
                panic!("TODO: parse bool value {:?}", child.text().unwrap());
                prop.value = Some(false); // TODO parse
                continue;
            }
            "pValue" => {
                prop.value_ref = Some(NodeId(child.text().unwrap().to_string().into()));
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

        if parse_child_or_property(child, &mut node).is_some() {
            panic!("Integer cannot have children");
        }
    }

    node.variant = NodeVariant::Property(PropertyVariant::Boolean(prop));
    node
}

fn parse_command(xml_node: XmlNode) -> Node {
    let name = xml_node.attribute("Name").unwrap().to_string();
    let mut node = Node::new_with_id(name);

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

        if parse_child_or_property(child, &mut node).is_some() {
            panic!("Integer cannot have children");
        }
    }

    node.variant = NodeVariant::Property(PropertyVariant::Command);
    node
}
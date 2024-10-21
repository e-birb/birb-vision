use birb_vision_core::{anyhow::anyhow, BoolProperty, DeviceResult, EnumEntry, EnumProperty, EnumValue, Node, NodeVariant, NumericProperty, NumericValue, PropertyState, PropertyValue, PropertyVariant, Representation, StringProperty};
use v4l::control::{MenuItem, Type, Value};


pub fn parse(control: v4l::control::Description) -> Option<Node> {
    let mut node: Node = Node::new_with_id(control.id as i32);
    node.display_name = control.name.clone().into();
    let variant: Option<NodeVariant> = match control.typ {
        Type::Integer | Type::Integer64 => {
            let mut variant = NumericProperty::<i64>::default();
            variant.min = control.minimum.into();
            variant.max = control.maximum.into();
            variant.default = control.default.into();
            variant.increment = (control.step as i64).into();
            variant.representation = Some(if control.name.to_lowercase().starts_with("exposure") {
                // TODO maybe Representation::Logarithmic? It might depend on the camera... test different cameras
                Representation::Linear
            } else {
                Representation::Linear
            });
            Some(PropertyVariant::Integer(variant).into())
        },
        Type::Boolean => {
            let mut variant = BoolProperty::default();
            variant.default = Some(control.default != 0);
            Some(PropertyVariant::Bool(variant).into())
        },
        Type::Menu => {
            let mut variant = EnumProperty::default();
            variant.entries = control
                .items
                .as_ref().unwrap()
                .iter()
                .map(|(id, item)| {
                    let MenuItem::Name(name) = item else {
                        panic!("Expected name");
                    };
                    EnumEntry {
                        discriminant: *id as i64,
                        name: name.clone().into(),
                        help: None,
                    }
                })
                .collect::<Vec<_>>()
                .into();
            Some(PropertyVariant::Enum(variant).into())
        },
        Type::Button => {
            Some(PropertyVariant::Command.into())
        },
        Type::CtrlClass => {
            // TODO
            None
        },
        Type::String => {
            let variant = StringProperty::default();
            Some(PropertyVariant::String(variant).into())
        },
        Type::Bitmask => {
            // TODO
            None
        },
        Type::IntegerMenu => {
            // TODO
            None
        },
        Type::U8 | Type::U16 | Type::U32 => {
            // TODO
            None
        },
        Type::Area => {
            // TODO
            None
        },
    };
    variant.map(|variant| {
        node.variant = variant;
        node
    })
}

pub fn node_value_to_property_state(node: &Node, value: v4l::control::Value) -> DeviceResult<PropertyState> {
    match &node.variant {
        NodeVariant::Group(_) => Err(anyhow!("Cannot read a group node"))?,
        NodeVariant::Property(property) => match property {
            PropertyVariant::Bool(_) => match value {
                Value::Boolean(current) => Ok(PropertyState::Bool(current)),
                _ => Err(anyhow!("Expected boolean value but the current control value was {value:?}"))?,
            },
            PropertyVariant::Integer(property) => match value {
                Value::Integer(current) => Ok(PropertyState::Int(NumericValue {
                    current,
                    range: (property.min.unwrap_or(0))..=(property.max.unwrap_or(0)),
                })),
                _ => Err(anyhow!("Expected integer value but the current control value was {value:?}"))?,
            },
            PropertyVariant::Float(_) => unimplemented!("v4l does not support float properties"),
            PropertyVariant::Enum(property) => match value {
                Value::Integer(current) => Ok(PropertyState::Enum(EnumValue {
                    current,
                    support: property.entries.iter().map(|e| e.discriminant).collect::<Vec<_>>().into(),
                })),
                _ => Err(anyhow!("Expected integer value but the current control value was {value:?}"))?,
            },
            PropertyVariant::String(_) => match value {
                Value::String(current) => Ok(PropertyState::String(current)),
                _ => Err(anyhow!("Expected string value but the current control value was {value:?}"))?,
            },
            PropertyVariant::Command => Err(anyhow!("Cannot read a command property"))?,
        },
        NodeVariant::Port => todo!(),
    }
}

pub fn property_value_to_v4l(value: PropertyValue) -> DeviceResult<Value> {
    let value = match value {
        PropertyValue::Bool(value) => Value::Boolean(value),
        PropertyValue::Integer(value) => Value::Integer(value),
        PropertyValue::Float(_) => todo!(), // maybe unsupported?
        PropertyValue::Enum(value) => Value::Integer(value),
        PropertyValue::String(value) => Value::String(value.clone()),
        PropertyValue::Command => Value::None,
    };
    Ok(value)
}
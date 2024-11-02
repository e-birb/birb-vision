use birb_vision_core::{anyhow::anyhow, BoolProperty, CommandProperty, DeviceResult, EnumEntry, EnumProperty, EnumState, Node, NumericProperty, NumericState, Property, PropertyState, PropertyValue, Representation, StringProperty, ValueOrRef};
use v4l::control::{MenuItem, Type, Value};


pub fn parse(control: v4l::control::Description) -> Option<Node> {
    let id = control.id as i32;
    let node: Option<Node> = match control.typ {
        Type::Integer | Type::Integer64 => {
            let mut variant = NumericProperty::<i64>::new(id);
            variant.min = Some(ValueOrRef::Value(control.minimum));
            variant.max = Some(ValueOrRef::Value(control.maximum));
            variant.default = control.default.into();
            variant.increment = Some(ValueOrRef::Value(control.step as i64));
            variant.representation = Some(if control.name.to_lowercase().starts_with("exposure") {
                // TODO maybe Representation::Logarithmic? It might depend on the camera... test different cameras
                Representation::Linear
            } else {
                Representation::Linear
            });
            Some(Property::Integer(variant).into())
        },
        Type::Boolean => {
            let mut variant = BoolProperty::new(id);
            variant.default = Some(control.default != 0);
            Some(Property::Bool(variant).into())
        },
        Type::Menu => {
            let mut variant = EnumProperty::new(id);
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
            Some(Property::Enum(variant).into())
        },
        Type::Button => {
            Some(Property::Command(CommandProperty::new(id)).into())
        },
        Type::CtrlClass => {
            // TODO
            None
        },
        Type::String => {
            let variant = StringProperty::new(id);
            Some(Property::String(variant).into())
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
    let mut node = node?;
    node.display_name = control.name.clone().into();
    Some(node)
}

pub fn node_value_to_property_state(node: &Node, value: v4l::control::Value) -> DeviceResult<PropertyState> {
    match &node {
        Node::Group(_) => Err(anyhow!("Cannot read a group node"))?,
        Node::Property(property) => match property {
            Property::Bool(_) => match value {
                Value::Boolean(current) => Ok(PropertyState::Bool(current)),
                _ => Err(anyhow!("Expected boolean value but the current control value was {value:?}"))?,
            },
            Property::Integer(property) => match value {
                Value::Integer(current) => Ok(PropertyState::Int(NumericState {
                    current,
                    range: (*property.min.clone().unwrap_or(ValueOrRef::Value(0)).as_value().unwrap()..=*property.max.clone().unwrap_or(ValueOrRef::Value(0)).as_value().unwrap()),
                })),
                _ => Err(anyhow!("Expected integer value but the current control value was {value:?}"))?,
            },
            Property::Float(_) => unimplemented!("v4l does not support float properties"),
            Property::Enum(property) => match value {
                Value::Integer(current) => Ok(PropertyState::Enum(EnumState {
                    current,
                    support: property.entries.iter().map(|e| e.discriminant).collect::<Vec<_>>().into(),
                })),
                _ => Err(anyhow!("Expected integer value but the current control value was {value:?}"))?,
            },
            Property::String(_) => match value {
                Value::String(current) => Ok(PropertyState::String(current)),
                _ => Err(anyhow!("Expected string value but the current control value was {value:?}"))?,
            },
            Property::Command(_) => Err(anyhow!("Cannot read a command property"))?,
        },
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
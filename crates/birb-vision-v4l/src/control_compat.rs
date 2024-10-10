use birb_vision_core::{BoolProperty, EnumEntry, EnumProperty, Node, NodeVariant, NumericProperty, PropertyVariant, Representation, StringProperty};
use v4l::control::{MenuItem, Type};


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
            Some(PropertyVariant::Boolean(variant).into())
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
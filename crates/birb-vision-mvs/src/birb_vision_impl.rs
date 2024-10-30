

use std::borrow::Cow;

use birb_vision_core::anyhow::anyhow;
use birb_vision_core::{NodeVariant, PropertyState, PropertyValue, PropertyVariant};

use birb_vision_core::{CameraDevice, DeviceResult, EnumValue, StreamEvent, Sample, Node, NodeId, NumericValue};
use crate::ctx::convert_info;
use crate::genicam::{ROOT_ID, USER_ROOT_ID};
use crate::{genicam::parse_root, mvs_try, prelude::*};

impl CameraDevice for MVDevice {
    fn get_device_info(&self) -> DeviceResult<birb_vision_core::backend::DeviceInfo> {
        Ok(convert_info(self.get_info()?))
    }

    fn all_properties(&self) -> DeviceResult<Vec<Node>> {
        let xml = self.get_GenICam_xml()?;
        std::fs::write("mvs.xml", &xml).unwrap();
        let doc = roxmltree::Document::parse(&xml).unwrap();
        let xml_root = doc.root_element();
        let mut list = vec![];
        let root = parse_root(xml_root, ROOT_ID, &mut list);
        list.push(root);

        Ok(list)
    }

    fn root_property(&self) -> DeviceResult<Option<NodeId>> {
        Ok(ROOT_ID.into())
    }

    fn user_root_property(&self) -> DeviceResult<Option<NodeId>> {
        Ok(USER_ROOT_ID.into())
    }

    // TODO use this to exclude 
    //fn properties(&self) -> DeviceResult<Node> {
    //    let e: Box<dyn std::error::Error + 'static> = "a".into();
//
    //    self
    //        .control_description()?
    //        .variant.into_group().unwrap()
    //        .children.into_iter()
    //        .cloned()
    //        .filter_map(|n| n.into_node().ok())
    //        .filter(|n| n.variant.is_group())
    //        .filter(|g| g.id.as_ref().map(|id| id.as_str() == Some("Root")).unwrap_or(false))
    //        .next().ok_or(DeviceError::Other(anyhow::Error::msg("Root node not found")))
    //}

    fn read_property(&self, node: &Node) -> DeviceResult<PropertyState> {
        let id = node.id.as_str().unwrap();
        let r = match &node.variant {
            NodeVariant::Property(variant) => match variant {
                PropertyVariant::Bool(_) => PropertyState::Bool(self.get_bool_value(id)?),
                PropertyVariant::Integer(_) => self
                    .get_int_value(id)
                    .map(|v| PropertyState::Int(NumericValue::<i64> {
                        current: v.current() as _,
                        range: v.min() as _ ..= v.max() as _,
                    }))?,
                PropertyVariant::Float(_) => self
                    .get_float_value(id)
                    .map(|v| PropertyState::Float(NumericValue::<f64> {
                        current: v.current() as _,
                        range: v.min() as _ ..= v.max() as _,
                    }))?,
                PropertyVariant::Enum(_) => self
                    .get_enum_value(id)
                    .map(|v| PropertyState::Enum(EnumValue {
                        current: v.current_value() as _,
                        support: Cow::Owned(v.support().iter().map(|v| *v as i64).collect::<Vec<_>>()),
                    }))?,
                PropertyVariant::String(_) => self
                    .get_string_value(id)
                    .map(|s| PropertyState::String(s.current_value().to_string()))?,
                PropertyVariant::Command => Err(anyhow!("Cannot read a command property"))?,
            },
            NodeVariant::Group(_) => Err(anyhow!("Cannot read a group property"))?,
            NodeVariant::Port => todo!(),
        };
        Ok(r)
    }

    fn write_property(&self, node: &Node, value: PropertyValue) -> DeviceResult {
        let id = node.id.as_str().unwrap();
        match value {
            PropertyValue::Bool(value) => self.set_bool_value(id, value).map_err(|e| e.into()),
            PropertyValue::Integer(value) => self.set_int_value(id, value as _).map_err(|e| e.into()),
            PropertyValue::Float(value) => self.set_float_value(id, value as _).map_err(|e| e.into()),
            PropertyValue::Enum(value) => self.set_enum_value(id, value as _).map_err(|e| e.into()),
            PropertyValue::String(value) => self.set_string_value(id, &value).map_err(|e| e.into()),
            PropertyValue::Command => self.set_command_value(id).map_err(|e| e.into()),
        }
    }

    fn start_grabbing(&self) -> DeviceResult<()> {
        self.start_grabbing().map_err(|e| e.into())
    }

    fn stop_grabbing(&self) -> DeviceResult<()> {
        self.stop_grabbing().map_err(|e| e.into())
    }

    fn flush(&self) -> DeviceResult<()> {
        log::error!("flush not implemented for MVSDevice");
        Ok(())
    }

    fn grab(&self) -> DeviceResult<()> {
        // TODO this function is deprecated, what should we use instead? Maybe MV_CC_SetCommandValue
        mvs_try!(self.cx => MV_CC_TriggerSoftwareExecute(self.handle)).map_err(|e| e.into())
    }

    fn set_stream_callback(&self, f: Box<dyn Fn(StreamEvent) + Send + Sync>) -> DeviceResult {
        self.set_image_callback(Box::new(move |sample| {
            f(StreamEvent::Sample(Ok(Sample::ImageSample(sample))))
        }));

        // TODO
        self.set_all_event_callback(Box::new(move || {
            println!("EVENT!-------------------------------------------------")
            //f(Event::Flushed)
        }));

        Ok(())
    }
}

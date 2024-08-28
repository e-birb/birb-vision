

use std::borrow::Cow;

use birb_vision_core::anyhow;

use birb_vision_core::{CameraDevice, DeviceAccessMode, DeviceError, DeviceResult, EnumValue, Event, Frame, Node, NodeId, NumericValue};
use crate::{genicam::parse_root, mvs_try, prelude::*, MVError};

impl CameraDevice for MVDevice {
    fn is_device_accessible(&self, mode: DeviceAccessMode) -> bool {
        todo!()
    }

    fn is_open(&self) -> Option<DeviceAccessMode> {
        todo!()
    }

    fn open(&self, mode: DeviceAccessMode) -> DeviceResult<()> {
        MVDevice::open(
            self,
            AccessMode::Exclusive,
            None,
        ).map_err(|e| e.into())
    }

    fn close(&self) -> DeviceResult<()> {
        MVDevice::close(self).map_err(|e| e.into())
    }

    fn control_description(&self) -> DeviceResult<Node> {
        let xml = self.get_GenICam_xml()?;
        std::fs::write("mvs.xml", &xml).unwrap();
        let doc = roxmltree::Document::parse(&xml).unwrap();
        let r = doc.root_element();
        let node = parse_root(r);

        Ok(node)
    }

    fn properties(&self) -> DeviceResult<Node> {
        let e: Box<dyn std::error::Error + 'static> = "a".into();

        self
            .control_description()?
            .variant.into_group().unwrap()
            .children.into_iter()
            .cloned()
            .filter_map(|n| n.into_node().ok())
            .filter(|n| n.variant.is_group())
            .filter(|g| g.id.as_ref().map(|id| id.as_str() == Some("Root")).unwrap_or(false))
            .next().ok_or(DeviceError::Other(anyhow::Error::msg("Root node not found")))
    }

    fn get_bool_property(&self, id: &NodeId) -> DeviceResult<bool> {
        self.get_bool_value(id.as_str().unwrap()).map_err(|e| e.into())
    }

    fn get_int_property(&self, id: &NodeId) -> DeviceResult<NumericValue<i64>> {
        self
            .get_int_value(id.as_str().unwrap())
            .map_err(|e| e.into())
            .map(|v| NumericValue::<i64> {
                current: v.current() as _,
                range: v.min() as _ ..= v.max() as _,
            })
    }

    fn get_float_property(&self, id: &NodeId) -> DeviceResult<NumericValue<f64>> {
        self
            .get_float_value(id.as_str().unwrap())
            .map_err(|e| e.into())
            .map(|v| NumericValue::<f64> {
                current: v.current() as _,
                range: v.min() as _ ..= v.max() as _,
            })
    }
    fn get_enum_property(&self, id: &NodeId) -> DeviceResult<EnumValue> {
        self
            .get_enum_value(id.as_str().unwrap())
            .map_err(|e| e.into())
            .map(|v| EnumValue {
                current: v.current_value() as _,
                support: Cow::Owned(v.support().iter().map(|v| *v as i64).collect::<Vec<_>>()),
            })
    }

    fn get_string_property(&self, id: &NodeId) -> DeviceResult<String> {
        self
            .get_string_value(id.as_str().unwrap())
            .map_err(|e| e.into())
            .map(|s| s.current_value().to_string())
    }

    fn set_property(&self, id: &NodeId, value: &birb_vision_core::PropertyValue) -> DeviceResult {
        todo!()
    }

    fn set_bool_property(&self, id: &NodeId, value: bool) -> DeviceResult {
        self.set_bool_value(id.as_str().unwrap(), value).map_err(|e| e.into())
    }
    fn set_int_property(&self, id: &NodeId, value: i64) -> DeviceResult {
        self.set_int_value(id.as_str().unwrap(), value as _).map_err(|e| e.into())
    }
    fn set_float_property(&self, id: &NodeId, value: f64) -> DeviceResult {
        self.set_float_value(id.as_str().unwrap(), value as _).map_err(|e| e.into())
    }
    fn set_enum_property(&self, id: &NodeId, value: i64) -> DeviceResult {
        self.set_enum_value(id.as_str().unwrap(), value as _).map_err(|e| e.into())
    }
    fn set_string_property(&self, id: &NodeId, value: &str) -> DeviceResult {
        self.set_string_value(id.as_str().unwrap(), value).map_err(|e| e.into())
    }
    fn send_command(&self, id: &NodeId) -> DeviceResult {
        self.set_command_value(id.as_str().unwrap()).map_err(|e| e.into())
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

    //async fn receive_frame(&self) -> DeviceResult<std::borrow::Cow<'_, Frame>> {
    //    // TODO HANDLE DIFFERENT PIXEL FORMATS
//
    //    let w = self.get_int_value("Width").map_err(|e| e.into())?.current();
    //    let h = self.get_int_value("Height").map_err(|e| e.into())?.current();
    //    //let pitch = self.get_int_value("LinePitch").map_err(|e| e.into())?.current();
    //    //assert_eq!(pitch, w, "LinePitch != Width");
//
    //    let mut buf = ImageBuffer::<Luma<u8>, Vec<u8>>::new(w as u32, h as u32).into_raw();
//
    //    self.get_one_frame_timeout(&mut buf, Duration::from_secs(1)).map_err(|e| e.into())?;
//
    //    let img = DynamicImage::ImageLuma8(ImageBuffer::from_raw(w as u32, h as u32, buf).unwrap());
//
    //    Ok(Cow::Owned(Frame::Image(img)))
    //}

    fn grab(&self) -> DeviceResult<()> {
        // TODO this function is deprecated, what should we use instead? Maybe MV_CC_SetCommandValue
        mvs_try!(self.cx => MV_CC_TriggerSoftwareExecute(self.handle)).map_err(|e| e.into())
    }

    fn set_stream_callback(&self, f: Box<dyn Fn(Event) + Send + Sync>) -> DeviceResult {
        self.set_image_callback(Box::new(move |img| {
            f(Event::Frame(Ok(Frame::Image(img))))
        }));

        // TODO
        self.set_all_event_callback(Box::new(move || {
            println!("EVENT!-------------------------------------------------")
            //f(Event::Flushed)
        }));

        Ok(())
    }
}

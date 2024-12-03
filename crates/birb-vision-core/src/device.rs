
use clap::ValueEnum;
use enum_as_inner::EnumAsInner;
use serde::{Deserialize, Serialize};

use crate::{context::DeviceInfo, DeviceError, DeviceResult, StreamEvent, Node, NodeId, PropertyState, PropertyValue};

mod io;

pub use io::DeviceIO;

// TODO a lower level "DeviceIO" or something trait that exposes lower level functions like read/write register, etc.
// and make a function `CameraDevice::io(&mut self) -> Option<&mut dyn DeviceIO>` (or result) that returns a reference to the IO trait
// (might be self). Teh methods exposed by CameraDevice are instead high-level functions.

pub trait CameraDevice: Send + Sync {
    fn get_device_info(&mut self) -> DeviceResult<DeviceInfo> {
        Err(DeviceError::NotImplemented)
    }

    fn access_mode(&mut self) -> DeviceResult<DeviceAccessMode> {
        Err(DeviceError::NotImplemented)
    }

    /// Get all the properties of the device
    fn all_properties(&mut self) -> DeviceResult<Vec<Node>> {
        return Ok(vec![]);
    }

    /// Roots of all properties
    fn root_properties(&mut self) -> DeviceResult<Vec<NodeId>> {
        Ok(
            self
                .all_properties()?
                .into_iter()
                .map(|p| p.id.clone())
                .collect()
        )
    }

    /// Root of the interesting properties to be exposed to the user
    fn user_root_properties(&mut self) -> DeviceResult<Vec<NodeId>> {
        self.root_properties()
    }

    // We may add a "timeout" parameter or some other async way to handle reads
    // of properties that may either take a long time, block teh device or
    // need to wait for some event to happen (example frames).
    // Allow read frames with this method? (maybe adding the appropriate node)
    fn read_property(&mut self, _id: &NodeId) -> DeviceResult<PropertyState> {
        Err(DeviceError::NotImplemented)
    }
    fn write_property(&mut self, _id: &NodeId, _value: PropertyValue) -> DeviceResult {
        Err(DeviceError::NotImplemented)
    }

    fn is_grabbing(&self) -> DeviceResult<bool> {
        Err(DeviceError::NotImplemented)
    }
    fn start_grabbing(&mut self) -> DeviceResult; // TODO a stream object?
    fn stop_grabbing(&mut self) -> DeviceResult;

    /// Get a reference to the underlying IO object to read/write registers.
    fn io<'a>(&'a mut self) -> Option<Box<dyn DeviceIO + 'a>> {
        None
    }

    /// Tell the camera to read a sample from the stream
    ///
    /// This acts as a sort of "software trigger" for the camera, telling it to read a sample from the stream.
    /// The actual behavior is implementation-defined, and may not have any effect on some devices or in some
    /// configurations such as:
    /// - the camera is not streaming
    /// - the camera is in free-run mode or is in not software-triggerable
    ///
    /// # Notes
    /// - This method is similar to [OpenCV's `VideoCapture::grab`](https://github.com/opencv/opencv/blob/ae4a11b0c0986809d2f938f68343c8da99286b29/modules/videoio/include/opencv2/videoio.hpp#L878), but it is not guaranteed to have effect.
    fn grab(&mut self) -> DeviceResult;

    fn flush(&mut self) -> DeviceResult {
        Err(DeviceError::NotImplemented)
    }

    /// Similar to [futures::stream::Stream::poll_next] but with no Pin requirement
    // TODO Is this "no Pin requirement" good?
    //fn poll_events(&self, ctx: &mut std::task::Context) -> Poll<DeviceResult<Event>>;

    fn set_stream_callback(&mut self, f: Box<dyn for<'a> Fn(StreamEvent<'a>) + Send + Sync>) -> DeviceResult;
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(EnumAsInner)]
#[derive(Serialize, Deserialize)]
#[derive(ValueEnum)]
pub enum DeviceAccessMode {

    /// Exclusive access to the device
    #[default]
    Exclusive,

    // TODO ExclusiveWithSwitch 

    /// Control access to the device
    ///
    /// Exclusive access to control the device, but
    /// other actors may still read from the device.
    Control,

    // TODO ControlWithSwitch
    // TODO ControlSwitchEnable
    // TODO ControlSwitchEnableWithKey

    /// Monitor access to the device
    Monitor,
}


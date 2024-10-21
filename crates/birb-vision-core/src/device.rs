use clap::ValueEnum;
use enum_as_inner::EnumAsInner;
use serde::{Deserialize, Serialize};

use crate::{backend::DeviceInfo, DeviceResult, Event, Node, NodeId, PropertyState, PropertyValue};

pub trait CameraDevice {
    fn get_device_info(&self) -> DeviceResult<DeviceInfo>;

    /// All controls
    fn all_properties(&self) -> DeviceResult<Vec<Node>>;

    fn root_property(&self) -> DeviceResult<NodeId>;

    /// Root of the interesting properties to be exposed to the user
    fn user_root_property(&self) -> DeviceResult<NodeId> {
        self.root_property()
    }

    fn read_property(&self, node: &Node) -> DeviceResult<PropertyState>;
    fn write_property(&self, node: &Node, value: PropertyValue) -> DeviceResult;

    fn start_grabbing(&self) -> DeviceResult;
    fn stop_grabbing(&self) -> DeviceResult; // TODO a stream object

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
    fn grab(&self) -> DeviceResult;

    fn flush(&self) -> DeviceResult;

    /// Similar to [futures::stream::Stream::poll_next] but with no Pin requirement
    // TODO Is this "no Pin requirement" good?
    //fn poll_events(&self, ctx: &mut std::task::Context) -> Poll<DeviceResult<Event>>;

    fn set_stream_callback(&self, f: Box<dyn for<'a> Fn(Event<'a>) + Send + Sync>) -> DeviceResult;
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
use enum_as_inner::EnumAsInner;

use crate::{DeviceResult, NodeId, Sample};


/// An event that can be emitted by a vision device
///
/// A lifetime might be associated with the event,
/// use [`Event::into_owned`] to convert it into an owned event.
#[derive(Debug)]
#[derive(EnumAsInner)]
// TODO #[derive(Serialize, Deserialize)]
pub enum StreamEvent<'a> {
    /// A new sample is available
    ///
    /// See [`Sample`] for more information.
    Sample(DeviceResult<Sample<'a>>),
    Flushed, // TODO maybe remove
    // TODO consider not having any events that are not "common" since the user may expect them
    // but never emitted by the implementation. Another possibility would be to group them
    // in another enum for non-common/ensured events.

    /// A property value changed
    PropertyChanged(NodeId),
}

impl<'a> StreamEvent<'a> {
    pub fn into_owned(self) -> StreamEvent<'static> {
        todo!()
    }
}
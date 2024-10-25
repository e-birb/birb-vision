use std::{borrow::Cow, fmt::Debug};

use enum_as_inner::EnumAsInner;
use image::DynamicImage;

mod pixel_format;
mod fourcc;
mod flat_sample;
pub use pixel_format::PixelFormat;
pub use fourcc::FourCC;
pub use flat_sample::*;

/// A sample (possibly a frame) captured by a camera
///
/// This enum is used to represent the different types of frames that can be captured by a camera.
///
/// ## Conversion
///
/// A sample might not be very user friendly as it may contain raw data. A typical use might be:
/// ```no_run
/// #let sample: Sample<'static> = todo!();
/// // the sample might consist of a reference, we can use `into_owned` to convert it into an owned sample
/// let owned_sample: Sample<'static> = sample.into_owned();
///
/// // often the sample consists of raw pixels in a common format, we can try to reinterpret it into
/// // a format compatible with the `image` crate
/// let sample: Sample = owned_sample.try_reinterpret();
/// 
/// // Sometimes the format is not in a known image format, but we can try to decode it
/// let image: Sample = sample.try_decode();
/// ```
#[derive(Clone)]
#[derive(EnumAsInner)]
pub enum Sample<'a> {
    /// An owned image
    Image(DynamicImage),

    /// A flat sample (more generic than [`Sample::FlatImageSample`])
    FlatSample(FlatSample<Cow<'a, [u8]>>),

    // TODO point cloud, maybe depth map, ...
}

impl<'a> Sample<'a> {
    /// Try reinterpreting the sample in a known format
    ///
    /// This method **tries** to reinterpret the data in a known [`image`]
    /// format ([`DynamicImage`] or [`FlatSamples`]) **without copying the data**,
    /// leaving the sample as is if this is not possible.
    pub fn try_reinterpret(self) -> Self {
        todo!()
    }

    pub fn into_owned(self) -> Sample<'static> {
        todo!()
    }

    /// Tries to decode the sample into a DynamicImage
    pub fn try_decode(self) -> Result<Result<DynamicImage, anyhow::Error>, Self> {
        todo!()
    }
}

impl<'a> Debug for Sample<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Sample::Image(_) => write!(f, "Image"),
            Sample::FlatSample(_) => write!(f, "FlatSample"),
        }
    }
}


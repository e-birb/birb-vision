use std::{borrow::Cow, fmt::Debug};

use enum_as_inner::EnumAsInner;
use image::DynamicImage;
use serde::{Deserialize, Serialize};

mod pixel_format;
pub use pixel_format::PixelFormat;

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
            _ => todo!(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Serialize, Deserialize)]
pub struct FourCC(pub [u8; 4]);

impl FourCC {
    pub fn new(bytes: [u8; 4]) -> Self {
        Self(bytes.clone())
    }
}

impl From<[u8; 4]> for FourCC {
    fn from(bytes: [u8; 4]) -> Self {
        Self(bytes)
    }
}

impl From<&[u8; 4]> for FourCC {
    fn from(bytes: &[u8; 4]) -> Self {
        Self(bytes.clone())
    }
}

#[derive(Debug, Clone)]
pub struct FlatSample<Buffer> {
    pub buffer: Buffer,

    /// Offset of the first row/column
    ///
    /// See [`Self::line_offset`]
    pub offset: usize,

    pub sample_type: SampleType,

    /// Width of the image
    pub width: u32,

    /// Height of the image
    pub height: u32,

    /// Stride in bytes for each row or column (pitch)
    ///
    /// If the stride is negative, the image is flipped.
    pub stride: i32,

    pub row_major: bool,
}

impl<Buffer> FlatSample<Buffer> {
    pub fn line_offset(&self, line_index: u32) -> usize {
        self.offset + (line_index as usize) * self.stride as usize
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SampleType {
    FourCC(FourCC),
    Plain(PixelFormat),
}
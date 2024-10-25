use serde::{Deserialize, Serialize};

use super::{FourCC, PixelFormat};

#[derive(Debug, Clone)]
pub struct FlatSample<Buffer> {
    pub buffer: Buffer,
    pub layout: FlatSampleLayout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Serialize, Deserialize)]
pub enum SampleType {
    FourCC(FourCC),
    Plain(PixelFormat),
}

#[derive(Debug, Clone, Copy)]
pub struct FlatSampleLayout {
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

impl FlatSampleLayout {
    pub fn line_offset(&self, line_index: u32) -> usize {
        self.offset + (line_index as usize) * self.stride as usize
    }
}
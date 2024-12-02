use std::{borrow::Cow, fmt::Debug, sync::Arc};

use enum_as_inner::EnumAsInner;
use image::DynamicImage;

mod pixel_format;
mod fourcc;
mod flat_sample;
mod locked_buffer;
pub use pixel_format::PixelFormat;
pub use fourcc::FourCC;
pub use flat_sample::*;
pub use locked_buffer::LockedBuffer;
use serde::{Deserialize, Serialize};


#[derive(Clone)]
pub enum ImageSampleBuffer<'a> {
    LockedBuffer(Arc<dyn LockedBuffer>),
    Cow(Cow<'a, [u8]>),
}

impl<'a> Debug for ImageSampleBuffer<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LockedBuffer(_) => write!(f, "LockedBuffer"),
            Self::Cow(_) => write!(f, "Cow"),
        }
    }
}

/// A sample (possibly a frame) captured by a camera
#[derive(Debug, Clone)]
#[derive(EnumAsInner)]
pub enum Sample<'a> {
    ImageSample(FlatSample<ImageSampleBuffer<'a>>),
    // TODO point cloud, maybe depth map, ...
}

impl<'a> Sample<'a> {
    pub fn into_owned(self) -> Sample<'static> {
        let Self::ImageSample(sample) = self;
        let layout = sample.layout;
        let buffer = match sample.buffer {
            ImageSampleBuffer::LockedBuffer(buffer) => ImageSampleBuffer::LockedBuffer(buffer),
            ImageSampleBuffer::Cow(buffer) => ImageSampleBuffer::Cow(buffer.into_owned().into()),
        };
        Sample::ImageSample(FlatSample { buffer, layout })
    }

    /// Tries to decode the sample into a DynamicImage
    ///
    /// # Returns
    /// - Err(Sample) if the sample is not decodable
    pub fn try_decode(self) -> Result<Result<DynamicImage, anyhow::Error>, Self> {
        let Sample::ImageSample(flat_sample) = self;

        match flat_sample.buffer {
            ImageSampleBuffer::LockedBuffer(buffer) => {
                let lb = buffer.clone();
                FlatSample::try_decode_buffer(Cow::Borrowed(buffer.data()), &flat_sample.layout)
                    .map_err(move |_| Sample::ImageSample(FlatSample {
                        buffer: ImageSampleBuffer::LockedBuffer(lb),
                        layout: flat_sample.layout
                    }))
            },
            ImageSampleBuffer::Cow(buffer) => {
                FlatSample::try_decode_buffer(buffer, &flat_sample.layout)
                    .map_err(|buffer| Sample::ImageSample(FlatSample {
                        buffer: ImageSampleBuffer::Cow(buffer),
                        layout: flat_sample.layout
                    }))
            },
        }
    }
}


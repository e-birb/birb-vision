use std::{borrow::Cow, fmt::Debug, sync::Arc, time::Instant};

use enum_as_inner::EnumAsInner;
use image::{DynamicImage, Luma, RgbImage};

mod pixel_format;
mod fourcc;
mod flat_sample;
mod locked_buffer;
use log_once::warn_once;
pub use pixel_format::PixelFormat;
pub use fourcc::FourCC;
pub use flat_sample::*;
pub use locked_buffer::LockedBuffer;

use crate::decoders::yuyv422_to_rgb;

/// A sample (possibly a frame) captured by a camera
#[derive(Clone)]
#[derive(EnumAsInner)]
pub enum Sample<'a> {
    LockedBuffer(Arc<dyn LockedBuffer>),
    FlatSample(FlatSample<Cow<'a, [u8]>>),
    // TODO point cloud, maybe depth map, ...
}

impl<'a> Sample<'a> {
    pub fn into_owned(self) -> Sample<'static> {
        todo!()
    }

    /// Tries to decode the sample into a DynamicImage
    pub fn try_decode(self) -> Result<Result<DynamicImage, anyhow::Error>, Sample<'static>> {
        let lb: Arc<dyn LockedBuffer>;

        let flat_sample = match self {
            Sample::LockedBuffer(locked_buffer) => {
                lb = locked_buffer;
                lb.sample()
            },
            Sample::FlatSample(flat_sample) => flat_sample,
        };

        let layout = &flat_sample.layout;

        if layout.sample_type == SampleType::FourCC(FourCC::new(b"YUYV")) {
            let buffer = flat_sample.buffer;
            let start = Instant::now();
            let data = yuyv422_to_rgb(&buffer, false).unwrap();
            println!("Converted in {:?}", start.elapsed());
            let img = DynamicImage::ImageRgb8(RgbImage::from_raw(layout.width as u32, layout.height as u32, data).unwrap());
            return Ok(Ok(img));
        }

        if layout.sample_type == SampleType::FourCC(FourCC::new(b"RGB3")) {
            let buffer = flat_sample.buffer.into_owned();
            let img = DynamicImage::ImageRgb8(RgbImage::from_raw(layout.width as u32, layout.height as u32, buffer).unwrap());
            return Ok(Ok(img));
        }

        if layout.sample_type == SampleType::FourCC(FourCC::new(b"MJPG")) {
            let buffer = flat_sample.buffer;
            let start = Instant::now();
            //let img = birb_vision_core::decoders::decode_mjpg(data).unwrap();
            //let img = DynamicImage::ImageRgb8(img);
            let img = image::load_from_memory(&buffer).unwrap();
            println!("Converted mjpeg in {:?}", start.elapsed());
            return Ok(Ok(img));
        }

        if layout.sample_type == SampleType::Plain(PixelFormat::Mono8Packed) {
            // TODO this is just a quick hack to get the image to display but it's not correct
            warn_once!("Quick hack to display Mono8Packed image");
            if layout.row_major && layout.height > 0 && layout.width > 0 && layout.offset == 0 {
                let buffer = flat_sample.buffer.into_owned();
                let image = image::ImageBuffer::<Luma<u8>, Vec<u8>>::from_raw(layout.width, layout.height, buffer).unwrap();
                let dynamic_image = DynamicImage::ImageLuma8(image);
                return Ok(Ok(dynamic_image));
            }
        }

        if layout.sample_type == SampleType::Plain(PixelFormat::RGB8Packed) {
            // TODO this is just a quick hack to get the image to display but it's not correct
            warn_once!("Quick hack to display RGB8Packed image");
            if layout.row_major && layout.height > 0 && layout.width > 0 && layout.offset == 0 {
                let buffer = flat_sample.buffer.into_owned();
                let image = image::ImageBuffer::<image::Rgb<u8>, Vec<u8>>::from_raw(layout.width, layout.height, buffer).unwrap();
                let dynamic_image = DynamicImage::ImageRgb8(image);
                return Ok(Ok(dynamic_image));
            }
        }

        let buffer = flat_sample.buffer.into_owned();

        Err(Sample::FlatSample(FlatSample {
            buffer: Cow::Owned(buffer),
            layout: layout.clone(),
        }))
    }
}

impl<'a> Debug for Sample<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Sample::LockedBuffer(_) => write!(f, "LockedBuffer"),
            Sample::FlatSample(_) => write!(f, "FlatSample"),
        }
    }
}


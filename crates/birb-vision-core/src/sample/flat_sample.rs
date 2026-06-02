#[cfg(feature = "conversion")]
use std::borrow::Cow;

use serde::{Deserialize, Serialize};

#[cfg(feature = "conversion")]
use image::{DynamicImage, Luma, RgbImage};
#[cfg(all(feature = "conversion", feature = "log"))]
use log_once::warn_once;

#[cfg(all(feature = "conversion", not(feature = "log")))]
macro_rules! warn_once {
    ($($arg:tt)*) => {};
}

#[cfg(feature = "conversion")]
use crate::decoders::{self, yuyv422_to_rgb};

use super::{FourCC, PixelFormat};

#[derive(Clone, Serialize, Deserialize)]
pub struct FlatSample<Buffer> {
    pub buffer: Buffer,
    pub layout: FlatSampleLayout,
}

impl<Buffer> std::fmt::Debug for FlatSample<Buffer> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FlatSample")
            .field("layout", &self.layout)
            .field("buffer", &"...")
            .finish()
    }
}

impl<Buffer> FlatSample<Buffer> {
}

#[cfg(feature = "conversion")]
impl FlatSample<()> {
    /// Tries to decode the sample into a DynamicImage
    ///
    /// # Returns
    /// - Err(Sample) if the sample is not decodable
    pub fn try_decode_buffer<'a>(buffer: Cow<'a, [u8]>, layout: &FlatSampleLayout) -> Result<Result<DynamicImage, anyhow::Error>, Cow<'a, [u8]>> {

        if layout.sample_type == SampleType::FourCC(FourCC::new(b"YUYV")) || layout.sample_type == SampleType::FourCC(FourCC::new(b"YUY2")) { // TODO not sure ...
            let data = yuyv422_to_rgb(&buffer, false).unwrap();
            let img = DynamicImage::ImageRgb8(RgbImage::from_raw(
                layout.width as u32,
                layout.height as u32,
                data,
            ).unwrap());
            return Ok(Ok(img));
        }

        if layout.sample_type == SampleType::FourCC(FourCC::new(b"NV12")) {
            let img = decoders::nv12_to_rgb_image(layout.width, layout.height, &buffer, false).unwrap();
            return Ok(Ok(DynamicImage::ImageRgb8(img)));
        }

        if layout.sample_type == SampleType::FourCC(FourCC::new(b"RGB3")) {
            let img = DynamicImage::ImageRgb8(RgbImage::from_raw(
                layout.width as u32,
                layout.height as u32,
                buffer.into_owned(),
            ).unwrap());
            return Ok(Ok(img));
        }

        if layout.sample_type == SampleType::FourCC(FourCC::new(b"MJPG")) || layout.sample_type == SampleType::FourCC(FourCC::new(b"JPEG")) {
            //let img = birb_vision_core::decoders::decode_mjpg(data).unwrap();
            //let img = DynamicImage::ImageRgb8(img);
            let img = image::load_from_memory(&buffer).unwrap();
            return Ok(Ok(img));
        }

        if layout.sample_type == SampleType::Plain(PixelFormat::Mono8Packed) {
            // TODO this is just a quick hack to get the image to display but it's not correct
            warn_once!("Quick hack to display Mono8Packed image");
            if layout.row_major && layout.height > 0 && layout.width > 0 && layout.offset == 0 {
                let buffer = buffer.into_owned();
                let image = image::ImageBuffer::<Luma<u8>, Vec<u8>>::from_raw(layout.width, layout.height, buffer).unwrap();
                let dynamic_image = DynamicImage::ImageLuma8(image);
                return Ok(Ok(dynamic_image));
            }
        }

        if layout.sample_type == SampleType::Plain(PixelFormat::RGB8Packed) {
            // TODO this is just a quick hack to get the image to display but it's not correct
            warn_once!("Quick hack to display RGB8Packed image");
            if layout.row_major && layout.height > 0 && layout.width > 0 && layout.offset == 0 {
                let buffer = buffer.into_owned();
                let image = image::ImageBuffer::<image::Rgb<u8>, Vec<u8>>::from_raw(layout.width, layout.height, buffer).unwrap();
                let dynamic_image = DynamicImage::ImageRgb8(image);
                return Ok(Ok(dynamic_image));
            }
        }

        if layout.sample_type == SampleType::Plain(PixelFormat::BGR8Packed) {
            // TODO this is just a quick hack to get the image to display but it's not correct
            warn_once!("Quick hack to display BGR8Packed image");
            if layout.row_major && layout.height > 0 && layout.width > 0 && layout.offset == 0 {
                let buffer = buffer.into_owned();
                let stride = layout.width * 3; // TODO
                let image = decoders::decode_bgr(&buffer, layout.width, layout.height, stride as _, true);
                let dynamic_image = DynamicImage::ImageRgb8(image);
                return Ok(Ok(dynamic_image));
            }
        }

        Err(buffer)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Serialize, Deserialize)]
pub enum SampleType {
    FourCC(FourCC),
    Plain(PixelFormat),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
use std::error::Error;

use image::RgbImage;


/// Decode a Motion JPEG image.
pub fn decode_mjpg(
    data: &[u8],
) -> Result<RgbImage, Box<dyn Error>> {
    let decompress = mozjpeg::Decompress::with_markers(mozjpeg::ALL_MARKERS)
        .from_mem(data)?;

    let mut img = decompress
        .rgb()
        .map_err(|e| e.to_string())?;

    let width = img.width();
    let height = img.height();
    assert_eq!(img.color_space(), mozjpeg::ColorSpace::JCS_RGB);

    let pixels = img.read_scanlines::<u8>()?;
    img.finish()?;

    assert_eq!(pixels.len(), width as usize * height as usize * 3);

    Ok(RgbImage::from_raw(width as u32, height as u32, pixels).unwrap())
}
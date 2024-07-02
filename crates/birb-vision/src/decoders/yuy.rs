// https://github.com/l1npengtul/nokhwa/blob/14339380c277a1c47ba0731481bb9ce12ef56cb4/nokhwa-core/src/types.rs

use std::error::Error;


/// Returns the predicted size of the destination Yuv422422 buffer.
#[inline]
pub fn yuyv422_predicted_size(size: usize, rgba: bool) -> usize {
    let pixel_size = if rgba { 4 } else { 3 };
    // yuyv yields 2 3-byte pixels per yuyv chunk
    (size / 4) * (2 * pixel_size)
}

#[inline]
pub fn yuyv422_to_rgb(data: &[u8], rgba: bool) -> Result<Vec<u8>, Box<dyn Error>> {
    let capacity = yuyv422_predicted_size(data.len(), rgba);
    let mut rgb = vec![0; capacity];
    buf_yuyv422_to_rgb(data, &mut rgb, rgba)?;
    Ok(rgb)
}

/// Same as [`yuyv422_to_rgb`] but with a destination buffer instead of a return `Vec<u8>`
/// # Errors
/// If the stream is invalid Yuv422, or the destination buffer is not large enough, this will error.
#[inline]
pub fn buf_yuyv422_to_rgb(data: &[u8], dest: &mut [u8], rgba: bool) -> Result<(), Box<dyn Error>> {
    let mut buf: Vec<u8> = Vec::new();
    if data.len() % 4 != 0 {
        //return Err(NokhwaError::ProcessFrameError {
        //    src: FrameFormat::Yuv422.into(),
        //    destination: "RGB888".to_string(),
        //    error: "Assertion failure, the YUV stream isn't 4:2:2! (wrong number of bytes)"
        //        .to_string(),
        //});
        return Err("Assertion failure, the YUV stream isn't 4:2:2! (wrong number of bytes)".into());
    }
    for chunk in data.chunks_exact(4) {
        let y0 = chunk[0] as f32;
        let u = chunk[1] as f32;
        let y1 = chunk[2] as f32;
        let v = chunk[3] as f32;

        let r0 = y0 + 1.370_705 * (v - 128.);
        let g0 = y0 - 0.698_001 * (v - 128.) - 0.337_633 * (u - 128.);
        let b0 = y0 + 1.732_446 * (u - 128.);

        let r1 = y1 + 1.370_705 * (v - 128.);
        let g1 = y1 - 0.698_001 * (v - 128.) - 0.337_633 * (u - 128.);
        let b1 = y1 + 1.732_446 * (u - 128.);

        if rgba {
            buf.extend_from_slice(&[
                r0 as u8, g0 as u8, b0 as u8, 255, r1 as u8, g1 as u8, b1 as u8, 255,
            ]);
        } else {
            buf.extend_from_slice(&[r0 as u8, g0 as u8, b0 as u8, r1 as u8, g1 as u8, b1 as u8]);
        }
    }
    dest.copy_from_slice(&buf);
    Ok(())
}

// equation from https://en.wikipedia.org/wiki/YUV#Converting_between_Y%E2%80%B2UV_and_RGB
/// Convert `YCbCr` 4:4:4 to a RGB888. [For further reading](https://en.wikipedia.org/wiki/YUV#Converting_between_Y%E2%80%B2UV_and_RGB)
#[allow(clippy::many_single_char_names)]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
#[must_use]
#[inline]
pub fn yuyv444_to_rgb(y: i32, u: i32, v: i32) -> [u8; 3] {
    let c298 = (y - 16) * 298;
    let d = u - 128;
    let e = v - 128;
    let r = ((c298 + 409 * e + 128) >> 8).clamp(0, 255) as u8;
    let g = ((c298 - 100 * d - 208 * e + 128) >> 8).clamp(0, 255) as u8;
    let b = ((c298 + 516 * d + 128) >> 8).clamp(0, 255) as u8;
    [r, g, b]
}

// equation from https://en.wikipedia.org/wiki/YUV#Converting_between_Y%E2%80%B2UV_and_RGB
/// Convert `YCbCr` 4:4:4 to a RGBA8888. [For further reading](https://en.wikipedia.org/wiki/YUV#Converting_between_Y%E2%80%B2UV_and_RGB)
///
/// Equivalent to [`yuyv444_to_rgb`] but with an alpha channel attached.
#[allow(clippy::many_single_char_names)]
#[must_use]
#[inline]
pub fn yuyv444_to_rgba(y: i32, u: i32, v: i32) -> [u8; 4] {
    let [r, g, b] = yuyv444_to_rgb(y, u, v);
    [r, g, b, 255]
}
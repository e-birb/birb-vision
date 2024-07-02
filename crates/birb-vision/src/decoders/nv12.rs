use std::error::Error;

use image::RgbImage;

use super::yuy::*;

pub fn nv12_to_rgb_image(
    width: u32,
    height: u32,
    data: &[u8],
    rgba: bool,
) -> Result<RgbImage, Box<dyn Error>> {
    let pixels = nv12_to_rgb(width, height, data, rgba)?;
    Ok(RgbImage::from_raw(width, height, pixels).unwrap())
}

/// Converts a Yuv422 4:2:0 bi-planar (NV12) datastream to a RGB888 Stream. [For further reading](https://en.wikipedia.org/wiki/YUV#Converting_between_Y%E2%80%B2UV_and_RGB)
/// # Errors
/// This may error when the data stream size is wrong.
#[inline]
pub fn nv12_to_rgb(
    width: u32,
    height: u32,
    data: &[u8],
    rgba: bool,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let pxsize = if rgba { 4 } else { 3 };
    let mut dest = vec![0; (pxsize * width * height) as usize];
    buf_nv12_to_rgb(width, height, data, &mut dest, rgba)?;
    Ok(dest)
}

// this depresses me
// like, everytime i open this codebase all the life is sucked out of me
// i hate it
/// Converts a Yuv422 4:2:0 bi-planar (NV12) datastream to a RGB888 Stream and outputs it into a destination buffer. [For further reading](https://en.wikipedia.org/wiki/YUV#Converting_between_Y%E2%80%B2UV_and_RGB)
/// # Errors
/// This may error when the data stream size is wrong.
#[allow(clippy::similar_names)]
#[inline]
pub fn buf_nv12_to_rgb(
    width: u32,
    height: u32,
    data: &[u8],
    out: &mut [u8],
    rgba: bool,
) -> Result<(), Box<dyn Error>> {
    if width % 2 != 0 || height % 2 != 0 {
        //return Err(NokhwaError::ProcessFrameError {
        //    src: FrameFormat::Nv12,
        //    destination: "RGB".to_string(),
        //    error: "bad resolution".to_string(),
        //});
        return Err("bad resolution".into());
    }

    if data.len() != ((width * height * 3) / 2) as usize {
        //return Err(NokhwaError::ProcessFrameError {
        //    src: FrameFormat::Nv12,
        //    destination: "RGB".to_string(),
        //    error: "bad input buffer size".to_string(),
        //});
        return Err("bad input buffer size".into());
    }

    let pxsize = if rgba { 4 } else { 3 };

    if out.len() != (pxsize * width * height) as usize {
        //return Err(NokhwaError::ProcessFrameError {
        //    src: FrameFormat::Nv12,
        //    destination: "RGB".to_string(),
        //    error: "bad output buffer size".to_string(),
        //});
        return Err("bad output buffer size".into());
    }

    let rgba_size = if rgba { 4 } else { 3 };

    let y_section = (width * height) as usize;

    let width_usize = width as usize;
    // let height_usize = resolution.height() as usize;

    for (hidx, horizontal_row) in data[0..y_section].chunks_exact(width_usize).enumerate() {
        for (cidx, column) in horizontal_row.chunks_exact(2).enumerate() {
            let u = data[(y_section) + ((hidx / 2) * width_usize) + (cidx * 2)];
            let v = data[(y_section) + ((hidx / 2) * width_usize) + (cidx * 2) + 1];

            let y0 = column[0];
            let y1 = column[1];
            let base_index = (hidx * width_usize * rgba_size) + cidx * rgba_size * 2;

            if rgba {
                let px0 = yuyv444_to_rgba(y0 as i32, u as i32, v as i32);
                let px1 = yuyv444_to_rgba(y1 as i32, u as i32, v as i32);

                out[base_index] = px0[0];
                out[base_index + 1] = px0[1];
                out[base_index + 2] = px0[2];
                out[base_index + 3] = px0[3];
                out[base_index + 4] = px1[0];
                out[base_index + 5] = px1[1];
                out[base_index + 6] = px1[2];
                out[base_index + 7] = px1[3];
            } else {
                let px0 = yuyv444_to_rgb(y0 as i32, u as i32, v as i32);
                let px1 = yuyv444_to_rgb(y1 as i32, u as i32, v as i32);

                out[base_index] = px0[0];
                out[base_index + 1] = px0[1];
                out[base_index + 2] = px0[2];
                out[base_index + 3] = px1[0];
                out[base_index + 4] = px1[1];
                out[base_index + 5] = px1[2];
            }
        }
    }

    Ok(())
}
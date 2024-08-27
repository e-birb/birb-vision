use image::RgbImage;



pub fn decode_bgr(
    src_buffer: &[u8],
    width: u32,
    height: u32,
    src_stride: i32,
    src_row_major: bool,
) -> RgbImage {
// TODO ) -> Result<image::RgbImage, image::ImageError> {
    // the row stride of the final buffer
    let row_stride = width * 3;

    // the final buffer is row-major and no padding
    let mut buffer = vec![0; width as usize * height as usize * 3];

    let offset_zero = if src_stride > 0 {
        0
    } else {
        if src_row_major {
            src_stride.abs() as u32 * (height - 1)
        } else {
            src_stride.abs() as u32 * (width - 1)
        }
    };

    if src_row_major {
        for y in 0..height {
            let src_offset = offset_zero as isize + y as isize * src_stride as isize;
            let dst_offset = y as usize * row_stride as usize;
            let src_slice = &src_buffer[src_offset as usize..(src_offset + (width * 3) as isize) as usize];
            let dst_slice = &mut buffer[dst_offset..dst_offset + (width * 3) as usize];

            for x in 0..width {
                dst_slice[x as usize * 3] = src_slice[x as usize * 3 + 2];
                dst_slice[x as usize * 3 + 1] = src_slice[x as usize * 3 + 1];
                dst_slice[x as usize * 3 + 2] = src_slice[x as usize * 3];
            }
        }
    } else {
        for y in 0..height {
            let dst_offset = y as usize * row_stride as usize;
            let dst_slice = &mut buffer[dst_offset..dst_offset + (width * 3) as usize];
            for x in 0..width {
                // TODO to check and test
                let src_offset = offset_zero as usize + x as usize * src_stride.abs() as usize;
                let src_slice = &src_buffer[src_offset..src_offset + (height * 3) as usize];
                dst_slice[x as usize * 3] = src_slice[y as usize * 3 + 2];
                dst_slice[x as usize * 3 + 1] = src_slice[y as usize * 3 + 1];
                dst_slice[x as usize * 3 + 2] = src_slice[y as usize * 3];
            }
        }
    }

    RgbImage::from_raw(width, height, buffer).unwrap()
}
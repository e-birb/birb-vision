use enum_as_inner::EnumAsInner;
use image::DynamicImage;


/// A frame captured by a camera.
///
/// This enum is used to represent the different types of frames that can be captured by a camera.
#[derive(Debug, Clone)]
#[derive(EnumAsInner)]
pub enum Frame {
    Image(DynamicImage),
    // TODO point cloud, maybe depth map, ...
}
use std::fmt::Debug;

use enum_as_inner::EnumAsInner;
use image::DynamicImage;


/// A frame captured by a camera.
///
/// This enum is used to represent the different types of frames that can be captured by a camera.
#[derive(Clone)]
#[derive(EnumAsInner)]
pub enum Frame {
    Image(DynamicImage),
    // TODO point cloud, maybe depth map, ...
}

impl Debug for Frame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Frame::Image(_) => write!(f, "Image"),
        }
    }
}
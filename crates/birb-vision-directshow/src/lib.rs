//! DirectShow backend for birb-vision.
//!
//! Uses the Windows DirectShow API to enumerate and capture from video
//! devices (webcams, capture cards, etc.).  Only available on `cfg(windows)`.
//!
//! # Architecture
//!
//! - [`DirectShowContext`] — enumerates devices and creates [`DirectShowDevice`] instances.
//! - [`DirectShowDevice`] — wraps a DirectShow capture filter, exposes camera
//!   controls (`IAMVideoProcAmp` / `IAMCameraControl`) and streams frames via
//!   a filter graph with `ISampleGrabber`.
//! - [`DSControl`] — all known DirectShow camera-control properties.

#![cfg(windows)]

mod ctx;
pub mod device;
mod error;

pub use ctx::DirectShowContext;
pub use device::DirectShowDevice;
pub use error::*;

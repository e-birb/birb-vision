use std::ffi::c_char;

mod error;
mod ctx;
mod device;
mod birb_vision_impl;

pub use error::*;
pub use ctx::*;
pub use device::*;

fn arr_to_str(arr: &[c_char]) -> String {
    let len = arr.iter().position(|&c| c == 0).unwrap_or(arr.len());
    arr[..len].iter().map(|&c| c as u8 as char).collect::<String>()
}


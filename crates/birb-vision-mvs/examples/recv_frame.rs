use std::time::Instant;

use birb_vision_mvs::prelude::*;
use birb_vision::{futures::StreamExt, CameraDevice, CameraDeviceEx};

fn main() {
    pollster::block_on(async_main());
}

async fn async_main() {
    let cx = MVContext::new(None).expect("Failed to initialize a MVS context");

    let devices = cx
        .enumerate_devices([TransportLayerType::Usb])
        .expect("Failed to enumerate devices");

    let mut device = devices.into_iter().next().unwrap().into_device(false).unwrap();

    CameraDevice::open(&mut device, Default::default()).unwrap();

    device.start_grabbing().unwrap();

    let mut image_stream = device.stream(10).unwrap();

    let start = Instant::now();
    for _ in 0..100 {
        let frame = image_stream.next().await.unwrap().into_frame().unwrap();
        //println!("Frame: {frame:?}");
    }
    println!("fps: {}", 100.0 / start.elapsed().as_secs_f32())
}
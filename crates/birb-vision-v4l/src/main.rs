use std::{io, time::Duration};

use birb_vision_v4l::V4lContext;
use birb_vision_core::context::VisionContext;

fn main() -> io::Result<()> {
    let cx = V4lContext::new();
    let devices = cx.enumerate(&cx.default_transport_layers());
    println!("{:?}", devices);
    
    let device = devices.unwrap()[0].clone();
    let mut dev = cx.create(&device).expect("Failed to create device").expect("Device not found");
    dev.set_stream_callback(Box::new(|event| {
        println!("{:?}", event);
    })).expect("Failed to set stream callback");
    dev.start_grabbing().expect("Failed to start grabbing");

    std::thread::sleep(Duration::from_secs(5));

    Ok(())
}
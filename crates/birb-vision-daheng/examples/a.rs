use std::time::Duration;

use birb_vision_daheng::{Ctx, Device};
use birb_vision_core::CameraDevice;

fn main() {
    let ctx = Ctx::new().unwrap();
    let n = ctx.get_all_device_base_info().unwrap().len();
    println!("Found {n} devices");

    for info in ctx.get_all_device_base_info().unwrap() {
        println!("Opening device: {:?}", info.model_name());
        let dev = Device::open(info).unwrap();
        dev.set_stream_callback(Box::new(|e| {
            println!("Event: {e:?}");
        })).unwrap();
        println!("Grabbing...");
        dev.start_grabbing().unwrap();
        std::thread::sleep(Duration::from_secs(5));
        dev.stop_grabbing().unwrap();
    }
}
use std::time::Duration;

use birb_vision::{CameraDevice};
use birb_vision_mvs::{device::TransportLayerType, MVContext, MVDevice};



fn main() {
    let cx = MVContext::new(None).expect("Failed to initialize a MVS context");

    let devices = cx
        .enumerate_devices([TransportLayerType::Usb])
        .expect("Failed to enumerate devices");

    let mut device = devices.into_iter().next().unwrap().into_device(false).unwrap();

    CameraDevice::open(&mut device, Default::default()).unwrap();

    device.set_stream_callback(Box::new(|ev| {
        println!("Event: {ev:?}");
        //im.save("im.png").unwrap();
    })).unwrap();

    device.start_grabbing().unwrap();

    std::thread::sleep(Duration::from_secs(2));
}
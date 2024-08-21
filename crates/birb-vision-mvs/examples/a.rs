use std::time::Duration;

use birb_vision::{CameraDevice};
use birb_vision_mvs::{device::TransportLayerType, MVContext, MVDevice};



fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let cx = MVContext::new(None).expect("Failed to initialize a MVS context");
    cx.set_sdk_log_path("/home/luca/develop/e-birb/repos/birb-vision/z").unwrap();

    let devices = cx
        .enumerate_devices([TransportLayerType::Usb])
        .expect("Failed to enumerate devices");

    let mut device = devices.into_iter().next().unwrap().into_device(true).unwrap();

    CameraDevice::open(&mut device, Default::default()).unwrap();

    device.set_stream_callback(Box::new(|ev| {
        println!("Event: {ev:?}");
        //im.save("im.png").unwrap();
    })).unwrap();

    let w = device.get_int_value("Width").unwrap();
    println!("Width: {:?}", w);
    device.set_enum_value_by_string("TriggerMode", "Off").unwrap();
    println!("{:#?}", device.get_info());
    println!("a: {:?}", device.get_enum_value("PixelFormat").unwrap());
    device.start_grabbing();

    std::thread::sleep(Duration::from_secs(2));
}
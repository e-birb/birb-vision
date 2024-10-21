use birb_vision_icube::{iCubeContext, CallbackEventType};
use birb_vision_core::{CameraDeviceEx, CameraDevice};


fn main() {
    //env_logger::builder()
    //    .filter_level(log::LevelFilter::Debug)
    //    .init();

    let cx = iCubeContext::new()
        .expect("failed to create iCube context, is the SDK installed?");

    cx.init_device_list(|devices| {
        for device in devices {
            println!("Device {}: {:?}", device.sdk_index(), device.name());
            let camera = device.open().expect("failed to open device");

            camera.set_callback(Box::new(|e| {
                match e {
                    CallbackEventType::NEW_FRAME(_) => println!("new frame"),
                    _ => println!("other event: {e:?}"),
                }
            }));

            camera.start_grabbing().expect("failed to start grabbing");
            std::thread::sleep(std::time::Duration::from_secs(5));
            //camera.stop_grabbing().expect("failed to stop grabbing");
        }
    });
}
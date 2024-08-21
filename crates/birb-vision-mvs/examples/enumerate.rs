use birb_vision_mvs::prelude::*;

fn main() {
    env_logger::init();

    // Initialize a context.
    // Don't worry about managing this object, this crate manages
    // everything for you.
    // You can call this function multiple times, or use the `MVSContext::current()`
    // method to get the context instance in the current thread.
    let cx = MVContext::new(None).expect("Failed to initialize a MVS context");

    println!("MVS SDK version: {}", cx.sdk_version());

    // Of course we need to enumerate the available devices first.
    // This will give us a list of device info, which we can use to create
    // a device handles.
    let devices = cx
        .enumerate_devices([TransportLayerType::Usb])
        .expect("Failed to enumerate devices");

    println!("Found {} MVS devices", devices.len());

    for device_info in devices {
        println!("{:#?}", device_info);

        let device = device_info
            .into_device(true)
            .expect("Failed to create a device handle");

        if device.open(AccessMode::Exclusive, None).is_ok() {
            println!("Device opened successfully");
        } else {
            println!("Failed to open device");
        }
    }
}

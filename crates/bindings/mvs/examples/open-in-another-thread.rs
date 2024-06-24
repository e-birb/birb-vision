use mvs::prelude::*;

fn main() {
    let devices = MVSContext::new(None)
        .expect("Failed to initialize a MVS context")
        .enumerate_devices([TransportLayerType::Usb])
        .expect("Failed to enumerate devices");

    std::thread::spawn(move || {
        for device_info in devices {
            let _device = device_info
                .into_device(true)
                .unwrap();
        }
    });
}
use birb_vision_media_foundation::{MFKnownControl, MediaFoundationContext};

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let cx = MediaFoundationContext::new().unwrap();

    let devices = cx.enumerate_devices().unwrap();

    for device_info in devices {
        println!("Device: {:?}", device_info);

        let Ok(mut device) = device_info.create_device() else {
            log::error!("...");
            continue;
        };

        let value = device.get_control_value(MFKnownControl::Exposure).unwrap();
        let range = device.get_control_range(MFKnownControl::Exposure).unwrap();
        dbg!(value);
        dbg!(range);
    }
}

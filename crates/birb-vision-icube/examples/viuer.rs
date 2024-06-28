use birb_vision_icube::iCubeContext;



fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let cx = iCubeContext::new()
        .expect("failed to create iCube context, is the SDK installed?");

    cx.init_device_list(|devices| {
        for device in devices {
            println!("Device {}: {:?}", device.sdk_index(), device.name());
        }
    });
}
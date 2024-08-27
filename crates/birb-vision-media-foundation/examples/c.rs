use birb_vision_core::{futures::StreamExt, CameraDevice, CameraDeviceEx};
use birb_vision_media_foundation::MediaFoundationContext;


fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let cx = MediaFoundationContext::new().unwrap();

    let devices = cx.enumerate_devices().unwrap();

    for device_info in devices {
        println!("Device: {:?}", device_info);

        let device = device_info.create_device().unwrap();

        device.start_stream().unwrap();
        device.grab();
        //device.flush_reader().unwrap();
        let mut rx = device.set_stream_callback(Box::new(|e| {
            println!("- Callback: {:?}", e);
        })).unwrap();

        println!("Main Thread: {:?}", std::thread::current().id());
        println!("Waiting for events...");
        std::thread::sleep(std::time::Duration::from_secs(2));
        device.grab();
        device.grab();
        device.grab();
        //pollster::block_on(async {
        //    let mut rx = rx.into_buffered_stream(100, |e| e);
        //    loop {
        //        let e = rx.next().await;
        //        println!("Recv: {:?}", e);
        //    }
        //});
        std::thread::sleep(std::time::Duration::from_secs(2));
        println!("Done waiting");
        //pollster::block_on(async {
        //    for _ in 0..50 {
        //        let start = std::time::Instant::now();
        //        let frame = device.next_frame().await.unwrap();
        //        println!("recv: {:?} in {:?}", frame, start.elapsed());
        //        let frame = frame.into_image().unwrap();
        //        //frame.save(format!("frame.png")).unwrap();
        //        std::thread::sleep(std::time::Duration::from_millis(100));
        //    }
        //});
    }
}
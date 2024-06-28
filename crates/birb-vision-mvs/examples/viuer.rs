use birb_vision_mvs::prelude::*;
use birb_vision::CameraDevice;

fn main() {
    pollster::block_on(async_main());
}

async fn async_main() {
    let cx = MVSContext::new(None).expect("Failed to initialize a MVS context");

    let devices = cx
        .enumerate_devices([TransportLayerType::Usb])
        .expect("Failed to enumerate devices");

    let mut device = devices.into_iter().next().unwrap().into_device(false).unwrap();

    CameraDevice::open(&mut device).await.unwrap();

    device.start_video_stream().await.unwrap();

    for _ in 0..100 {
        let frame = device.receive_frame().await.unwrap();
        let im = frame.as_image().unwrap();

        viuer::print(im, &Default::default()).unwrap();
    }
}
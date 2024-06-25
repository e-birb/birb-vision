use birb_vision_nokhwa::NokhwaCamera;
use image::DynamicImage;
use nokhwa::{pixel_format::RgbFormat, utils::{CameraIndex, RequestedFormat, RequestedFormatType}, Camera as NCamera};
use birb_vision::CameraDevice;

fn main() {
    pollster::block_on(async_main());
}

async fn async_main() {
    let mut last_frame = std::time::Instant::now();

    let camera = NCamera::new(
        CameraIndex::Index(0),
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::None),
    ).unwrap();

    let mut camera = NokhwaCamera::new::<RgbFormat>(camera);

    camera.open().await.unwrap();
    camera.start_video_stream().await.unwrap();

    for _ in 0..100 {
        let frame = camera.receive_frame().await.unwrap().into_owned();

        let now = std::time::Instant::now();
        let elapsed = now - last_frame;
        let fps = 1.0 / elapsed.as_secs_f64();
        last_frame = now;

        println!("FPS: {:.2}", fps);

        let conf = viuer::Config {
            // set offset
            x: 1,
            y: 1,
            // set dimensions
            width: Some(40),
            height: None,
            ..Default::default()
        };
        viuer::print(&frame, &conf).unwrap();
    }

    camera.close().await.unwrap_or_else(|e| {
        println!("Error: {}", e);
    });
}
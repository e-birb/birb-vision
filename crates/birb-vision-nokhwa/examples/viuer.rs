use birb_vision_nokhwa::NokhwaCamera;
use clap::Parser;
use nokhwa::{pixel_format::RgbFormat, utils::{CameraIndex, RequestedFormat, RequestedFormatType}, Camera as NCamera};
use birb_vision::CameraDevice;

#[derive(Parser)]
struct Args {
    #[clap(short, long)]
    fps: bool,

    #[clap(short, long)]
    no_display: bool,
}

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Warn)
        .init();
    pollster::block_on(async_main());
}

async fn async_main() {
    let args = Args::parse();

    let mut last_frame = std::time::Instant::now();

    println!("Available cameras:");
    for camera in nokhwa::query(nokhwa::native_api_backend().unwrap()).unwrap() {
        println!("  - {} - {}", camera.human_name(), camera.description());
    }

    let camera = NCamera::new(
        CameraIndex::Index(0),
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::None),
    ).unwrap();

    let mut camera = NokhwaCamera::new::<RgbFormat>(camera);

    camera
        .open().await
        .expect("Failed to open camera");

    camera
        .start_video_stream().await
        .expect("Failed to start video stream");

    for _ in 0..100 {
        let frame = camera
            .receive_frame().await
            .expect("Failed to receive frame")
            .into_owned();

        let now = std::time::Instant::now();
        let elapsed = now - last_frame;
        let fps = 1.0 / elapsed.as_secs_f64();
        last_frame = now;

        if args.fps {
            println!("FPS: {:.2}", fps);
        }

        let conf = viuer::Config {
            // set offset
            x: 1,
            y: 0,
            // set dimensions
            width: None,
            height: None,
            ..Default::default()
        };
        if !args.no_display {
            viuer::print(&frame.as_image().unwrap(), &conf).unwrap();
        }
    }

    camera
        .close().await
        .unwrap_or_else(|e| {
            // We ignore the close camera error since we are dropping the camera anyway.
            log::error!("Failed to close camera: {:?}", e);
        });
}
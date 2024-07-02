use std::error::Error;

use birb_vision_media_foundation::{CompressedFrame, MediaFoundationContext, PixelFormat, VideoFormatQuery, VideoSubtype};
use windows::Win32::Media::MediaFoundation::MEDIASUBTYPE_RGB24;


fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let cx = MediaFoundationContext::new().unwrap();

    let devices = cx.enumerate_devices().unwrap();

    for device_info in devices {
        println!("Device: {:?}", device_info);

        let mut device = device_info.create_device().unwrap();
        let formats = device.compatible_format_list().unwrap()
            .into_iter()
            .map(|f| f)
            .collect::<Vec<_>>();

        println!("Formats: {:?}", formats.len());
        for f in formats {
            // prints as 32 digits hexadecimal numbers
            if f.recognize_supported_media_subtype().is_none() {
                continue;
            }
            println!("{f:#?}");
        }

        // don't forget to select a supported format, otherwise it will fail
        // TODO add a sorting option
        let selected_format = device.select_format(VideoFormatQuery::any_supported_format()).expect("Failed to select a supported format");
        println!("Selected format: {:?}", selected_format);
        assert_eq!(device.get_current_format().unwrap(), selected_format);

        device.start_stream().unwrap();

        for _ in 0..100 {
            device.flush().unwrap();
            let f = device.receive_and_decode_frame().unwrap();
            viuer::print(&f, &Default::default()).unwrap();
        }
    }
}
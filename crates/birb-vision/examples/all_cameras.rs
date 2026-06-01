use std::{io::Write, time::Duration};

use birb_vision::all_backends;
use birb_vision_core::CameraDeviceEx;
use colored::Colorize;



fn main() -> anyhow::Result<()> {
    let all = all_backends().all_packages();

    let mut frame_counter = 0;

    println!("Found {} backends:", all.len());
    for (id, package) in all {
        println!("- {} ({})", package.display_name.green(), id.cyan());

        let context = match package.build_backend() {
            Ok(context) => context,
            Err(e) => {
                println!("  {}: {}", "Failed to build backend".red(), e);
                continue;
            }
        };

        let devices = match context.enumerate(&context.default_transport_layers()) {
            Ok(devices) => devices,
            Err(e) => {
                println!("  {}: {}", "Failed to enumerate devices".red(), e);
                continue;
            }
        };

        println!("  Found {} devices:", devices.len());

        for device in devices {
            print!("  - {}: ", device.display_name.blue());
            std::io::stdout().flush().unwrap();

            let Some(device) = context.create(&device)? else {
                println!("{}", "Failed to open".red());
                continue;
            };
            println!("{}", "OK".green());
            let properties = device.all_properties()?;
            println!("    {} properties:", properties.len());
            for prop in properties {
                println!("    - {}: {}", prop.display_name.yellow(), prop.description.as_deref().unwrap_or_default());
            }

            device.start_grabbing()?;
            let sample = device.get_one_frame(Duration::from_secs(3));
            let sample = match pollster::block_on(sample) {
                Ok(frame) => frame,
                Err(e) => {
                    println!("    {}: {}", "Failed to grab frame".red(), e);
                    continue;
                }
            };

            let Ok(sample) = sample.try_decode() else {
                println!("  Cannot decode sample");
                continue;
            };

            let sample = match sample {
                Ok(image) => image,
                Err(e) => {
                    println!("    {}: {}", "Failed to decode sample".red(), e);
                    continue;
                }
            };

            frame_counter += 1;
            let count = frame_counter;

            sample.save(format!("frame-{}.png", count))?;
            println!("    Saved frame to frame-{}.png", count);
        }
    }

    Ok(())
}
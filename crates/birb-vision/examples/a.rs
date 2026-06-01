use std::io::Write;

use birb_vision::all_backends;
use birb_vision_core::{ImageSampleBuffer, Sample, StreamEvent};
use colored::Colorize;



fn main() -> anyhow::Result<()> {
    let all = all_backends().all_packages();

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

            let opens = context
                .create(&device)
                .ok()
                .flatten()
                .is_some();
            println!("{}", if opens { "OK".green() } else { "Failed to open".red() });

            let device = context.create(&device)?.unwrap();
            let properties = device.all_properties()?;
            println!("    {} properties:", properties.len());
            for prop in properties {
                println!("    - {}: {}", prop.display_name.yellow(), prop.description.as_deref().unwrap_or_default());
            }

            let (tx, rx) = std::sync::mpsc::channel::<()>();
            device.set_stream_callback(Box::new(move |e| {
                println!("    Event: {:?}", e);
                if let StreamEvent::Sample(sample) = e {
                    let sample = sample.unwrap();
                    let Sample::ImageSample(f) = &sample;
                    let b = &f.buffer;
                    let ImageSampleBuffer::Cow(cow) = b else {
                        println!("Buffer is not a Cow, skipping decode");
                        return;
                    };
                    dbg!(cow.len());
                    dbg!("Buffer type: {:?}", b);
                    let sample = sample.try_decode().unwrap().unwrap();
                    sample.save("sample.png").unwrap();
                }
                tx.send(()).unwrap();
            }))?;
            device.start_grabbing()?;
            rx.recv()?;
            std::thread::sleep(std::time::Duration::from_secs(3));
            device.stop_grabbing()?;
        }
    }

    Ok(())
}
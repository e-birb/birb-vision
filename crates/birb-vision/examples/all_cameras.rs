use std::io::Write;

use birb_vision::all_backends;
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
        }
    }

    Ok(())
}
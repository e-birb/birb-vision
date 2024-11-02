use std::io::Write;

use birb_vision::all_backends;
use colored::Colorize;



fn main() {
    let all = all_backends().all_packages();

    println!("Found {} backends:", all.len());
    for (id, package) in all {
        println!("- {} ({})", package.display_name.green(), id.cyan());

        let context = package
            .build_backend()
            .expect("Failed to initialize backend context");

        let devices = context
            .enumerate(&context.default_transport_layers())
            .expect("Failed to enumerate devices");

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
}
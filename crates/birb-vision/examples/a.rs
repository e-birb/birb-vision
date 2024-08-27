use birb_vision::all_backends;



fn main() {
    let all = all_backends().all_packages();

    for (name, package) in all {
        println!("{}: {:#?}", name, package);
        let devices = package.build_backend().unwrap().enumerate().unwrap();
        println!("Found {} devices:", devices.len());
        for device in devices {
            println!("  - {:#}", device);
        }
    }
}
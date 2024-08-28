use birb_vision::all_backends;



fn main() {
    let all = all_backends().all_packages();

    for (name, package) in all {
        println!("{}: {:#?}", name, package);
        let backend = package.build_backend().unwrap();
        let devices = backend.enumerate(&backend.default_transport_layers()).unwrap();
        println!("Found {} devices:", devices.len());
        for device in devices {
            println!("  - {:#}", device);
        }
    }
}
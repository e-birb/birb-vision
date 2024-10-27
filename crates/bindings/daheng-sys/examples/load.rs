use daheng_sys::SDK;


fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let sdk = unsafe { SDK::auto_select() }.unwrap();

    println!("Loaded SDK: {sdk:?}");

    match sdk {
        SDK::V1(api) => {
            let v = unsafe { api.GXGetLibVersion() };
            let version = unsafe { std::ffi::CStr::from_ptr(v) }.to_str().unwrap();
            println!("SDK version {version}");
        }
        SDK::V2(_) => {
            println!("SDK v2 does not have a version function");
        }
    }
}
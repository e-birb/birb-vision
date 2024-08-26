use std::{io::Write, path::PathBuf};


fn main() {
    generate_config_file();
}

enum SdkTarget {
    Win32,
    Win64,
    Linux
}

impl SdkTarget {
    fn parse(target_triple: &str) -> Self {
        if target_triple.contains("windows") {
            if target_triple.contains("i686") {
                return SdkTarget::Win32;
            } else if target_triple.contains("x86_64") {
                return SdkTarget::Win64;
            }
        }

        if target_triple.contains("linux") {
            return SdkTarget::Linux;
        }

        // TODO add a way to bypass this error and by forcing a specific target
        panic!("Unsupported target triple: {}", target_triple);
    }

    fn dynamic_library_name(&self) -> &'static str {
        use SdkTarget::*;
        match self {
            Win32 | Win64 => "MvCameraControl",
            Linux => "/opt/MVS/lib/64/libMvCameraControl.so",
        }
    }
}

fn generate_config_file() {
    let target_triple = std::env::var("TARGET").unwrap();
    let out_path = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    let target = SdkTarget::parse(&target_triple);

    let config_file = out_path.join("mvs_config.rs");
    let mut config_file = std::fs::File::create(&config_file).unwrap();

    writeln!(
        config_file,
        "pub const DYNAMIC_LIBRARY_NAME: &str = \"{}\";\n",
        target.dynamic_library_name()
    )
    .unwrap();
}
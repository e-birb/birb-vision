use std::{env, io::Write, path::PathBuf};

use bindgen::callbacks::ParseCallbacks;

fn main() {
    let fetaure_libloading = env::var("CARGO_FEATURE_LIBLOADING").is_ok();

    let sdk_target = SdkTarget::detect();

    if !fetaure_libloading {
        // if dynamic loading is disabled, we need to link the lib
        println!("cargo:rustc-link-search={}", sdk_target.lib_dir().display());
        println!("cargo:rustc-link-lib={}", sdk_target.lib_name());
    }

    println!("cargo:rerun-if-changed={}", sdk_target.header());

    let mut builder = bindgen::Builder::default()
        .header(sdk_target.header())
        //.clang_arg("-xc++")
        .clang_args([
            format!("-I{}", sdk_target.include_dir().display()),
            "-x".to_string(),
            "c++".to_string(),
            "-target".to_string(),
            sdk_target.target_triple.clone(),
        ]);

    if fetaure_libloading {
        builder = builder
            .dynamic_library_name("MVS")
            .dynamic_link_require_all(true)
            // use a regex to match function names starting with "MV_CC_"
            .allowlist_item("MV_.*");
    }

    let bindings = builder
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .parse_callbacks(Box::new(Cb))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    bindings
        .write_to_file(out_path.join("mvs_bindings.rs"))
        .expect("Couldn't write bindings!");

    let additional_doc_file = out_path.join("mvs_additional_doc.md");
    std::fs::write(
        additional_doc_file,
        format!(
            r###"
# Additional Info

- local [MVS SDK Development Dir](file:///{development_dir_url}): `{development_dir}`
"###,
            development_dir_url = sdk_target
                .development_dir
                .display()
                .to_string()
                .replace("\\", "/")
                .replace(" ", "%20"),
            development_dir = sdk_target
                .development_dir
                .display()
                .to_string()
                .replace("\\", "/"),
        ),
    )
    .unwrap();

    let config_file = out_path.join("mvs_config.rs");
    let mut config_file = std::fs::File::create(&config_file).unwrap();

    // write pub const DYNAMIC_LIBRARY_NAME: &str = "MvCameraControl";
    writeln!(
        config_file,
        "pub const DYNAMIC_LIBRARY_NAME: &str = \"{}\";\n",
        sdk_target.dynamic_library_name()
    )
    .unwrap();

    //std::fs::write(
    //    &errors_file,
    //    format!("define_error! {{\n{errors}}}"),
    //).unwrap();
}

struct SdkTarget {
    target_triple: String,
    sdk_target_type: SdkTargetType,
    development_dir: PathBuf,
}

impl SdkTarget {
    fn detect() -> Self {
        let target_triple = env::var("TARGET").unwrap();
        let sdk_target_type = SdkTargetType::parse(&target_triple);
        let development_dir = env::var("MVCAM_COMMON_RUNENV").unwrap().into();

        Self {
            target_triple,
            sdk_target_type,
            development_dir,
        }
    }

    fn header(&self) -> &'static str {
        "ffi/mvs.h"
    }

    fn include_dir(&self) -> PathBuf {
        self.development_dir.join("Includes")
    }

    fn lib_dir(&self) -> PathBuf {
        self.development_dir.join(match self.sdk_target_type {
            SdkTargetType::Win32 => "Libraries/win32",
            SdkTargetType::Win64 => "Libraries/win64",
        })
    }

    fn lib_name(&self) -> &'static str {
        "MvCameraControl"
    }

    fn dynamic_library_name(&self) -> &'static str {
        match self.sdk_target_type {
            SdkTargetType::Win32 => self.lib_name(), // TODO maybe add .dll?
            SdkTargetType::Win64 => self.lib_name(), // TODO maybe add .dll?
        }
    }
}

enum SdkTargetType {
    Win32,
    Win64,
}

impl SdkTargetType {
    fn parse(target_triple: &str) -> Self {
        if target_triple.contains("windows") {
            if target_triple.contains("i686") {
                return SdkTargetType::Win32;
            } else if target_triple.contains("x86_64") {
                return SdkTargetType::Win64;
            }
        }

        // TODO add a way to bypass this error and by forcing a specific target
        panic!("Unsupported target triple: {}", target_triple);
    }
}

#[derive(Debug)]
struct Cb;

impl ParseCallbacks for Cb {
    fn process_comment(&self, comment: &str) -> Option<String> {
        let comment = if comment.contains("@~english") {
            comment.split("@~english").last().unwrap().to_string()
        } else {
            'a: {
                let lines = comment.lines().collect::<Vec<_>>();
                for i in 0..lines.len() {
                    if lines[i].trim().is_empty() {
                        if i + 1 < lines.len() {
                            break 'a lines[i + 1..].join("\n");
                        }
                    }
                }

                comment.to_string()
            }
        }
        .replace("[", "\\[")
        .replace("]", "\\]");

        Some(doxygen_rs::transform(&comment))
    }
}

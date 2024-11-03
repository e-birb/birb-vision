use std::{env, path::PathBuf};

const HEADER_V1: &str = "ffi/v1/GxIAPI.h";
const HEADER_V2: &str = "ffi/v2/GxIAPI.h";

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    let target_triple = env::var("TARGET").unwrap();

    // HACK for some reason bindgen and rustfmt freak out on windows after producing
    // an unreasonably large bindings file, then rustfmt stack overflows
    // I think it is related to https://github.com/rust-lang/rust-bindgen/issues/2929
    let target_triple = "x86_64-unknown-linux-gnu".to_string();

    bindgen::Builder::default()
        .header(HEADER_V1)
        .dynamic_library_name("API")
        .dynamic_link_require_all(false)
        .generate_comments(true)
        .clang_args([
            "-target".to_string(), target_triple.clone(),
            "-x".to_string(), "c++".to_string(),
        ])
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(out_path.join("bindings_v1.rs"))
        .expect("Couldn't write bindings!");

    bindgen::Builder::default()
        .header(HEADER_V2)
        .dynamic_library_name("API")
        .dynamic_link_require_all(true)
        .generate_comments(false)
        .clang_args([
            "-target".to_string(), target_triple.clone(),
            "-x".to_string(), "c++".to_string(),
        ])
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(out_path.join("bindings_v2.rs"))
        .expect("Couldn't write bindings!");
}
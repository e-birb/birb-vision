use bindgen::callbacks::ParseCallbacks;
use std::env;
use std::path::PathBuf;

const HEADER: &str = "interfaces/v0/birb-vision-nest/interface.h";

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    std::fs::create_dir_all(out_path.join("bindings")).unwrap();

    bindgen::Builder::default()
        .header(HEADER)
        .dynamic_library_name("Api")
        .dynamic_link_require_all(true)
        .generate_comments(true)
        .parse_callbacks(Box::new(Cb))
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(out_path.join("bindings/v0.rs"))
        .expect("Couldn't write bindings!");
}

#[derive(Debug)]
struct Cb;

impl ParseCallbacks for Cb {
    fn process_comment(&self, comment: &str) -> Option<String> {
        Some(doxygen_rs::transform(comment))
    }

    /// See [`bindgen::callbacks::ParseCallbacks`]
    fn header_file(&self, filename: &str) {
        println!("cargo:rerun-if-changed={}", filename);
    }
}
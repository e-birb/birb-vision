fn main() {
    bindgen::Builder::default()
        .header("interfaces/v0/birb-vision-nest/interface.h")
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file("src/bindings/v0.rs")
        .expect("Couldn't write bindings!");
}
use std::env;
use std::path::PathBuf;
use cbindgen::Style;

fn main() {
    let out_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_language(cbindgen::Language::C)
        .with_style(Style::Tag)
        .generate()
        .expect("Unable to generate C bindings")
        .write_to_file("rigz_core.h");

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:warning=Generated header file: {}", out_dir.join("rigz_core.h").display());
}

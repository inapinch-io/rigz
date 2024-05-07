use std::env;
use std::fs::read_dir;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn main() {
    println!("cargo:rerun-if-changed=../rigz_modules/build.zig");
    println!("cargo:rerun-if-changed=../rigz_modules/build.zig.zon");
    let zig_src_dir = PathBuf::from("../rigz_modules/src");
    for entry in read_dir(zig_src_dir).expect("Failed to read Zig source directory") {
        let entry = entry.expect("Failed to read directory entry");
        println!("cargo:rerun-if-changed={}", entry.path().display());
    }

    let name = "rigz_modules";
    let output = Command::new("zig")
        .current_dir(PathBuf::from("../rigz_modules/"))
        .args(["build"])
        .stderr(Stdio::piped())
        .output()
        .expect("Failed to execute Zig compiler");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);

        eprintln!("Error compiling Zig code: {}", stderr);

        if stderr.contains("unable to execute command") {
            panic!("Compilation failure: Execution error");
        } else if stderr.contains("linker command failed") {
            panic!("Linking failure: Linker command error");
        } else {
            panic!("Compilation failure: Unknown error");
        }
    }

    println!("cargo:rustc-link-search=native={}", "rigz_modules/zig-out/lib");
    println!("cargo:rustc-link-lib=static={}", name);
}

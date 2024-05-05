use std::env;
use std::fs::read_dir;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn main() {
    println!("cargo:rerun-if-changed={}", "../rigz_modules/build.zig");
    println!("cargo:rerun-if-changed={}", "../rigz_modules/build.zig.zon");
    let zig_src_dir = PathBuf::from("../rigz_modules/src");
    for entry in read_dir(zig_src_dir).expect("Failed to read Zig source directory") {
        let entry = entry.expect("Failed to read directory entry");
        println!("cargo:rerun-if-changed={}", entry.path().display());
    }

    let zig_src_path = PathBuf::from("../rigz_modules/src/root.zig");
    let name = "runtime";
    let output = Command::new("zig")
        .args(&[
            "build-lib",
            zig_src_path.to_str().unwrap(),
            "--name",
            name,
            "-static",
            "-fPIE",
            /*
               was required for zig static libs compiled on 0.11.0
                   https://github.com/ziglang/zig/issues/6817
                   - on Mac, Undefined symbols for architecture x86_64:
                       "___zig_probe_stack", referenced from:
               Also fixes - https://gitlab.com/inapinch_rigz/rigz/-/jobs/6778221190
            */
            "-fcompiler-rt",
            // This is to avoid zig requiring building rigz_core directly, X Cargo -> Zig -> Cargo (It should work but I'd rather not find out)
            "-I../rigz_core",
        ])
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

    // TODO: change output to somewhere in OUT_DIR
    println!(
        "cargo:rustc-link-search=native={}",
        env::current_dir()
            .expect("Failed to get current_dir")
            .to_str()
            .unwrap()
    );
    println!("cargo:rustc-link-lib=static={}", name);
}

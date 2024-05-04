use std::env;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn main() {
    let zig_src_path = PathBuf::from("../rigz_plugins/src/root.zig");
    let name = "runtime";

    // Attempt to compile the Zig source file
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
        ])
        .stderr(Stdio::piped()) // Capture stderr for error analysis
        .output()
        .expect("Failed to execute Zig compiler");

    // Check if the command was successful
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Log the detailed compiler error
        eprintln!("Error compiling Zig code: {}", stderr);

        // Determine the type of error from the compiler output
        if stderr.contains("unable to execute command") {
            panic!("Compilation failure: Execution error");
        } else if stderr.contains("linker command failed") {
            panic!("Linking failure: Linker command error");
        } else {
            panic!("Compilation failure: Unknown error");
        }
    }

    // Print cargo metadata directives if compilation was successful
    println!(
        "cargo:rustc-link-search=native={}",
        env::current_dir()
            .expect("Failed to get current_dir")
            .to_str()
            .unwrap()
    );
    println!("cargo:rustc-link-lib=static={}", name);
}

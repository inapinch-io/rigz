use clap_derive::Args;
use log::info;
use std::fs::File;
use std::process::exit;

#[derive(Args, Debug)]
pub struct InitArgs {
    #[arg(long, default_value = "true", action)]
    create_config: bool,
    #[arg(long, default_value = "true", action)]
    create_sample_files: bool,
}

fn create_file(filename: &str, contents: &str) {
    let mut file =
        File::create_new(filename).expect(format!("Failed to create {}", filename).as_str());
    // file.write_all(contents)?;
    info!("created {}", filename)
}

pub(crate) fn init_project(args: InitArgs) -> ! {
    if args.create_config {
        let default_config = "{}";
        create_file("rigz.json", default_config);
    }

    if args.create_sample_files {
        let hello_world = "puts 'Hello World'";
        create_file("hello.rigz", hello_world);
    }

    exit(0)
}

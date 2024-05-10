use clap_derive::Args;
use log::{info, trace};
use std::fs::File;
use std::io::Write;
use std::process::exit;

#[derive(Args, Debug)]
pub struct InitArgs {
    #[arg(long, default_value = "true", action)]
    create_config: bool,
    #[arg(long, default_value = "true", action)]
    create_sample_files: bool,
}

fn create_file(path: &str, contents: &str) {
    let mut file = File::create_new(path).expect(format!("Failed to create {}", path).as_str());
    file.write_all(contents.as_ref())
        .expect(format!("Failed to write contents {}", path).as_str());
    info!("created {}", path)
}

pub(crate) fn init_project(args: InitArgs) -> ! {
    trace!("`init_project`: {:?}", args);
    if args.create_config {
        let default_config = "{}";
        create_file("rigz.json", default_config);

        let default_module_config = r#"
# this file is only required if this repo will become a module
module {
    name = "hello_world"
}
"#;
        create_file("module.rigz", default_module_config);
    }

    if args.create_sample_files {
        let hello_world = "puts 'Hello World'";
        create_file("hello.rigz", hello_world);
    }

    exit(0)
}

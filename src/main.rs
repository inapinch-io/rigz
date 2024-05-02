use std::collections::HashSet;
use std::fs::File;
use std::io::Write;
use clap::{CommandFactory, Parser};
use std::path::PathBuf;
use std::process::exit;
use rigz_runtime::{initialize, Options};
use rigz_runtime::parse::ParseOptions;
use rigz_runtime::run::{run};
use anyhow::Result;
use clap::builder::Str;
use clap_derive::{Args, Subcommand};
use log::info;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(next_line_help = true)]
pub struct CLI {
    #[arg(short, long, value_name = "FILE", env = "RIGZ_CONFIG")]
    config: Option<PathBuf>,

    #[arg(short, long, env = "RIGZ_VERBOSE")]
    verbose: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Init(InitArgs),
    Run(RunArgs),
    // Test(TestArgs),
    // Console(ConsoleArgs),
}

#[derive(Args)]
pub struct InitArgs {

}

#[derive(Args)]
pub struct RunArgs {

}

pub fn create_file<'a, 'b>(filename: &'a str, contents: &'b str) -> &'a str {
    let mut file = File::create_new(filename)
        .expect(format!("Failed to create {}", filename).as_str());
    // file.write_all(contents)?;
    filename
}

pub fn init_project(args: InitArgs) -> ! {
    let mut paths = Vec::new();
    let default_config = "{}";
    paths.push(create_file("rigz.json", default_config));
    let hello_world = "puts 'Hello World'";
    paths.push(create_file("hello.rigz", hello_world));
    for path in paths {
        info!("created {}", path)
    }
    exit(0)
}

fn main() -> Result<()> {
    let cli = CLI::parse();
    if cli.command.is_none() {
        let mut app = CLI::command();
        app.print_help().expect("`print_help` failed");
        exit(0);
    }

    // cli.config_path => Options using serde_json
    let options = Options {
        parse: ParseOptions {
            use_64_bit_numbers: false,
            source_files_patterns: vec![],
            match_options: Default::default(),
        }
    };

    let result = match cli.command.unwrap() {
        Commands::Init(args) => {
            init_project(args)
        }
        Commands::Run(args) => {
            let runtime = initialize(options)?;
            run(&runtime, args.into())?
        }
    };

    Ok(())
}

impl Into<rigz_runtime::run::RunArgs> for RunArgs {
    fn into(self) -> rigz_runtime::run::RunArgs {
        rigz_runtime::run::RunArgs {

        }
    }
}

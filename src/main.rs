use std::collections::HashSet;
use std::fs::File;
use std::io::{Read, Write};
use clap::{CommandFactory, Parser};
use std::path::PathBuf;
use std::process::exit;
use rigz_runtime::{initialize, Options};
use rigz_runtime::parse::ParseOptions;
use rigz_runtime::run::{run};
use anyhow::{anyhow, Result};
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

#[derive(Subcommand, Debug)]
enum Commands {
    Init(InitArgs),
    Run(RunArgs),
    Setup(SetupArgs),
    Test(TestArgs),
    Console(ConsoleArgs),
}

#[derive(Args, Debug)]
pub struct ConsoleArgs {

}

#[derive(Args, Debug)]
pub struct InitArgs {
    #[arg(short, long)]
    create_config: bool,
    #[arg(short, long)]
    create_sample_files: bool
}

impl Default for InitArgs {
    fn default() -> Self {
        InitArgs {
            create_config: true,
            create_sample_files: true,
        }
    }
}

#[derive(Args, Debug)]
pub struct RunArgs {

}

#[derive(Args, Debug)]
pub struct SetupArgs {

}

#[derive(Args, Debug)]
pub struct TestArgs {

}

fn create_file(filename: &str, contents: &str) {
    let mut file = File::create_new(filename)
        .expect(format!("Failed to create {}", filename).as_str());
    // file.write_all(contents)?;
    info!("created {}", filename)
}

pub fn init_project(args: InitArgs) -> ! {
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

fn main() -> Result<()> {
    let cli = CLI::parse();
    if cli.command.is_none() {
        let mut app = CLI::command();
        app.print_help().expect("`print_help` failed");
        exit(0);
    }

    let options = match cli.config {
        None => Options::default(),
        Some(config_path) => {
            let mut file = File::open(config_path)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            serde_json::from_str(contents.as_str())?
        }
    };

    let command = cli.command.unwrap();
    let result = match command {
        Commands::Init(args) => {
            init_project(args)
        }
        _ => {
            let mut runtime = initialize(options)?;
            match command {
                Commands::Setup(args) => {
                    exit(0)
                }
                Commands::Run(args) => {
                    run(&mut runtime, args.into())?
                }
                Commands::Console(args) => {
                    exit(0)
                }
                Commands::Test(args) => {
                    exit(0)
                }
                _ => {
                    return Err(anyhow!("Unimplemented command: {:?}", command))
                }
            }
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

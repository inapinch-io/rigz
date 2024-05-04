use crate::init::{init_project, InitArgs};
use anyhow::anyhow;
use clap_derive::{Args, Subcommand};
use rigz_runtime::run::{run, RunResult};
use rigz_runtime::{initialize, Options};
use std::path::PathBuf;
use std::process::exit;

#[derive(Subcommand, Debug)]
pub enum Commands {
    Init(InitArgs),
    Run(RunArgs),
    Setup(SetupArgs),
    Test(TestArgs),
    Console(ConsoleArgs),
}

#[derive(Args, Debug)]
pub struct ConsoleArgs {}

#[derive(Args, Debug)]
pub struct RunArgs {}

impl Into<rigz_runtime::run::RunArgs> for RunArgs {
    fn into(self) -> rigz_runtime::run::RunArgs {
        rigz_runtime::run::RunArgs {}
    }
}

#[derive(Args, Debug)]
pub struct SetupArgs {}

#[derive(Args, Debug)]
pub struct TestArgs {
    test_directory: PathBuf,
}

impl Commands {
    pub fn handle(self, options: Options) -> anyhow::Result<RunResult> {
        match self {
            Commands::Init(args) => init_project(args),
            _ => {
                let mut runtime = initialize(options)?;
                match self {
                    Commands::Setup(args) => exit(0),
                    Commands::Run(args) => run(&mut runtime, args.into()),
                    Commands::Console(args) => exit(0),
                    Commands::Test(args) => exit(0),
                    _ => return Err(anyhow!("Unimplemented command: {:?}", self)),
                }
            }
        }
    }
}

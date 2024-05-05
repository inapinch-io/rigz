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
pub struct RunArgs {
    #[arg(short, long, action)]
    all_errors_fatal: bool,
    #[arg(short, long, action)]
    ignore_symbol_not_found: bool,
    #[arg(short, long, action)]
    prefer_none_over_prior_result: bool,
}

impl Into<rigz_runtime::run::RunArgs> for RunArgs {
    fn into(self) -> rigz_runtime::run::RunArgs {
        rigz_runtime::run::RunArgs {
            all_errors_fatal: self.all_errors_fatal,
            ignore_symbol_not_found: self.ignore_symbol_not_found,
            prefer_none_over_prior_result: self.prefer_none_over_prior_result,
        }
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
                    Commands::Setup(_args) => exit(0),
                    Commands::Run(args) => run(&mut runtime, args.into()),
                    Commands::Console(_args) => exit(0),
                    Commands::Test(_args) => exit(0),
                    _ => return Err(anyhow!("Unimplemented command: {:?}", self)),
                }
            }
        }
    }
}

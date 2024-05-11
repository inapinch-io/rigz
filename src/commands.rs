use crate::init::{init_project, InitArgs};
use anyhow::anyhow;
use clap_derive::{Args, Subcommand};
use rigz_runtime::run::{initialize_runtime, run, RunResult};
use rigz_runtime::{initialize, Options};
use std::path::PathBuf;
use std::process::exit;
use std::rc::Rc;

#[derive(Subcommand, Debug)]
pub enum Commands {
    Init(InitArgs),
    Run(RunArgs),
    Setup(SetupArgs),
    Test(TestArgs),
    Console(ConsoleArgs),
}

impl Commands {
    pub fn handle(self, options: Options) -> anyhow::Result<RunResult> {
        match self {
            Commands::Init(args) => init_project(args),
            _ => {
                let config = initialize(options)?;
                match self {
                    Commands::Setup(_args) => exit(0),
                    Commands::Run(args) => {
                        let args = args.into();
                        let mut runtime = initialize_runtime(config, Rc::new(args))?;
                        run(&mut runtime, args)
                    }
                    _ => return Err(anyhow!("Unimplemented command: {:?}", self)),
                    // Commands::Console(_args) => exit(0),
                    // Commands::Test(_args) => exit(0),
                }
            }
        }
    }
}

#[derive(Args, Debug)]
pub struct ConsoleArgs {}

#[derive(Args, Debug, Default)]
pub struct RunArgs {
    #[arg(short, long, action)]
    all_errors_fatal: bool,
    #[arg(short, long, action)]
    ignore_symbol_not_found: bool,
    #[arg(short, long, action)]
    prefer_none_over_prior_result: bool,
    #[arg(short, long, action)]
    require_aliases: bool,
}

impl From<RunArgs> for rigz_runtime::run::RunArgs {
    fn from(value: RunArgs) -> Self {
        rigz_runtime::run::RunArgs {
            all_errors_fatal: value.all_errors_fatal,
            ignore_symbol_not_found: value.ignore_symbol_not_found,
            prefer_none_over_prior_result: value.prefer_none_over_prior_result,
            require_aliases: value.require_aliases,
        }
    }
}

#[derive(Args, Debug)]
pub struct SetupArgs {}

#[derive(Args, Debug)]
pub struct TestArgs {
    test_directory: PathBuf,
}

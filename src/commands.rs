use crate::init::{init_project, InitArgs};
use anyhow::anyhow;
use clap_derive::{Args, Subcommand};
use rigz_runtime::run::{initialize_runtime, run, RunResult};
use rigz_runtime::{initialize, Options};
use std::path::PathBuf;
use std::process::exit;
use std::rc::Rc;
use log::error;
use rustyline::config::Configurer;
use rustyline::{ColorMode, DefaultEditor};
use rustyline::error::ReadlineError;

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
                match self {
                    Commands::Setup(_args) => {
                        let _config = initialize(options)?;
                        exit(0)
                    },
                    Commands::Run(args) => {
                        let config = initialize(options)?;
                        let args = args.into();
                        let mut runtime = initialize_runtime(config, Rc::new(args))?;
                        run(&mut runtime, args)
                    }
                    Commands::Test(args) => {
                        if !args.test_directory.exists() {
                            return Err(anyhow!("Test Directory does not exist: {:?}", args.test_directory))
                        }

                        return Err(anyhow!("`test` not implemented"))
                        // exit(0)
                    },
                    Commands::Console(args) => {
                        let config = initialize(options)?;
                        let args = args.into();
                        let mut runtime = initialize_runtime(config, Rc::new(args))?;
                        let mut rl = DefaultEditor::new()?;
                        #[cfg(feature = "with-file-history")]
                        if rl.load_history("history.txt").is_err() {
                            println!("No previous history.");
                        }
                        let mut parser = tree_sitter::Parser::new();
                        let parser_status = parser
                            .set_language(&tree_sitter_rigz::language());

                        if parser_status.is_err() {
                            error!("tree-sitter: Error loading Rigz grammar")
                        }

                        loop {
                            let readline = rl.readline(">> ");
                            match readline {
                                Ok(line) => {
                                    rl.add_history_entry(line.as_str())?;
                                    match line.as_str() {
                                        "exit" => break,
                                        "help" => print_help_string(&line),
                                        &_ => {
                                            println!("Line: {}", line);
                                        }
                                    }
                                },
                                Err(ReadlineError::Interrupted) => {
                                    println!("CTRL-C");
                                    break
                                },
                                Err(ReadlineError::Eof) => {
                                    println!("CTRL-D");
                                    break
                                },
                                Err(err) => {
                                    println!("Error: {:?}", err);
                                    break
                                }
                            }
                        }
                        #[cfg(feature = "with-file-history")]
                        rl.save_history("history.txt");
                        exit(0)
                    },
                    _ => return Err(anyhow!("Unimplemented command: {:?}", self)),
                }
            }
        }
    }
}

impl Into<rigz_runtime::run::RunArgs> for ConsoleArgs {
    fn into(self) -> rigz_runtime::run::RunArgs {
        rigz_runtime::run::RunArgs {
            all_errors_fatal: false,
            ignore_symbol_not_found: false,
            prefer_none_over_prior_result: false,
            require_aliases: false,
        }
    }
}

fn print_help_string(input: &str) {
    if input == "help" {
        println!("help:")
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

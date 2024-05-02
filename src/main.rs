use clap::{CommandFactory, Parser};
use std::path::PathBuf;
use std::process::exit;
use rigz_runtime::{initialize, Options};
use rigz_runtime::parse::ParseOptions;
use rigz_runtime::run::{run};
use anyhow::Result;
use clap_derive::{Args, Subcommand};

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
    Init,
    Run(RunArgs),
    // Test(TestArgs),
    // Console(ConsoleArgs),
}

#[derive(Args)]
pub struct RunArgs {

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

    let runtime = initialize(options)?;

    let result = match cli.command.unwrap() {
        Commands::Init => {
            exit(0)
        }
        Commands::Run(args) => {
            run(runtime, args.into())?
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

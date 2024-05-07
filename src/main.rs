pub mod commands;
pub mod init;
pub mod logger;
pub mod output;

use crate::commands::Commands;
use crate::logger::setup_logger;

use anyhow::Result;
use clap::{CommandFactory, Parser};
use rigz_runtime::Options;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::process::exit;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(next_line_help = true)]
pub struct CLI {
    #[arg(short, long, value_name = "FILE", env = "RIGZ_CONFIG")]
    config: Option<PathBuf>,

    #[arg(short, long, env = "RIGZ_VERBOSE", default_value = "0")]
    verbose: i8,

    #[arg(short, long, env = "RIGZ_OUTPUT")]
    output: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

impl CLI {
    fn options(&self) -> Result<Options> {
        let options = match &self.config {
            None => Options::default(),
            Some(config_path) => {
                let mut file = File::open(config_path)?;
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;
                serde_json::from_str(contents.as_str())?
            }
        };
        Ok(options)
    }
}

fn main() -> Result<()> {
    let cli = CLI::parse();
    if cli.command.is_none() {
        let mut app = CLI::command();
        app.print_help().expect("`print_help` failed");
        exit(0);
    }

    setup_logger(&cli);
    let options = cli.options()?;

    let result = cli.command.unwrap().handle(options)?;
    output::handle_result(cli.output.clone(), result)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::PathBuf;
    use rigz_core::Argument;
    use crate::{CLI, main};
    use crate::commands::{Commands, RunArgs};

    #[test]
    fn cli_works() {
        std::env::set_var("RIGZ_VERBOSE", "3");
        let cli = CLI {
            config: Some(PathBuf::from("local_run.json")),
            verbose: 3,
            output: None,
            command: Some(Commands::Run(RunArgs::default())),
        };
        let options = cli.options().expect("Options Failed");

        let result = cli.command.unwrap().handle(options).expect("Run Failed");
        let mut expected = HashMap::new();
        expected.insert("hello.rigz".to_string(), Argument::None());
        assert_eq!(result.value, expected);
    }
}

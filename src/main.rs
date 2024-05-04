pub mod commands;
pub mod init;

use crate::commands::Commands;
use anyhow::Result;
use clap::{CommandFactory, Parser};
use rigz_runtime::Options;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::exit;

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

    let result = cli.command.unwrap().handle(options)?;

    Ok(())
}

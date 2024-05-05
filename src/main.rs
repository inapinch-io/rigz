pub mod commands;
pub mod init;

use crate::commands::Commands;
use anyhow::{anyhow, Result};
use clap::{CommandFactory, Parser};
use rigz_core::{Argument, ArgumentMap, ArgumentVector, Function};
use rigz_runtime::run::RunResult;
use rigz_runtime::Options;
use serde::Serialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::process::exit;
use log::{info, LevelFilter, warn};

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

#[derive(Clone, Default, Debug)]
pub enum OutputFormat {
    #[default]
    PRINT,
    JSON,
    LOG,
}

use log::{Record, Level, Metadata};

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

static LOGGER: SimpleLogger = SimpleLogger;

fn main() -> Result<()> {
    let cli = CLI::parse();
    if cli.command.is_none() {
        let mut app = CLI::command();
        app.print_help().expect("`print_help` failed");
        exit(0);
    }

    // TODO: support custom loggers
    log::set_logger(&LOGGER).unwrap_or(());
    match cli.verbose {
        i8::MIN => {
            log::set_max_level(LevelFilter::Off)
        }
        -1 => {
            log::set_max_level(LevelFilter::Error)
        }
        0 => {
            log::set_max_level(LevelFilter::Warn)
        }
        1 => {
            log::set_max_level(LevelFilter::Info)
        }
        2 => {
            log::set_max_level(LevelFilter::Debug)
        }
        3 => {
            log::set_max_level(LevelFilter::Trace)
        }
        _ => {
            log::set_max_level(LevelFilter::Warn);
            warn!("Invalid `cli.verbose`: {}, default is Warn", cli.verbose);
        }
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
    let format = match cli.output.clone() {
        None => OutputFormat::default(),
        Some(f) => {
            let fmt = f.trim().to_lowercase();
            if fmt == "json" {
                OutputFormat::JSON
            } else {
                return Err(anyhow!("Invalid Format: `{}`", fmt));
            }
        }
    };
    match format {
        OutputFormat::PRINT => {
            println!("Results:");
            for (file, value) in result.value {
                println!("\t{}: {}", file, value)
            }
        }
        OutputFormat::LOG => {
            info!("Results:");
            for (file, value) in result.value {
                info!("\t{}: {}", file, value)
            }
        }
        _ => {
            let output: OutputValue = result.into();
            match format {
                OutputFormat::JSON => {
                    let contents = serde_json::to_string_pretty(&output)?;
                    println!("{}", contents)
                }
                _ => {
                    return Err(anyhow!(
                        "Unsupported Output Format {:?}",
                        cli.output.clone()
                    ))
                }
            }
        }
    }

    Ok(())
}

#[derive(Serialize)]
pub enum OutputValue {
    None,
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Bool(bool),
    String(String),
    Object(HashMap<String, OutputValue>),
    List(Vec<OutputValue>),
    Error(String),
}

impl From<RunResult> for OutputValue {
    fn from(value: RunResult) -> Self {
        let mut results = HashMap::new();
        for (k, v) in value.value {
            results.insert(k, v.into());
        }
        OutputValue::Object(results)
    }
}

impl From<Argument> for OutputValue {
    fn from(value: Argument) -> Self {
        match value {
            Argument::None() => OutputValue::None,
            Argument::Int(i) => OutputValue::Int(i),
            Argument::Long(l) => OutputValue::Long(l),
            Argument::Float(f) => OutputValue::Float(f),
            Argument::Double(d) => OutputValue::Double(d),
            Argument::Bool(b) => OutputValue::Bool(b),
            Argument::String(s) => OutputValue::String(s.into()),
            Argument::Object(o) => OutputValue::Object(to_object(o)),
            Argument::List(l) => OutputValue::List(to_list(l)),
            Argument::FunctionCall(f) => OutputValue::Object(fn_to_object(f)),
            Argument::Error(e) => OutputValue::Error(e.into()),
        }
    }
}

fn fn_to_object(function: Function) -> HashMap<String, OutputValue> {
    todo!()
}

fn to_list(arg: ArgumentVector) -> Vec<OutputValue> {
    todo!()
}

fn to_object(arg: ArgumentMap) -> HashMap<String, OutputValue> {
    todo!()
}

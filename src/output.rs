use anyhow::anyhow;
use log::info;
use rigz_core::{Argument, ArgumentMap, ArgumentVector, Function};
use rigz_runtime::run::RunResult;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize)]
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

#[derive(Clone, Default, Debug)]
pub enum OutputFormat {
    #[default]
    PRINT,
    JSON,
    LOG,
}

pub fn handle_result(output: Option<String>, result: RunResult) -> anyhow::Result<()> {
    let format = match output {
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
                _ => return Err(anyhow!("Unsupported Output Format {:?}", output.clone())),
            }
        }
    }
    Ok(())
}

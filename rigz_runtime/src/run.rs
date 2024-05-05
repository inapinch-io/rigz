use crate::Runtime;
use anyhow::{anyhow, Result};
use log::{info, warn};
use rigz_core::{Argument, ArgumentDefinition, ArgumentMap, Function};
use rigz_parse::{Definition, Element, FunctionCall, Object, Value};
use std::collections::HashMap;

#[derive(Debug)]
pub struct RunArgs {
    pub all_errors_fatal: bool,
    pub ignore_symbol_not_found: bool,
    pub prefer_none_over_prior_result: bool,
}

pub struct RunResult {
    pub value: HashMap<String, Argument>,
}

pub fn run(runtime: &Runtime, args: RunArgs) -> Result<RunResult> {
    let mut value = HashMap::new();
    for (file, ast) in &runtime.asts {
        let mut prior_result = Argument::None();
        info!("Running {}", file);
        for element in &ast.elements {
            match element {
                Element::FunctionCall(fc) => {
                    prior_result = unsafe { call_function(runtime, fc, prior_result, &args)? };
                }
                _ => return Err(anyhow!("Invalid Element in root of AST: {:?}", element)),
            }
        }
        value.insert(file.to_string(), prior_result);
    }
    Ok(RunResult { value })
}

unsafe fn call_function(
    runtime: &Runtime,
    fc: &FunctionCall,
    prior_result: Argument,
    config: &RunArgs,
) -> Result<Argument> {
    let (args, def) = convert(fc)?;
    let symbol = fc.identifier.clone();
    let result = unsafe { runtime.invoke_symbol(&symbol, args, def, &prior_result, config)? };
    match result {
        Argument::None() => {
            if config.prefer_none_over_prior_result {
                Ok(Argument::None())
            } else {
                Ok(prior_result)
            }
        }
        Argument::FunctionCall(_) => {
            todo!()
        }
        _ => Ok(result),
    }
}

fn convert(function_call: &FunctionCall) -> Result<(Vec<Argument>, Option<ArgumentDefinition>)> {
    let mut args = to_args(&function_call.args)?;
    let mut definition = None;
    if function_call.definition.is_some() {
        let raw = function_call.definition.clone();
        definition = raw.map(|def| match def {
            Definition::Object(o) => ArgumentDefinition::One(
                to_map(&o).expect("Failed to convert definition into Object"),
            ),
            Definition::List(l) => {
                let elements = l.0.clone();
                ArgumentDefinition::Many(
                    to_args(&elements)
                        .expect("Failed to convert definition into List")
                        .into(),
                )
            }
        })
    }
    Ok((args, definition))
}

fn to_args(elements: &Vec<Element>) -> Result<Vec<Argument>> {
    let mut args = Vec::new();
    for arg in elements {
        args.push(element_to_arg(arg)?);
    }
    Ok(args)
}

fn element_to_arg(element: &Element) -> Result<Argument> {
    let argument = match element {
        Element::Value(v) => match v {
            Value::Int(i) => Argument::Int(i.clone()),
            Value::Long(l) => Argument::Long(l.clone()),
            Value::Float(f) => Argument::Float(f.clone()),
            Value::Double(d) => Argument::Double(d.clone()),
            Value::Bool(b) => Argument::Bool(b.clone()),
            Value::String(s) => Argument::String(s.as_str().into()),
            Value::Object(o) => Argument::Object(to_map(o)?),
            Value::List(l) => {
                let elements = l.0.clone();
                Argument::List(to_args(&elements)?.into())
            }
            Value::FunctionCall(fc) => Argument::FunctionCall(Function { a: 0 }),
            Value::None => Argument::None(),
        },
        _ => return Err(anyhow!("Unsupported Argument Type {:?}", element)),
    };
    Ok(argument)
}

fn to_map(object: &Object) -> Result<ArgumentMap> {
    let mut internal = HashMap::new();
    for (k, v) in &object.0 {
        internal.insert(k.clone(), element_to_arg(v)?);
    }
    Ok(internal.into())
}

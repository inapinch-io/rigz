use crate::Runtime;
use anyhow::{anyhow, Result};
use log::{info, warn};
use rigz_core::{Argument, ArgumentDefinition, Function, Map};
use rigz_parse::{Definition, Element, FunctionCall, Object, Value};
use std::collections::HashMap;

pub struct RunArgs {}

pub struct RunResult {}

pub fn run(runtime: &Runtime, args: RunArgs) -> Result<RunResult> {
    for (file, ast) in &runtime.asts {
        info!("Running {}", file);
        let mut prior_result = Argument::None();
        for element in &ast.elements {
            match element {
                Element::FunctionCall(fc) => {
                    prior_result = unsafe {
                        call_function(runtime, fc, prior_result)?
                    };
                }
                _ => return Err(anyhow!("Invalid Element in root of AST: {:?}", element)),
            }
        }
    }
    Ok(RunResult {})
}

unsafe fn call_function(runtime: &Runtime, fc: &FunctionCall, prior_result: Argument) -> Result<Argument> {
    let (args, def) = convert(fc)?;
    let symbol = fc.identifier.clone();
    let result = unsafe {
        runtime.invoke_symbol(&symbol, args, def, prior_result)?
    };
    match result {
        Argument::None() => Ok(Argument::None()),
        Argument::FunctionCall(_) => {
            todo!()
        }
        Argument::Object(_) => {
            todo!()
        }
        Argument::List(_) => {
            todo!()
        }
        _ => {
            Ok(result)
        }
    }
}

fn convert(function_call: &FunctionCall) -> Result<(Vec<Argument>, Option<ArgumentDefinition>)> {
    let mut args = to_args(&function_call.args)?;
    let mut definition = None;
    if function_call.definition.is_some() {
        let raw = function_call.definition.clone();
        definition = raw.map(|def| match def {
            Definition::Object(o) => ArgumentDefinition::One(to_map(&o).expect("Failed to convert definition into Object")),
            Definition::List(l) => {
                let elements = l.0.clone();
                ArgumentDefinition::Many(to_args(&elements).expect("Failed to convert definition into List").into())
            },
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
            Value::Symbol(s) => Argument::Symbol(s.as_str().into()),
            Value::None => Argument::None(),
        },
        _ => return Err(anyhow!("Unsupported Argument Type {:?}", element)),
    };
    Ok(argument)
}

fn to_map(object: &Object) -> Result<Map> {
    let mut internal = HashMap::new();
    for (k, v) in &object.0 {
        internal.insert(k.clone(), element_to_arg(v)?);
    }
    Ok(internal.into())
}

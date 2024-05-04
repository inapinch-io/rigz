use crate::Runtime;
use anyhow::{anyhow, Result};
use log::info;
use rigz_core::{Argument, ArgumentDefinition, Function, Map};
use rigz_parse::{Definition, Element, FunctionCall, Identifier, List, Object, Value};
use std::collections::HashMap;

pub struct RunArgs {}

pub struct RunResult {}

pub fn run(runtime: &Runtime, args: RunArgs) -> Result<RunResult> {
    for (file, ast) in &runtime.asts {
        info!("Running {}", file);
        for element in &ast.elements {
            match element {
                Element::FunctionCall(fc) => {
                    let (args, def) = convert(fc)?;
                    let symbol = fc.identifier.clone();
                    unsafe {
                        runtime
                            .invoke_symbol(&symbol, args, def)
                            .expect(format!("Invocation Failed `{}`", symbol).as_str())
                    }
                }
                _ => return Err(anyhow!("Invalid Element in root of AST: {:?}", element)),
            }
        }
    }
    Ok(RunResult {})
}

fn convert(function_call: &FunctionCall) -> Result<(Vec<Argument>, Option<ArgumentDefinition>)> {
    let mut args = to_args(&function_call.args)?;
    let mut definition = None;
    if function_call.definition.is_some() {
        let raw = function_call.definition.clone();
        definition = raw.map(|def| match def {
            Definition::Object(o) => todo!(),
            Definition::List(l) => todo!(),
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

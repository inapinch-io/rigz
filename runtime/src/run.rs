use crate::{Runtime, RuntimeConfig};
use anyhow::{anyhow, Result};
use log::{info, warn};
use rigz_core::{Argument, FunctionCall, RuntimeStatus};
use rigz_parse::{ASTFunctionCall, Definition, Element, Object, Value};
use serde::Serialize;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone, Default, Debug, Serialize, Copy)]
pub struct RunArgs {
    pub all_errors_fatal: bool,
    pub ignore_symbol_not_found: bool,
    pub prefer_none_over_prior_result: bool,
    pub require_aliases: bool,
}

pub struct RunResult {
    pub value: HashMap<String, Argument>,
}

pub fn initialize_runtime(config: RuntimeConfig, args: Rc<RunArgs>) -> Result<Runtime> {
    let mut modules = HashMap::with_capacity(config.modules.len());
    let mut globals = HashMap::new();
    let mut lookup = Vec::new();
    let base_config = config.initialize_args(args.clone());
    for definition in config.modules {
        let module = definition.to_module(args.clone())?;
        let name = module.name().to_string();
        info!("Initializing {}", name);
        match module.initialize(base_config) {
            RuntimeStatus::Ok(_) => {}
            RuntimeStatus::NotFound => {
                info!("Not Initialization Method for {}", name);
            }
            RuntimeStatus::Err(e) => return Err(anyhow!("Module initialization failed - {}", e)),
        }
        match modules.insert(name, module) {
            None => {}
            Some(old) => {
                warn!("Overwrote Module {}", old.name());
            }
        };
    }
    Ok(Runtime {
        asts: config.asts,
        modules,
        globals,
        lookup,
    })
}

pub fn run(runtime: &Runtime, args: RunArgs) -> Result<RunResult> {
    let mut value = HashMap::with_capacity(runtime.asts.len());
    for (file, ast) in &runtime.asts {
        let mut prior_result = Argument::None;
        info!("Running {}", file);
        for element in &ast.elements {
            match element {
                Element::FunctionCall(fc) => {
                    prior_result = call_function(runtime, convert(fc)?, prior_result, &args)?;
                }
                _ => return Err(anyhow!("Invalid Element in root of AST: {:?}", element)),
            }
        }
        value.insert(file.to_string(), prior_result);
    }
    Ok(RunResult { value })
}
fn call_function(
    runtime: &Runtime,
    fc: FunctionCall,
    prior_result: Argument,
    config: &RunArgs,
) -> Result<Argument> {
    let result = runtime.invoke_symbol(
        fc.name.as_str(),
        fc.args,
        fc.definition,
        &prior_result,
        config,
    )?;
    match result {
        Argument::None => {
            if config.prefer_none_over_prior_result {
                Ok(Argument::None)
            } else {
                Ok(prior_result)
            }
        }
        Argument::FunctionCall(fc) => call_function(runtime, fc, prior_result, config),
        _ => Ok(result),
    }
}

fn convert(function_call: &ASTFunctionCall) -> Result<FunctionCall> {
    let args = to_args(&function_call.args)?;
    let mut definition = rigz_core::Definition::None;
    if function_call.definition.is_some() {
        let raw = function_call.definition.clone();
        definition = raw
            .map(|def| match def {
                Definition::Object(o) => rigz_core::Definition::One(
                    to_map(&o).expect("Failed to convert definition into Object"),
                ),
                Definition::List(l) => {
                    let elements = l.0.clone();
                    rigz_core::Definition::Many(
                        to_args(&elements).expect("Failed to convert definition into List"),
                    )
                }
            })
            .unwrap()
    }
    Ok(FunctionCall {
        name: function_call.identifier.to_string(),
        args,
        definition,
    })
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
            Value::Int(i) => Argument::Int(*i),
            Value::Long(l) => Argument::Long(*l),
            Value::Float(f) => Argument::Float(*f),
            Value::Double(d) => Argument::Double(*d),
            Value::Bool(b) => Argument::Bool(*b),
            Value::String(s) => Argument::String(s.as_str().into()),
            Value::Object(o) => Argument::Object(to_map(o)?),
            Value::List(l) => {
                let elements = l.0.clone();
                Argument::List(to_args(&elements)?)
            }
            Value::FunctionCall(fc) => Argument::FunctionCall(convert(fc)?),
            Value::None => Argument::None,
        },
        _ => return Err(anyhow!("Unsupported Argument Type {:?}", element)),
    };
    Ok(argument)
}

fn to_map(object: &Object) -> Result<HashMap<String, Argument>> {
    let mut internal = HashMap::new();
    for (k, v) in &object.0 {
        internal.insert(k.clone(), element_to_arg(v)?);
    }
    Ok(internal)
}

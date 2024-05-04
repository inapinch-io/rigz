use crate::Runtime;
use anyhow::{anyhow, Result};
use log::info;
use rigz_core::{Argument, ArgumentDefinition};
use rigz_parse::{Definition, Element, FunctionCall, Identifier, Value};

pub struct RunArgs {

}

pub struct RunResult {

}

pub fn run(runtime: &Runtime, args: RunArgs) -> Result<RunResult> {
    for (file, ast) in &runtime.asts {
        info!("Running {}", file);
        for element in &ast.elements {
            match element {
                Element::FunctionCall(fc) => {
                    let (args, def) = convert(fc)?;
                    let symbol = match fc.identifier.clone() {
                        Identifier::Symbol(s) => s.clone(),
                        Identifier::Default(s) => s.clone(),
                    };
                    unsafe {
                        runtime.invoke_symbol(&symbol, args, def)
                            .expect(format!("Invocation Failed `{}`", symbol).as_str())
                    }
                }
                _ => {
                    return Err(anyhow!("Invalid Element in root of AST: {:?}", element))
                }
            }
        }
    }
    Ok(RunResult {})
}

fn convert(function_call: &FunctionCall) -> Result<(Vec<Argument>, Option<ArgumentDefinition>)> {
    let mut args = Vec::new();
    let mut definition = None;
    if function_call.definition.is_some() {
        let raw = function_call.definition.clone();
        definition = raw.map(|def| {
            match def {
                Definition::Object(o) => todo!(),
                Definition::List(l) => todo!(),
            }
        })
    }
    for arg in &function_call.args {
        let argument = match arg {
            Element::Value(v) => {
                match v {
                    Value::Int(i) => {
                        Argument::Int(i.clone())
                    }
                    Value::Long(l) => {
                        Argument::Long(l.clone())
                    }
                    Value::Float(f) => {
                        Argument::Float(f.clone())
                    }
                    Value::Double(d) => {
                        Argument::Double(d.clone())
                    }
                    Value::Bool(b) => {
                        Argument::Bool(b.clone())
                    }
                    Value::String(s) => {
                        Argument::String(s.as_str().into())
                    }
                    Value::Object(o) => {
                        todo!()
                    }
                    Value::List(l) => {
                        todo!()
                    }
                    Value::FunctionCall(fc) => {
                        todo!()
                    }
                    Value::Symbol(s) => {
                        Argument::Symbol(s.as_str().into())
                    }
                    Value::None => Argument::None()
                }
            }
            _ => {
                return Err(anyhow!("Unsupported Argument Type {:?}", arg))
            }
        };
        args.push(argument)
    }
    Ok((args, definition))
}
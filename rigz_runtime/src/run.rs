use crate::{Argument, ArgumentDefinition, Runtime};
use anyhow::{anyhow, Result};
use log::info;
use rigz_parse::{Element, FunctionCall, Identifier};

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
                    runtime.invoke_symbol(&symbol, args, def)
                        .expect(format!("Invocation Failed `{}`", symbol).as_str())
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
        definition = raw.map(|def| ArgumentDefinition {})
    }
    for arg in &function_call.args {
        match arg {
            Element::FunctionCall(_) => {}
            Element::Identifier(_) => {}
            Element::Value(_) => {}
            Element::Object(_) => {}
            Element::List(_) => {}
            Element::Int(_) => {}
            Element::Long(_) => {}
            Element::Float(_) => {}
            Element::Double(_) => {}
            Element::Bool(_) => {}
            Element::String(_) => {}
            Element::Symbol(_) => {}
            _ => {
                return Err(anyhow!("Unsupported Argument Type {:?}", arg))
            }
        }
        args.push(Argument {

        })
    }
    Ok((args, definition))
}
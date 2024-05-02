use crate::Runtime;
use anyhow::{anyhow, Result};
use rigz_parse::{Element, Identifier};

pub struct RunArgs {

}

pub struct RunResult {

}

pub fn run(runtime: &Runtime, args: RunArgs) -> Result<RunResult> {
    for element in &runtime.ast.elements {
        match element {
            Element::FunctionCall(fc) => {
                match &fc.identifier {
                    Identifier::Symbol(symbol) => {
                        let symbol = runtime.symbols.get(symbol);
                        if symbol.is_none() {
                            return Err(anyhow!("Symbol not Found: {:?}", symbol))
                        }
                    }
                    Identifier::Default(func) => {
                        let function = runtime.get_function(func);
                        if function.is_none() {
                            return Err(anyhow!("Function not Found: {:?}", func))
                        }

                    }
                }
            }
            _ => {
                return Err(anyhow!("Invalid Element in root of AST: {:?}", element))
            }
        }
    }
    Ok(RunResult {})
}
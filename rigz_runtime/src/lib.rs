pub mod parse;
pub mod run;
pub mod modules;

use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::ops::Deref;
use anyhow::{anyhow, Result};
use rigz_parse::{AST, Element, FunctionCall};
use serde::Deserialize;
use crate::modules::{Module, ModuleOptions};
use crate::parse::{parse_source_files, ParseOptions};

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[derive(Clone, Default, Deserialize)]
pub struct Options {
    pub parse: Option<ParseOptions>,
    pub disable_std_lib: Option<bool>,
    pub modules: Option<Vec<ModuleOptions>>
}

#[repr(C)]
pub struct Runtime<'a> {
    asts: HashMap<String, AST>,
    symbols: HashMap<String, Symbol>,
    modules: HashMap<String, Module<'a>>
}

impl Runtime<'_> {
    pub fn invoke_symbol(&self, name: &String, arguments: Vec<Argument>, definition: Option<ArgumentDefinition>) -> Result<()> {
        let mut symbol = self.symbols.get(name)
            .expect(format!("Symbol Not Found: `{}`", name).as_str());
        symbol.invoke(self, arguments, definition)
    }
}

#[derive(Debug)]
pub struct Function {

}

#[repr(C)]
pub struct Symbol {
    method: Box<dyn Fn(&Runtime, Vec<Argument>, Option<ArgumentDefinition>) -> Result<()>>
}

impl Debug for Symbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Symbol [method: dyn FnMut(&mut Runtime, Vec<Argument>, Option<ArgumentDefinition>)]")
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct Argument {

}

#[derive(Debug)]
#[repr(C)]
pub struct ArgumentDefinition {

}

impl Symbol {
    pub fn invoke(&self, runtime: &Runtime, arguments: Vec<Argument>, definition: Option<ArgumentDefinition>) -> Result<()> {
        let method= self.method.deref();
        method(runtime, arguments, definition)
    }
}

fn initialize_modules(options: Options) -> Result<(HashMap<String, Module<'static>>, HashMap<String, Symbol>)> {
    let mut symbols = HashMap::new();
    let module_config = match &options.modules {
        None => Vec::new(),
        Some(m) => {
            let mut base = if !options.disable_std_lib.unwrap_or(false) {
                let mut with_lib = Vec::new();
                with_lib.push(ModuleOptions {
                    name: "std".to_string(),
                    source: "https://gitlab.com/inapinch/rigz.git".to_string(),
                    sub_folder: Some("rigz_lib".to_string()),
                    dist: None, // TODO: Use dist once std lib is ironed out and stored in CDN
                    metadata: None,
                    config: None,
                });
                with_lib
            } else {
                Vec::new()
            };
            for config in m {
                base.push(ModuleOptions {
                    name: config.name.clone(),
                    source: config.source.clone(),
                    sub_folder: config.sub_folder.clone(),
                    dist: config.dist.clone(),
                    metadata: config.metadata.clone(),
                    config: config.config.clone(),
                })
            }
            base
        },
    };
    let mut modules = HashMap::new();
    for m in module_config {
        let name = m.name.clone();
        let module = m.download().expect(format!("Failed to Download Module {}", name).as_str());
        unsafe {
            module.init(&mut symbols).expect(format!("Failed to Initialize Module {}", name).as_str());
        }
        modules.insert(name, module);
    }
    Ok((modules, symbols))
}

pub fn initialize(options: Options) -> Result<Runtime<'static>> {
    let asts = parse_source_files(options.parse.clone().unwrap_or(ParseOptions::default()))?;
    let options = options.clone();
    let (modules, symbols) = initialize_modules(options).expect("Failed to initialize modules");

    Ok(Runtime { asts, symbols, modules })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

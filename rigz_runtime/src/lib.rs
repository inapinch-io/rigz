pub mod modules;
pub mod parse;
pub mod run;

use crate::modules::{initialize_module, invoke_symbol, module_runtime, ModuleOptions};
use crate::parse::{parse_source_files, ParseOptions};
use anyhow::{anyhow, Result};
use rigz_core::{Argument, ArgumentDefinition, Vector};
use rigz_parse::AST;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::ops::Deref;

#[derive(Clone, Default, Deserialize)]
pub struct Options {
    pub parse: Option<ParseOptions>,
    pub disable_std_lib: Option<bool>,
    pub modules: Option<Vec<ModuleOptions>>,
}

#[repr(C)]
pub struct Runtime {
    asts: HashMap<String, AST>,
}

impl Runtime {
    pub unsafe fn invoke_symbol(
        &self,
        name: &String,
        arguments: Vec<Argument>,
        definition: Option<ArgumentDefinition>,
    ) -> Result<()> {
        let status = invoke_symbol(name.as_str(), arguments.into(), definition.unwrap_or(ArgumentDefinition::Empty())).status;
        match status {
            0 => Ok(()),
            -1 => return Err(anyhow!("Symbol Not Found {}", name)),
            _ => {
                return Err(anyhow!(
                    "Something went wrong: {} exited with {}",
                    name,
                    status
                ))
            }
        }
    }
}

fn initialize_modules(options: Options) -> Result<()> {
    let mut module_runtime = unsafe { module_runtime() };
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
        }
    };
    for m in module_config {
        let name = m.name.clone();
        let module = m
            .download()
            .expect(format!("Failed to Download Module {}", name).as_str());
        unsafe {
            let status = initialize_module(&mut module_runtime, module).status;
            match status {
                0 => continue,
                -1 => return Err(anyhow!("Module Not Found {}", name)),
                _ => {
                    return Err(anyhow!(
                        "Something went wrong: {} exited with {}",
                        name,
                        status
                    ))
                }
            }
        }
    }
    Ok(())
}

pub fn initialize(options: Options) -> Result<Runtime> {
    let asts = parse_source_files(options.parse.clone().unwrap_or(ParseOptions::default()))?;
    let options = options.clone();
    initialize_modules(options).expect("Failed to initialize modules");

    Ok(Runtime { asts })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hello_world_options() -> Options {
        Options {
            parse: Some(ParseOptions {
                use_64_bit_numbers: None,
                source_files: vec!["../examples/hello_world/hello.rigz".to_string()],
                glob_options: None,
            }),
            disable_std_lib: None,
            modules: None,
        }
    }

    #[test]
    fn default_initialize_works() {
        let result = initialize(hello_world_options()).expect("Failed to initialize");
        assert_eq!(result.asts.is_empty(), false);
    }
}

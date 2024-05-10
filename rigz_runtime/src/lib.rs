pub mod modules;
pub mod parse;
pub mod run;

use crate::modules::{ModuleDefinition, ModuleOptions};
use crate::parse::{parse_source_files, ParseOptions};
use crate::run::RunArgs;
use anyhow::{anyhow, Error, Result};
use log::{trace, warn};
use rigz_core::{Argument, Definition, Module, RuntimeStatus};
use rigz_parse::AST;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Clone, Default, Deserialize)]
pub struct Options {
    pub cache_directory: Option<String>,
    pub disable_std_lib: Option<bool>,
    pub modules: Option<Vec<ModuleOptions>>,
    pub parse: Option<ParseOptions>,
}

pub struct RuntimeConfig {
    asts: HashMap<String, AST>,
    pub modules: Vec<ModuleDefinition>,
}

pub struct Runtime {
    asts: HashMap<String, AST>,
    pub modules: Vec<Box<dyn Module>>,
}

pub enum RunResult<T> {
    Ok(T),
    NotFound,
    Err(Error),
}

impl Runtime {
    pub fn invoke_symbol(
        &self,
        name: &str,
        arguments: Vec<Argument>,
        definition: Definition,
        prior_result: &Argument,
        config: &RunArgs,
    ) -> Result<Argument> {
        if config.require_aliases {
            // let module = self.module(name)?;
            // check global symbols
            // if name contains '.' split and try to find module
            todo!()
        }

        let mut actual_result = None;
        for module in &self.modules {
            let module_name = module.name();
            trace!("Checking `{}` in Module: {}", name, module_name);
            let result = match module.function_call(
                name,
                arguments.clone(),
                definition.clone(),
                prior_result.clone(),
             ) {
                RuntimeStatus::Ok(a) => a,
                RuntimeStatus::NotFound => {
                    warn!("Not Found: {}", name);
                    continue
                },
                RuntimeStatus::Err(e) => {
                    return Err(anyhow!("Function Invocation Failed: {} {}", name, e))
                }
            };
            actual_result = Some(result);
            break;
        }

        let result = actual_result.expect(format!("Failed to find function: {}", name).as_str());
        Ok(result)
    }
}

pub fn initialize(options: Options) -> Result<RuntimeConfig> {
    let asts = parse_source_files(options.parse.clone().unwrap_or(ParseOptions::default()))?;
    let modules = setup_modules(options)?;
    Ok(RuntimeConfig { asts, modules })
}

fn create_cache_dir(options: &Options) -> String {
    let cache_directory = options
        .cache_directory
        .clone()
        .unwrap_or(".rigz/cache/modules".to_string());
    std::fs::create_dir_all(cache_directory.as_str())
        .expect(format!("Failed to create cache directory: {}", cache_directory).as_str());
    cache_directory
}

fn setup_modules(options: Options) -> Result<Vec<ModuleDefinition>> {
    let cache_directory = create_cache_dir(&options);
    let mut modules = Vec::new();

    let mut base_modules = options.modules.unwrap_or(Vec::new());
    if !options.disable_std_lib.unwrap_or(false) {
        base_modules.append(ModuleOptions::default_options().as_mut())
    }
    for module in base_modules {
        let name = module.name.as_str();
        let definition = module
            .download(PathBuf::from(cache_directory.clone()))
            .expect(format!("Failed to Download Module {}", name).as_str());
        modules.push(definition);
    }
    Ok(modules)
}

pub(crate) fn path_to_string(path: &PathBuf) -> Result<String> {
    let str = match path.to_str() {
        None => return Err(anyhow!("Unable to convert {:?} to String", path)),
        Some(s) => s.to_string(),
    };
    Ok(str)
}

#[cfg(test)]
mod tests {
    use log::LevelFilter;
    use super::*;

    fn hello_world_options() -> Options {
        log::set_max_level(LevelFilter::Trace);
        Options {
            cache_directory: None,
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

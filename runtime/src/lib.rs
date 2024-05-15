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
    pub modules: HashMap<String, Box<dyn Module>>,
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

        match self.attempt_call_module_function(name, &arguments, &definition, prior_result) {
            RuntimeStatus::Ok(a) => return Ok(a),
            RuntimeStatus::NotFound => {}
            RuntimeStatus::Err(e) => return Err(anyhow!("Function Call Failed - {}", e)),
        }

        // fallback, check each module
        let mut actual_result = None;
        for (module_name, module) in &self.modules {
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
                    continue;
                }
                RuntimeStatus::Err(e) => {
                    return Err(anyhow!("Function Invocation Failed: {} {}", name, e))
                }
            };
            actual_result = Some(result);
            break;
        }

        match actual_result {
            Some(a) => Ok(a),
            None => {
                let message = std::format!("`Failed to find function - {}", name);
                if config.all_errors_fatal {
                    Err(anyhow!("{}", message))
                } else {
                    warn!("{}", message);
                    Ok(Argument::Error(format!("`Failed to find function - {}", name)))
                }
            }
        }
    }

    fn attempt_call_module_function(
        &self,
        name: &str,
        arguments: &Vec<Argument>,
        definition: &Definition,
        prior_result: &Argument,
    ) -> RuntimeStatus<Argument> {
        if name.contains('.') {
            trace!("Attempting to find module call for {}", name);
            let mut parts = name.split('.');
            let module_name = parts.next();
            if module_name.is_none() {
                warn!(
                    "module_name starts with ., {}, defaulting to fall back method",
                    module_name.unwrap()
                );
                return RuntimeStatus::NotFound;
            }
            if module_name != Some(".") {
                let module = self.modules.get(module_name.unwrap());
                if module.is_none() {
                    warn!(
                        "Module not found, {}, defaulting to fall back method",
                        module_name.unwrap()
                    );
                    return RuntimeStatus::NotFound;
                }

                let module = module.unwrap();
                let mut new_name = String::new();
                for str in parts {
                    new_name.push_str(str);
                }
                return module.function_call(
                    new_name.as_str(),
                    arguments.clone(),
                    definition.clone(),
                    prior_result.clone(),
                );
            }
        }
        RuntimeStatus::NotFound
    }
}

pub fn initialize(options: Options) -> Result<RuntimeConfig> {
    let asts = parse_source_files(options.parse.clone().unwrap_or_default())?;
    let modules = setup_modules(options)?;
    Ok(RuntimeConfig { asts, modules })
}

fn create_cache_dir(options: &Options) -> String {
    let cache_directory = options
        .cache_directory
        .clone()
        .unwrap_or(".rigz/cache/modules".to_string());
    std::fs::create_dir_all(cache_directory.as_str())
        .unwrap_or_else(|_| panic!("Failed to create cache directory: {}", cache_directory));
    cache_directory
}

fn setup_modules(options: Options) -> Result<Vec<ModuleDefinition>> {
    let cache_directory = create_cache_dir(&options);
    let mut modules = Vec::new();

    let mut base_modules = options.modules.unwrap_or_default();
    if !options.disable_std_lib.unwrap_or(false) {
        base_modules.append(ModuleOptions::default_options().as_mut())
    }
    for module in base_modules {
        let name = module.name.as_str();
        let definition = module
            .download(PathBuf::from(cache_directory.clone()))
            .unwrap_or_else(|_| panic!("Failed to Download Module {}", name));
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
    use super::*;
    use log::LevelFilter;

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
        assert!(!result.asts.is_empty());
    }
}

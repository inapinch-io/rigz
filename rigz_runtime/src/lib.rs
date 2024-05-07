pub mod modules;
pub mod parse;
pub mod run;

use crate::modules::{initialize_module, invoke_symbol, ModuleOptions, ModuleRuntime, Platform};
use crate::parse::{parse_source_files, ParseOptions};
use crate::run::RunArgs;
use anyhow::{anyhow, Result};
use log::{error, info, trace};
use rigz_core::{Argument, ArgumentDefinition, ArgumentVector, Library};
use rigz_parse::AST;
use serde::Deserialize;
use std::collections::HashMap;
use std::ffi::{c_char, CStr};
use std::fmt::format;
use std::path::PathBuf;

#[derive(Clone, Default, Deserialize)]
pub struct Options {
    pub cache_directory: Option<String>,
    pub disable_std_lib: Option<bool>,
    pub modules: Option<Vec<ModuleOptions>>,
    pub parse: Option<ParseOptions>,
}

impl Options {
    fn module_config(&self) -> Vec<ModuleOptions> {
        match &self.modules {
            None => ModuleOptions::default_options(),
            Some(m) => {
                let mut base = if !self.disable_std_lib.unwrap_or(false) {
                    ModuleOptions::default_options()
                } else {
                    Vec::new()
                };
                for config in m {
                    base.push(ModuleOptions {
                        name: config.name.clone(),
                        source: config.source.clone(),
                        sub_folder: config.sub_folder.clone(),
                        dist: config.dist.clone(),
                        version: config.version.clone(),
                        metadata: config.metadata.clone(),
                        config: config.config.clone(),
                    })
                }
                base
            }
        }
    }
}

#[repr(C)]
pub struct Runtime {
    asts: HashMap<String, AST>,
    pub runtime: ModuleRuntime,
}

impl Runtime {
    pub unsafe fn invoke_symbol(
        &self,
        name: &String,
        arguments: Vec<Argument>,
        definition: Option<ArgumentDefinition>,
        prior_result: &Argument,
        config: &RunArgs,
    ) -> Result<Argument> {
        if config.require_aliases {
            // let library = self.library(name)?;
            // check global symbols
            // if name contains '.' split and try to find library
            todo!()
        }

        let args: ArgumentVector = arguments.into();
        let definition = definition.unwrap_or(ArgumentDefinition::Empty());

        let mut actual_result = None;
        for (lib_name, ref_lib) in self.runtime.libraries.clone() {
            trace!("Checking `{}` in Module: {}", name, lib_name);
            let result = invoke_symbol(
                ref_lib,
                name.into(),
                args.clone(),
                definition.clone(),
                prior_result,
            );
            if result.status == -1 {
                continue;
            }
            actual_result = Some(result);
            break;
        }

        let result = actual_result.expect(format!("Failed to find function: {}", name).as_str());
        let message = match result.status {
            0 => return Ok(result.value),
            -1 => {
                let m = format!("Symbol Not Found: `{}`", name);
                if config.ignore_symbol_not_found {
                    return Err(anyhow!(m));
                }
                m
            }
            _ => {
                format!(
                 "Invocation Failed: {} - {} ({}).",
                    name,
                    error_to_string(result.error_message),
                    result.status
                )
            }
        };

        if config.all_errors_fatal {
            Err(anyhow!(message))
        } else {
            Ok(Argument::Error(message.into()))
        }
    }
    fn library(&self, name: &String) -> Result<Library> {
        let library = self.runtime.libraries.get(name);
        match library {
            None => Err(anyhow!("Library Not Found: `{}`", name)),
            Some(l) => Ok(Library {
                name: l.name.clone(),
                path: l.path.clone(),
                handle: l.handle,
                format: l.format.clone(),
                pass_through: l.pass_through,
            }),
        }
    }
}

fn error_to_string<'a>(raw: *const c_char) -> &'a str {
    let c_str = unsafe {
        if raw.is_null() {
            return "null";
        }
        CStr::from_ptr(raw)
    };

    c_str.to_str().unwrap_or("null")
}

fn initialize_modules(options: Options) -> Result<ModuleRuntime> {
    let cache_directory = options
        .cache_directory
        .clone()
        .unwrap_or(".rigz/cache/modules".to_string());
    std::fs::create_dir_all(cache_directory.as_str())
        .expect(format!("Failed to create cache directory: {}", cache_directory).as_str());

    let mut module_runtime = ModuleRuntime::new();
    for m in options.module_config() {
        let name = m.name.clone();
        let library_path = m
            .download(PathBuf::from(cache_directory.clone()))
            .expect(format!("Failed to Download Module {}", name).as_str());
        unsafe {
            let result = initialize_module(name.clone().into(), library_path);
            let status = result.status;
            match status {
                0 => {
                    module_runtime.register_library(result.value);
                }
                _ => {
                    let error = unsafe { CStr::from_ptr(result.error_message) };
                    let error = match error.to_str() {
                        Ok(valid_str) => valid_str.to_string(),
                        Err(e) => {
                            error!("Failed to convert error_message to string {}", e);
                            "<Invalid UTF-8>".to_string()
                        }
                    };
                    return Err(anyhow!(
                        "Something went wrong: {} - {} ({}).",
                        name,
                        error,
                        status,
                    ));
                }
            }
        }
    }
    Ok(module_runtime)
}

pub fn initialize(options: Options) -> Result<Runtime> {
    let asts = parse_source_files(options.parse.clone().unwrap_or(ParseOptions::default()))?;
    let runtime = initialize_modules(options).expect("Failed to initialize modules");

    Ok(Runtime { asts, runtime })
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

    fn hello_world_options() -> Options {
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

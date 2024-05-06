pub mod modules;
pub mod parse;
pub mod run;

use crate::modules::{initialize_module, invoke_symbol, module_runtime, ModuleOptions, Platform};
use crate::parse::{parse_source_files, ParseOptions};
use crate::run::RunArgs;
use anyhow::{anyhow, Result};
use rigz_core::{Argument, ArgumentDefinition};
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
    pub platform: Option<Platform>,
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
        prior_result: &Argument,
        config: &RunArgs,
    ) -> Result<Argument> {
        let result = invoke_symbol(
            name.into(),
            arguments.into(),
            definition.unwrap_or(ArgumentDefinition::Empty()),
            prior_result,
        );
        let message = match result.status {
            0 => return Ok(result.value),
            -1 => {
                let m = format!("Symbol Not Found {}", name);
                if config.ignore_symbol_not_found {
                    return Err(anyhow!(m));
                }
                m
            }
            _ => {
                format!(
                    "Something went wrong: {} ({}){}",
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

fn initialize_modules(options: Options) -> Result<()> {
    let mut module_runtime = unsafe { module_runtime() };
    let module_config = match &options.modules {
        None => {
            let mut with_lib = Vec::new();
            with_lib.push(ModuleOptions {
                name: "std".to_string(),
                source: "https://gitlab.com/inapinch_rigz/rigz.git".to_string(),
                sub_folder: Some("rigz_lib".to_string()),
                version: None,
                dist: None, // TODO: Use dist once std lib is ironed out and stored in CDN
                metadata: None,
                config: None,
            });
            with_lib
        },
        Some(m) => {
            let mut base = if !options.disable_std_lib.unwrap_or(false) {
                let mut with_lib = Vec::new();
                with_lib.push(ModuleOptions {
                    name: "std".to_string(),
                    source: "https://gitlab.com/inapinch_rigz/rigz.git".to_string(),
                    sub_folder: Some("rigz_lib".to_string()),
                    version: None,
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
                    version: config.version.clone(),
                    metadata: config.metadata.clone(),
                    config: config.config.clone(),
                })
            }
            base
        }
    };
    let cache_directory = options
        .cache_directory
        .unwrap_or(".rigz/cache/modules".to_string());
    std::fs::create_dir_all(cache_directory.as_str())
        .expect(format!("Failed to create cache directory: {}", cache_directory).as_str());

    for m in module_config {
        let name = m.name.clone();
        let module = m
            .download(PathBuf::from(cache_directory.clone()))
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

pub (crate) fn path_to_string(path: &PathBuf) -> Result<String> {
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
            platform: None,
        }
    }

    #[test]
    fn default_initialize_works() {
        let result = initialize(hello_world_options()).expect("Failed to initialize");
        assert_eq!(result.asts.is_empty(), false);
    }
}

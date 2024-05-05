use crate::{Argument, ArgumentDefinition};
use anyhow::{anyhow, Error, Result};
use log::{info, warn};
use rigz_core::{ArgumentVector, StrSlice};
use rigz_parse::{parse, Definition, Element, ParseConfig, AST};
use serde::Deserialize;
use serde_value::Value;
use std::collections::HashMap;
use std::ffi::{c_char, c_int};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::process::Command;
use git2::Repository;

#[derive(Clone, Default, Deserialize)]
pub struct ModuleOptions {
    pub name: String,
    pub source: String,
    pub version: Option<String>,
    pub sub_folder: Option<String>,
    pub dist: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
    pub config: Option<Value>,
}


impl ModuleOptions {
    pub(crate) fn download(self, cache_path: PathBuf) -> Result<Module> {
        let dest = cache_path.join(self.clone_path());
        let _repo = self.download_source(&dest)?;
        let module_definition = self.load_config(&dest)?;
        let module_type = module_definition.build.clone().into();
        if self.dist.is_none() {
            let module_name = module_definition.name.to_string();
            let (build_command, outputs) = module_definition.prepare();
            let base_path = dest.clone();
            let config_path = match self.sub_folder {
                None => {
                    base_path
                }
                Some(p) => {
                    base_path.join(p)
                }
            };
            let mut parts = build_command.split_whitespace();

            let executable = parts.next().unwrap();
            let args: Vec<&str> = parts.collect();
            info!("Building {} ({}): `{}` ({})", self.name, module_name, build_command, config_path.to_str().expect("Failed to convert"));
            match Command::new(&executable).args(args).current_dir(&config_path).output() {
                Ok(o) => {
                    info!("Command finished {}", o.status)
                },
                Err(e) => {
                    return Err(anyhow!("Command Failed: `{}`, Path: {}. Error: {}", build_command, &config_path.to_str().unwrap_or("<config_path> Unknown"), e))
                }
            };
            // match output files to platform so zig can link them
        } else {
            let url = self.dist.clone().unwrap();
            info!("Downloading Dist {}: {}", self.name, url);
        };
        Ok(Module {
            name: self.name.into(),
            module_type,
        })
    }

    fn clone_path(&self) -> &str {
        self.source
            .split("/")
            .last()
            .expect("failed to split module.source on '/'")
            .split(".")
            .nth(0)
            .expect("failed to split module.source on '.'")
    }

    fn download_source(&self, dest: &PathBuf) -> Result<Repository> {
        let source = &self.source;
        info!("Cloning from {}", source);
        let repo = if dest.exists() {
            Repository::open(dest).expect(format!("Failed to open {}", source).as_str())
        } else {
            let mut dest = dest.clone();
            Repository::clone(source.as_str(), dest)
                .expect(format!("Failed to clone {}", source).as_str())
        };
        Ok(repo)
    }

    fn module_source_path(&self, dest: &PathBuf) -> PathBuf {
        let base_path = dest.clone();
        match &self.sub_folder {
            None => {
                base_path
            }
            Some(p) => {
                base_path.join(p)
            }
        }
    }

    fn load_config(&self, dest: &PathBuf) -> Result<ModuleDefinition> {
        let config_path = self.module_source_path(&dest).join("module.rigz");

        if !config_path.exists() {
            return Err(anyhow!("Module Config File Does Not Exit: {}", config_path.to_str().expect("<config_path: unknown>")))
        }
        let mut config = File::open(&config_path)?;
        let mut contents = String::new();
        let config_path = &config_path.to_str().unwrap();
        config
            .read_to_string(&mut contents)
            .expect(format!("Failed to read config: {}", config_path).as_str());
        let default_parse = ParseConfig::default();
        let config = parse(contents, &default_parse)
            .expect(format!("Failed to parse config: {}", config_path).as_str());
        config.try_into()
    }
}

#[derive(Default, Deserialize)]
pub struct ModuleDefinition {
    outputs: Option<HashMap<Platform, Vec<PathBuf>>>,
    name: String,
    build: String,
    config: Option<Value>,
}

impl ModuleDefinition {
    fn prepare(self) -> (String, HashMap<Platform, Vec<PathBuf>>) {
        match self.build.as_str().trim() {
            "cargo" => ("cargo build".into(), self.default_outputs()),
            "zig" => {
                (
                    format!("zig build && zig build-lib build.zig -dynamic --name {}", self.name),
                    self.default_outputs(),
                )
            },
            _ => (
                self.build.to_string(),
                self
                    .outputs
                    .expect("`module_definition.outputs` are required with custom `build`"),
            ),
        }
    }

    // files output to current directory
    fn default_outputs(&self) -> HashMap<Platform, Vec<PathBuf>> {
        let name = &self.name;
        let mut default = HashMap::new();
        default.insert(
            Platform::Unix,
            vec![
                PathBuf::from(format!("lib{}.a", name)),
                PathBuf::from(format!("lib{}.a.o", name)),
            ],
        );
        // TODO: Add other platforms
        default
    }
}

impl TryFrom<AST> for ModuleDefinition {
    type Error = Error;

    fn try_from(value: AST) -> Result<Self> {
        for element in value.elements {
            return if let Element::FunctionCall(fc) = element {
                if fc.identifier == "module" {
                    Ok(fc
                        .definition
                        .expect("definition is missing for module")
                        .try_into()?)
                } else {
                    Err(anyhow!(
                        "Invalid identifier in Function Call: {:?}",
                        fc.identifier
                    ))
                }
            } else {
                Err(anyhow!("Invalid Element in AST: {:?}", element))
            };
        }
        Err(anyhow!("AST is empty"))
    }
}

impl TryFrom<Definition> for ModuleDefinition {
    type Error = Error;

    fn try_from(value: Definition) -> Result<Self> {
        match value {
            Definition::Object(o) => {
                let mut o = o.0;
                Ok(ModuleDefinition {
                    outputs: convert_to_outputs(o.remove("outputs"))?,
                    name: o
                        .remove("name")
                        .expect("`module { name }` is missing")
                        .to_string(),
                    build: o
                        .remove("build")
                        .expect("`module { build } is missing")
                        .to_string(),
                    config: convert_to_value(o.remove("config"))?,
                })
            }
            Definition::List(l) => return Err(anyhow!("Lists are not currently supported here")),
        }
    }
}

fn convert_to_value(element: Option<Element>) -> Result<Option<Value>> {
    return Ok(None);
}

fn convert_to_outputs(element: Option<Element>) -> Result<Option<HashMap<Platform, Vec<PathBuf>>>> {
    return Ok(None);
}

#[derive(Default, Deserialize, Eq, Hash, PartialEq, Clone)]
pub enum Platform {
    #[default]
    Unix,
    Windows,
    Wasm,
    Jar,
}

#[derive(Deserialize)]
#[repr(C)]
pub enum ModuleType {
    Cargo,
    Custom,
    Zig,
}

impl From<String> for ModuleType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "cargo" => ModuleType::Cargo,
            "zig" => ModuleType::Zig,
            &_ => ModuleType::Custom,
        }
    }
}

#[repr(C)]
pub struct ModuleRuntime {}

#[repr(C)]
pub struct RuntimeStatus {
    pub status: c_int,
    pub value: Argument,
    pub error_message: *const c_char,
}

extern "C" {
    pub fn invoke_symbol(
        name: StrSlice,
        arguments: ArgumentVector,
        definition: ArgumentDefinition,
        prior_result: &Argument,
    ) -> RuntimeStatus;

    pub fn initialize_module(runtime: &mut ModuleRuntime, module: Module) -> RuntimeStatus;

    pub fn module_runtime() -> ModuleRuntime;
}

#[repr(C)]
pub struct Module {
    pub name: StrSlice,
    pub module_type: ModuleType,
}

impl Module {
    pub(crate) unsafe fn init(&self) -> Result<Module> {
        todo!()
    }
}

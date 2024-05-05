use crate::{Argument, ArgumentDefinition};
use anyhow::{anyhow, Error, Result};
use log::info;
use rigz_core::{ArgumentVector, StrSlice};
use rigz_parse::{parse, Definition, Element, ParseConfig, AST};
use serde::Deserialize;
use serde_value::Value;
use std::collections::HashMap;
use std::ffi::{c_char, c_int};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

#[derive(Clone, Default, Deserialize)]
pub struct ModuleOptions {
    pub name: String,
    pub source: String,
    pub sub_folder: Option<String>,
    pub dist: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
    pub config: Option<Value>,
}

impl ModuleOptions {
    pub(crate) fn download(self, cache_path: PathBuf) -> Result<Module> {
        let clone_path = self
            .source
            .split("/")
            .last()
            .expect("failed to split module.source on '/'")
            .split(".")
            .nth(0)
            .expect("failed to split module.source on '.'");
        let dest = cache_path.join("/").join(clone_path);
        let module_definition = download_source(&self.source, &dest)?;
        if self.dist.is_none() {
            let name = module_definition.name;
            let (build_command, outputs) = match module_definition.build.as_str() {
                "cargo" => ("cargo build".into(), default_outputs(name)),
                "zig" => (
                    format!("zig build-lib -dynamic -name {}", name),
                    default_outputs(name),
                ),
                _ => (
                    module_definition.build.to_string(),
                    module_definition
                        .outputs
                        .expect("`module_definition.outputs` are required with custom `build`"),
                ),
            };
            info!("Building {}: {}", self.name, build_command);
            // run build_command
            // match output files to platform so zig can link them
        } else {
            let url = self.dist.clone().unwrap();
            info!("Downloading Dist {}: {}", self.name, url);
        };
        todo!()
    }
}

// files output to current directory
fn default_outputs(name: String) -> HashMap<Platform, Vec<PathBuf>> {
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

fn download_source(source: &String, dest: &PathBuf) -> Result<ModuleDefinition> {
    info!("Cloning from {}", source);
    // TODO: check if already exists
    let repo = git2::Repository::clone(source.as_str(), dest)
        .expect(format!("Failed to clone {}", source).as_str());
    let config_path = dest.join("module.rigz");
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

#[derive(Default, Deserialize)]
pub struct ModuleDefinition {
    outputs: Option<HashMap<Platform, Vec<PathBuf>>>,
    name: String,
    build: String,
    config: Option<Value>,
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

fn convert_to_value(p0: Option<Element>) -> Result<Option<Value>> {
    todo!()
}

fn convert_to_outputs(p0: Option<Element>) -> Result<Option<HashMap<Platform, Vec<PathBuf>>>> {
    todo!()
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

use crate::{path_to_string, Argument, ArgumentDefinition};
use anyhow::{anyhow, Error, Result};
use git2::Repository;
use log::{debug, error, info, warn};
use rigz_core::{ArgumentVector, Library, RuntimeStatus, StrSlice};
use rigz_parse::{parse, Definition, Element, ParseConfig, AST};
use serde::Deserialize;
use serde_value::Value;
use std::collections::HashMap;
use std::ffi::{c_char, c_int, c_void};
use std::fs::File;
use std::io::Read;
use std::os::unix::process::ExitStatusExt;
use std::path::PathBuf;
use std::process::{Command, ExitStatus};

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
    pub(crate) fn default_options() -> Vec<ModuleOptions> {
        vec![ModuleOptions {
            name: "std".to_string(),
            source: "https://gitlab.com/inapinch_rigz/rigz.git".to_string(),
            sub_folder: Some("rigz_lib".to_string()),
            version: None,
            dist: None, // TODO: Use dist once std lib is ironed out and stored in CDN
            metadata: None,
            config: None,
        }]
    }
}

fn run_command(command: String, config_path: &PathBuf) -> Result<()> {
    let mut parts = command.split_whitespace();

    let executable = parts.next().unwrap();
    let args: Vec<&str> = parts.collect();
    match Command::new(executable)
        .args(args)
        .current_dir(config_path)
        .output()
    {
        Ok(o) => {
            if o.status != ExitStatus::from_raw(0) {
                let path = path_to_string(config_path)?;
                return Err(anyhow!(
                    "Command Failed: `{}`, Path: `{}` Output: {:?}",
                    command,
                    path,
                    o
                ));
            } else {
                info!("Command finished");
                debug!(
                    "Output: {}",
                    std::str::from_utf8(&o.stdout).unwrap_or("Failed to convert stdout")
                )
            }
        }
        Err(e) => {
            let path = path_to_string(config_path)?;
            return Err(anyhow!(
                "Command Failed: `{}`, Path: `{}`. Error: {}",
                command,
                path,
                e
            ));
        }
    }
    Ok(())
}

impl ModuleOptions {
    pub(crate) fn download(self, cache_path: PathBuf) -> Result<StrSlice> {
        let dest = cache_path.join(self.clone_path());
        let _repo = self.download_source(&dest)?;
        let module_definition = self.load_config(&dest)?;
        let library = if self.dist.is_none() {
            let module_name = module_definition.name.to_string();
            let (build_command, outputs) = module_definition.prepare();
            let config_path = self.module_source_path(&dest);
            let path = path_to_string(&config_path)?;
            info!("Building {} ({})", self.name, module_name);
            debug!(
                "{} ({}): `{}` ({})",
                self.name, module_name, build_command, path
            );
            run_command(build_command, &config_path)?;
            match outputs.get(&Platform::Unix) {
                None => return Err(anyhow!("No Output found for {}, Path: {}", self.name, path)),
                Some(o) => config_path.join(o),
            }
        } else {
            let url = self.dist.clone().unwrap();
            info!("Downloading Dist {}: {}", self.name, url);
            dest
        };
        Ok(path_to_string(&library)?.into())
    }

    fn clone_path(&self) -> &str {
        self.source
            .split('/')
            .last()
            .expect("failed to split module.source on '/'")
            .split('.')
            .nth(0)
            .expect("failed to split module.source on '.'")
    }

    fn download_source(&self, dest: &PathBuf) -> Result<Repository> {
        let source = &self.source;
        let repo = if dest.exists() {
            info!("{}: using {}", self.name, path_to_string(dest)?);
            Repository::open(dest).expect(format!("Failed to open {}", source).as_str())
        } else {
            info!("{}: cloning from {}", self.name, source);
            Repository::clone(source.as_str(), dest)
                .expect(format!("Failed to clone {}", source).as_str())
        };
        Ok(repo)
    }

    fn module_source_path(&self, dest: &PathBuf) -> PathBuf {
        let base_path = dest.clone();
        match &self.sub_folder {
            None => base_path,
            Some(p) => base_path.join(p),
        }
    }

    fn load_config(&self, dest: &PathBuf) -> Result<ModuleDefinition> {
        let config_path = self.module_source_path(dest).join("module.rigz");

        if !config_path.exists() {
            return Err(anyhow!(
                "Module Config File Does Not Exit: {}",
                path_to_string(&config_path)?
            ));
        }
        let mut config = File::open(&config_path)?;
        let mut contents = String::new();
        let config_path = path_to_string(&config_path)?;
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
    outputs: Option<HashMap<Platform, PathBuf>>,
    name: String,
    build: String,
    config: Option<Value>,
    format: Option<FunctionFormat>,
}

impl ModuleDefinition {
    fn prepare(self) -> (String, HashMap<Platform, PathBuf>) {
        match self.build.as_str().trim() {
            "cargo" => ("cargo build".into(), self.default_outputs()),
            "zig" => (
                format!("zig build-lib build.zig --name {} -dynamic", self.name),
                self.default_outputs(),
            ),
            _ => (
                self.build.to_string(),
                self.outputs
                    .expect("`module_definition.outputs` are required with custom `build`"),
            ),
        }
    }

    // files output to current directory
    fn default_outputs(&self) -> HashMap<Platform, PathBuf> {
        let name = &self.name;
        let mut default = HashMap::new();
        default.insert(Platform::Unix, PathBuf::from(format!("lib{}.so", name)));
        default.insert(Platform::Mac, PathBuf::from(format!("lib{}.dylib", name)));
        // TODO: Add other platforms
        default
    }
}

impl TryFrom<AST> for ModuleDefinition {
    type Error = Error;

    fn try_from(value: AST) -> Result<Self> {
        if let Some(element) = value.elements.into_iter().next() {
            if let Element::FunctionCall(fc) = element {
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
            }
        } else {
            Err(anyhow!("AST is empty"))
        }
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
                    format: o.remove("format").map(|f| {
                        f.to_string()
                            .try_into()
                            .expect("Failed to convert module.format")
                    }),
                })
            }
            Definition::List(l) => return Err(anyhow!("Lists are not currently supported here")),
        }
    }
}

fn convert_to_value(element: Option<Element>) -> Result<Option<Value>> {
    return Ok(None);
}

fn convert_to_outputs(element: Option<Element>) -> Result<Option<HashMap<Platform, PathBuf>>> {
    return Ok(None);
}

#[derive(Default, Deserialize, Eq, Hash, PartialEq, Clone)]
pub enum Platform {
    #[default]
    Unix,
    Mac,
    Windows,
    Wasm,
    Jar,
}

pub struct ModuleRuntime {
    pub(crate) libraries: HashMap<String, Library>,
}

impl ModuleRuntime {
    pub(crate) fn new() -> ModuleRuntime {
        ModuleRuntime {
            libraries: HashMap::new(),
        }
    }
    pub(crate) fn register_library(&mut self, library: Library) -> () {
        match self.libraries.insert(library.name.to_string(), library) {
            None => {}
            Some(previous) => {
                warn!("Overwrote {}", previous.name)
            }
        }
    }
}

#[repr(C)]
pub struct ModuleStatus {
    pub status: c_int,
    pub value: Library,
    pub error_message: *const c_char,
}

extern "C" {
    pub fn invoke_symbol(
        library: Library,
        name: StrSlice,
        arguments: ArgumentVector,
        definition: ArgumentDefinition,
        prior_result: &Argument,
    ) -> RuntimeStatus;

    pub fn initialize_module(name: StrSlice, library_path: StrSlice) -> ModuleStatus;
}

#[derive(Clone, Deserialize, Default)]
pub enum FunctionFormat {
    #[default]
    FIXED, // fn(ArgumentVector, ArgumentDefinition, Argument) RuntimeStatus
    PASS, // fn(StrSlice, ArgumentVector, ArgumentDefinition, Argument) RuntimeStatus
          // DYNAMIC TODO: call raw signatures directly, ie: fn add(i32, i32) -> i32
}

impl From<rigz_core::FunctionFormat> for FunctionFormat {
    fn from(value: rigz_core::FunctionFormat) -> Self {
        match value {
            rigz_core::FunctionFormat::FIXED => FunctionFormat::FIXED,
            rigz_core::FunctionFormat::PASS => FunctionFormat::PASS,
            // rigz_core::FunctionFormat::DYNAMIC => FunctionFormat::DYNAMIC,
        }
    }
}

impl TryFrom<String> for FunctionFormat {
    type Error = Error;

    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        let format = match value.as_str() {
            "FIXED" => FunctionFormat::FIXED,
            "PASS" => FunctionFormat::PASS,
            &_ => return Err(anyhow!("Unknown Function Format: {}", value)),
        };
        Ok(format)
    }
}

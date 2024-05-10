use crate::{path_to_string, Module};
use anyhow::{anyhow, Error, Result};
use log::{info, warn};
use rigz_parse::{parse, Definition, Element, ParseConfig, AST};
use serde::Deserialize;
use serde_value::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::process::Command;
use std::rc::Rc;
use rigz_lua::LuaModule;
use crate::run::RunArgs;

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
            sub_folder: Some("modules/std_lib".to_string()),
            version: None,
            dist: None, // TODO: Use dist once std lib is ironed out and stored in CDN
            metadata: None,
            config: None,
        }]
    }
}

struct Repository {
    source: String,
    dest: PathBuf
}

impl Repository {
    fn download(&self) -> Result<()> {
        let dest= path_to_string(&self.dest)?;
        match Command::new("git")
            .arg("clone")
            .arg(&self.source)
            .arg(dest.as_str())
            .status() {
            Ok(e) => {
                if e.success() {
                    Ok(())
                } else {
                    Err(anyhow!("git clone {} {} failed. status - {}", self.source, dest, e))
                }
            }
            Err(e) => {
                Err(anyhow!("Command Failed - git clone {} {} - {}", self.source, dest, e))
            }
        }
    }

    fn open(&self) -> Result<()> {
        Command::new("git")
            .current_dir(&self.dest)
            .arg("fetch")
            .status()
            .map_err(|err| anyhow::anyhow!("Failed to run git command: {}", err))?;

        let output = Command::new("git")
            .current_dir(&self.dest)
            .args(&["diff", "--quiet", "origin"])
            .output();

        match output {
            Ok(output) => {
                if !output.status.success() {
                    warn!("There are remote changes for {}", self.source);
                    Ok(())
                } else {
                    Ok(())
                }
            }
            Err(err) => {
                Err(anyhow!("Command Failed, git diff, {} - {}", self.source, err))
            }
        }
    }
}

impl ModuleOptions {
    pub(crate) fn download(&self, cache_path: PathBuf) -> Result<ModuleDefinition> {
        let dest = cache_path.join(self.clone_path());
        let _repo = self.download_source(&dest)?;
        self.load_config(&dest)
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
        let source = self.source.as_str();
        let repo = Repository { source: source.to_string(), dest: dest.clone() };
        if dest.exists() {
            info!("{}: using {}", self.name, path_to_string(dest)?);
            repo.open()
        } else {
            info!("{}: cloning from {}", self.name, source);
            repo.download()
        }?;
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
    name: String,
    build: Option<String>,
    config: Option<Value>,
    format: Option<FunctionFormat>,
}

#[derive(Debug, Default, Deserialize)]
pub(crate) enum FunctionFormat {
    #[default]
    Lua,
    LuaModule,
    // dynlib
    // JNI
    // wasm
}

impl ModuleDefinition {
    pub fn initialize(self, _run_args: Rc<RunArgs>) -> Result<Box<dyn Module>> {
        let format = self.format.unwrap_or(FunctionFormat::Lua);
        let name = self.name.clone();
        let module: Box<dyn Module> = match format {
            FunctionFormat::Lua => {
                LuaModule::new(name, vec![], Default::default())
            }
            _ => {
                return Err(anyhow!("{:?} is not supported for {}", format, name))
            }
        };
        Ok(module)
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
                    name: o
                        .remove("name")
                        .expect("`module { name }` is missing")
                        .to_string(),
                    build: o
                        .remove("build")
                        .map(|b| b.to_string()),
                    config: convert_to_value(o.remove("config"))?,
                    format: o.remove("format").map(|f| {
                        f.to_string()
                            .try_into()
                            .expect("Failed to convert module.format")
                    }),
                })
            }
            Definition::List(_l) => return Err(anyhow!("Lists are not currently supported here")),
        }
    }
}

impl From<String> for FunctionFormat {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Lua" => FunctionFormat::Lua,
            "LuaModule" => FunctionFormat::LuaModule,
            _ => {
                warn!("Unsupported Format: {}, defaulting to Lua", value);
                FunctionFormat::Lua
            }
        }
    }
}

fn convert_to_value(_element: Option<Element>) -> Result<Option<Value>> {
    return Ok(None);
}

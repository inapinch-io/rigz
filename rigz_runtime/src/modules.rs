use std::collections::HashMap;
use std::path::PathBuf;
use libloading::{Library, Symbol};
use serde::Deserialize;
use serde_value::Value;
use anyhow::{anyhow, Result};
use log::{error, info, warn};

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
    pub(crate) fn download(self) -> Result<Module<'static>> {
        info!("Cloning from {}", self.source);
        // git clone
        // read module definition
        let module_definition = ModuleDefinition {
            outputs: Default::default(),
            build_command: None,
            config: None,
        };
        if self.dist.is_none() {
            let build_command = if module_definition.build_command.is_none() {
                return Err(anyhow!("Unable to build {} without `build_command`", self.name))
            } else {
                module_definition.build_command.unwrap()
            };
            info!("Building {}: {}", self.name, build_command);
        } else {
            let url = self.dist.clone().unwrap();
            info!("Downloading Dist {}: {}", self.name, url);
        };
        todo!()
    }
}

#[derive(Default, Deserialize)]
pub struct ModuleDefinition {
    outputs: HashMap<String, PathBuf>,
    build_command: Option<String>,
    config: Option<Value>,
}

pub struct Module<'a> {
    name: &'a str,
    lib: Library
}

impl Module<'_> {
    pub(crate) unsafe fn init(&self, symbols: &mut HashMap<String, crate::Symbol>) -> Result<Module> {
        let init_func: Option<Symbol<extern fn() -> u32>> = match self.lib.get(b"initialize") {
            Ok(s) => {
                Some(s)
            }
            Err(e) => {
                warn!("Failed to initialize module: {}", self.name);
                None
            }
        };
        todo!()
    }
}
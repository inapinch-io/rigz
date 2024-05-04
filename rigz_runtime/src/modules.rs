use crate::{Argument, ArgumentDefinition};
use anyhow::{anyhow, Result};
use log::info;
use rigz_core::{StrSlice, Vector};
use serde::Deserialize;
use serde_value::Value;
use std::collections::HashMap;
use std::ffi::{c_char, c_int};
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
    pub(crate) fn download(self) -> Result<Module> {
        let module_definition = download_source(self.source);
        if self.dist.is_none() {
            let build_command = if module_definition.build_command.is_none() {
                return Err(anyhow!(
                    "Unable to build {} without `build_command`",
                    self.name
                ));
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

fn download_source(source: String) -> ModuleDefinition {
    info!("Cloning from {}", source);
    todo!()
}

#[derive(Default, Deserialize)]
pub struct ModuleDefinition {
    outputs: Vec<PathBuf>,
    build_command: Option<String>,
    config: Option<Value>,
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
        arguments: Vector,
        definition: ArgumentDefinition,
    ) -> RuntimeStatus;

    pub fn initialize_module(runtime: &mut ModuleRuntime, module: Module) -> RuntimeStatus;

    pub fn module_runtime() -> ModuleRuntime;
}

#[repr(C)]
pub struct Module {
    pub name: StrSlice,
}

impl Module {
    pub(crate) unsafe fn init(&self) -> Result<Module> {
        todo!()
    }
}

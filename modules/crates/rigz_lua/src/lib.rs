mod args;

use std::cmp::max;
use crate::args::{to_args, Arg, Definition};
use anyhow::anyhow;
use log::{debug, info, warn};
use mlua::{Function, Lua, Value, Variadic};
use rigz_core::{Argument, InitializationArgs, Module, RuntimeStatus};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

#[derive(Copy, Clone, Debug, Default, Deserialize)]
pub enum FunctionFormat {
    #[default]
    StructFunction,        // { name, args, context, prior }
                           // Dynamic https://gitlab.com/inapinch/rigz/rigz/-/issues/2
}

impl From<String> for FunctionFormat {
    fn from(value: String) -> Self {
        match value.as_str() {
            "StructFunction" => FunctionFormat::StructFunction,
            _ => {
                warn!("Unsupported Format: {}, defaulting to Args", value);
                FunctionFormat::StructFunction
            }
        }
    }
}

pub struct LuaModule {
    pub(crate) name: String,
    pub(crate) function_format: FunctionFormat,
    pub(crate) module_root: PathBuf,
    pub(crate) lua: Lua,
    pub(crate) source_files: Vec<PathBuf>,
    pub(crate) input_files: HashMap<String, Vec<File>>,
}

fn inspect(lua: &Lua, value: Value) -> mlua::Result<String> {
    let result = match value {
        Value::Table(t) => {
            let mut result = String::new();
            let len = t.clone().pairs::<Value, Value>().count();
            let pairs = t.pairs::<Value, Value>();
            let mut index = 1;
            let mut is_array = false;
            for pair in pairs {
                let (key, value) = pair?;
                result.push(' ');
                let key_str = inspect(lua, key)?;

                if index == 1 {
                    if key_str.as_str() == "1" {
                        is_array = true;
                        result.push('[');
                    } else {
                        result.push('{');
                    }
                }
                if !is_array {
                    result.push_str(key_str.as_str());
                    result.push_str(" = ");
                }
                result.push_str(inspect(lua, value)?.as_str());
                if index < len {
                    result.push(',');
                }
                index += 1;
            }

            if is_array {
                result.push(']');
            } else {
                result.push('}');
            }
            result
        }
        _ => value.to_string()?
    };
    Ok(result)
}

impl LuaModule {
    pub fn new(
        name: String,
        module_root: PathBuf,
        source_files: Vec<PathBuf>,
        input_files: HashMap<String, Vec<File>>,
        config: Option<serde_value::Value>,
    ) -> Box<dyn Module> {
        let function_format = match config {
            None => FunctionFormat::default(),
            Some(f) => {
                if let serde_value::Value::Map(mut map) = f {
                    let f = map.remove(&serde_value::Value::String("function_format".into()));
                    if f.is_none() {
                        FunctionFormat::default()
                    } else {
                        f.unwrap()
                            .deserialize_into()
                            .unwrap_or(FunctionFormat::default())
                    }
                } else {
                    FunctionFormat::default()
                }
            }
        };
        Box::new(LuaModule {
            name,
            function_format,
            module_root,
            input_files,
            lua: Lua::new(),
            source_files,
        })
    }

    pub(crate) fn invoke_function(
        &self,
        name: &str,
        args: Vec<Arg>,
        context: Definition,
        previous_value: Arg,
    ) -> RuntimeStatus<Arg> {
        let lua = &self.lua;
        let table = lua.globals();
        
        lua
            .scope(|_| {
                let function: Function = match table.get::<_, Function>(name) {
                    Ok(f) => f,
                    Err(e) => {
                        warn!("Function Not Found: {} - {}", name, e);
                        return Ok(RuntimeStatus::NotFound);
                    }
                };

                let status = match self.function_format {
                    FunctionFormat::StructFunction => {
                        let table = lua.create_table()?;
                        table.set("name", name)?;
                        table.set("args", args)?;
                        table.set("previous_value", previous_value)?;
                        table.set("context", context)?;
                        RuntimeStatus::Ok(function.call(table)?)
                    }
                };
                Ok(status)
            })
            .unwrap_or(RuntimeStatus::Err("Lua Execution Failed".to_string()))
    }

    fn load_source_files(&self) -> anyhow::Result<()> {
        if self.source_files.is_empty() {
            warn!("No source files configured for module {}", self.name);
        }

        for file in &self.source_files {
            let ext = file
                .extension()
                .map(|o| o.to_str().unwrap_or("<invalid>"))
                .unwrap_or("<none>");
            if ext != "lua" {
                continue;
            }
            let current_file = file.to_str().unwrap_or("<unknown>");
            info!("{} loading {}", self.name, current_file);
            let contents = load_file(file)?;
            match self.lua.scope(|_| {
                let global = self.lua.globals();
                let chunk = self.lua.load(contents);
                chunk.exec()?;
                global.set("__module_name", self.name.as_str())?;
                Ok(())
            }) {
                Ok(_) => continue,
                Err(e) => {
                    return Err(anyhow!(
                        "Failed to load file: {} - {} {}",
                        self.name,
                        current_file,
                        e
                    ))
                }
            }
        }

        Ok(())
    }
}

fn load_file(path_buf: &PathBuf) -> anyhow::Result<String> {
    let mut contents = String::new();
    let mut file = File::open(path_buf)?;
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

impl Module for LuaModule {
    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn root(&self) -> PathBuf {
        self.module_root.clone()
    }

    fn function_call(
        &self,
        name: &str,
        arguments: Vec<Argument>,
        definition: rigz_core::Definition,
        prior_result: Argument,
    ) -> RuntimeStatus<Argument> {
        match self.invoke_function(
            name,
            to_args(arguments),
            definition.into(),
            prior_result.into(),
        ) {
            RuntimeStatus::Ok(a) => RuntimeStatus::Ok(a.into()),
            RuntimeStatus::NotFound => RuntimeStatus::NotFound,
            RuntimeStatus::Err(e) => RuntimeStatus::Err(e),
        }
    }

    fn initialize(&self, args: InitializationArgs) -> RuntimeStatus<()> {
        match self.load_source_files() {
            Ok(_) => {}
            Err(e) => return RuntimeStatus::Err(format!("Failed to load source files - {}", e)),
        };


        if self.input_files.is_empty() {
            debug!("No input files passed into module {}", self.name)
        }

        match self.lua.scope(|_| {
            let global = self.lua.globals();
            global.set("inspect", self.lua.create_function(inspect)?)?;
            global.set("__module_name", self.name.as_str())?;
            Ok(())
        }) {
            Ok(_) => RuntimeStatus::Ok(()),
            Err(e) => RuntimeStatus::Err(format!("Initialization Failed: {} - {}", self.name, e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let module = LuaModule::new(
            "hello_world".to_string(),
            Default::default(),
            vec![],
            Default::default(),
            None,
        );

        let result = module.function_call(
            "print",
            vec![Argument::String("Hello World".into())],
            rigz_core::Definition::None,
            Argument::None,
        );
        assert_eq!(result, RuntimeStatus::Ok(Argument::None));
    }
}

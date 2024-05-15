mod args;

use crate::args::{to_args, Arg, Definition};
use anyhow::anyhow;
use log::{debug, info, warn};
use mlua::{Function, Lua, Variadic};
use rigz_core::{Argument, Module, RuntimeStatus};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

#[derive(Copy, Clone, Debug, Default, Deserialize)]
pub enum FunctionFormat {
    #[default]
    Args, //  variadic args (...) - (*args, context), no prior result
    ArgsFunction,          // (...), (name, *args, context)
    ArgsWithPrior,         // (...), (*args, context, prior)
    ArgsWithPriorFunction, // (...), (name, *args, context, prior)
    Struct,                // { args, context, prior }
    StructFunction,        // { name, args, context, prior }
                           // Dynamic https://gitlab.com/inapinch/rigz/rigz/-/issues/2
}

impl From<String> for FunctionFormat {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Args" => FunctionFormat::Args,
            "ArgsFunction" => FunctionFormat::ArgsFunction,
            "ArgsWithPrior" => FunctionFormat::ArgsWithPrior,
            "ArgsWithPriorFunction" => FunctionFormat::ArgsWithPriorFunction,
            "Struct" => FunctionFormat::Struct,
            "StructFunction" => FunctionFormat::StructFunction,
            _ => {
                warn!("Unsupported Format: {}, defaulting to Args", value);
                FunctionFormat::Args
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

                let mut is_args = false;
                let mut needs_prior = false;
                let mut is_function_args = false;
                let mut is_struct_args = false;
                match self.function_format {
                    FunctionFormat::Args => {
                        is_args = true;
                    }
                    FunctionFormat::ArgsFunction => {
                        is_args = true;
                        is_function_args = true;
                    }
                    FunctionFormat::ArgsWithPrior => {
                        is_args = true;
                        needs_prior = true;
                    }
                    FunctionFormat::ArgsWithPriorFunction => {
                        is_args = true;
                        is_function_args = true;
                        needs_prior = true;
                    }
                    FunctionFormat::Struct => {
                        is_struct_args = true;
                    }
                    FunctionFormat::StructFunction => {
                        is_struct_args = true;
                        is_function_args = true;
                    }
                }

                if is_args {
                    let mut lua_args: Vec<Arg> = Vec::with_capacity(args.len());
                    if is_function_args {
                        lua_args.push(Arg::String(name.to_string()))
                    }
                    for arg in args {
                        lua_args.push(arg);
                    }

                    if let Definition::None = context {
                        // TODO make configurable
                        info!("Excluding empty context")
                    } else {
                        lua_args.push(Arg::Definition(context));
                    }

                    if needs_prior {
                        lua_args.push(previous_value);
                    }

                    let result = function.call(Variadic::from_iter(lua_args))?;
                    return Ok(RuntimeStatus::Ok(result))
                }
                
                if is_struct_args {
                    let table = lua.create_table()?;
                    if is_function_args {
                        table.set("name", name)?;
                    }
                    
                    table.set("args", args)?;
                    table.set("previous_value", previous_value)?;
                    table.set("context", context)?;
                    let result = function.call(table)?;
                    return Ok(RuntimeStatus::Ok(result))
                }

                Ok(RuntimeStatus::Err("Unimplemented Argument Options".into()))
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

    fn initialize(&self) -> RuntimeStatus<()> {
        match self.load_source_files() {
            Ok(_) => {}
            Err(e) => return RuntimeStatus::Err(format!("Failed to load source files - {}", e)),
        };
        if self.input_files.is_empty() {
            debug!("No input files passed into module {}", self.name)
        }

        match self.lua.scope(|_| {
            let global = self.lua.globals();
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

use mlua::{Error, FromLua, Function, IntoLua, Lua, Value, Variadic};
use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;
use log::{debug, info, warn};
use rigz_core::{Argument, Module, RigzFile, RuntimeStatus};

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Arg {
    None,
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Bool(bool),
    String(String),
    Object(HashMap<String, Arg>),
    List(Vec<Arg>),
    FunctionCall(FunctionCall),
    Definition(Definition),
    Error(String),
    File(RigzFile),
}

impl FromLua<'_> for Arg {
    fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
        let arg = match value {
            Value::Nil => Arg::None,
            Value::Boolean(b) => Arg::Bool(b),
            Value::Error(e) => Arg::Error(e.to_string()),
            Value::Integer(i) => Arg::Long(i),
            Value::Number(n) => Arg::Double(n),
            Value::String(s) => Arg::String(s.to_str()?.to_string()),
            Value::Table(t) => {
                // TODO: check vec vs map
                let mut results = HashMap::new();
                for each in t.pairs() {
                    let (k, v): (String, Value) = each?;
                    results.insert(k, Self::from_lua(v, &lua)?);
                }
                Arg::Object(results)
            }
            // TODO - Value::LightUserData(_) => {}
            // TODO - Value::UserData(_) => {}
            _ => return Err(Error::RuntimeError("Unsupported".into())),
        };
        Ok(arg)
    }
}

impl IntoLua<'_> for Arg {
    fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
        let value = lua.scope(|_| {
            Ok({
                match self {
                    Arg::None => Value::Nil,
                    Arg::Int(i) => Value::Integer(i.into()),
                    Arg::Long(l) => Value::Integer(l),
                    Arg::Float(f) => Value::Number(f.into()),
                    Arg::Double(d) => Value::Number(d),
                    Arg::Bool(b) => Value::Boolean(b),
                    Arg::String(s) => s.into_lua(lua)?,
                    Arg::Object(o) => o.into_lua(lua)?,
                    Arg::List(l) => l.into_lua(lua)?,
                    Arg::FunctionCall(fc) => fc.into_lua(lua)?,
                    Arg::Definition(c) => c.into_lua(lua)?,
                    Arg::Error(e) => Value::Error(Error::RuntimeError(e)),
                    _ => {
                        let arg: Argument = self.into();
                        Value::Error(Error::RuntimeError(format!("{} is not implemented yet", arg)))
                    }
                }
            })
        })?;
        Ok(value)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Definition {
    None,
    Some(HashMap<String, Arg>),
    Many(Vec<Arg>),
}

impl From<rigz_core::Definition> for Definition {
    fn from(value: rigz_core::Definition) -> Self {
        match value {
            rigz_core::Definition::None => Definition::None,
            rigz_core::Definition::One(o) => Definition::Some(to_context(o)),
            rigz_core::Definition::Many(l) => Definition::Many(to_args(l))
        }
    }
}

impl Into<rigz_core::Definition> for Definition {
    fn into(self) -> rigz_core::Definition {
        match self {
            Definition::None => rigz_core::Definition::None,
            Definition::Some(o) => rigz_core::Definition::One(to_object(o)),
            Definition::Many(l) => rigz_core::Definition::Many(to_arguments(l))
        }
    }
}

impl IntoLua<'_> for Definition {
    fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
        let value = lua.scope(|_| {
            Ok(match self {
                Definition::None => Value::Nil,
                Definition::Some(o) => o.into_lua(lua)?,
                Definition::Many(m) => m.into_lua(lua)?,
            })
        })?;
        Ok(value)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FunctionCall {
    pub name: String,
    pub args: Vec<Arg>,
    pub context: Definition,
}

impl From<rigz_core::FunctionCall> for FunctionCall {
    fn from(value: rigz_core::FunctionCall) -> Self {
        FunctionCall {
            name: value.name,
            args: to_args(value.args),
            context: value.definition.into(),
        }
    }
}

impl Into<rigz_core::FunctionCall> for FunctionCall {
    fn into(self) -> rigz_core::FunctionCall {
        rigz_core::FunctionCall {
            name: self.name,
            args: to_arguments(self.args),
            definition: self.context.into(),
        }
    }
}

impl IntoLua<'_> for FunctionCall {
    fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
        let result = lua.scope(|_| {
            let lua_args = lua.create_table()?;
            Ok(Value::Table(lua_args))
        })?;
        Ok(result)
    }
}

pub(crate) fn invoke_function(
    lua: &Lua,
    name: &str,
    args: Vec<Arg>,
    context: Definition,
    previous_value: Arg,
) -> RuntimeStatus<Arg> {
    let table = lua.globals();
    let value = lua.scope(|_| {
        let function: Function = match table.get::<&str, Function>(name) {
            Ok(f) => f,
            Err(e) => {
                warn!("Function Not Found: {} - {}", name, e);
                return Ok(RuntimeStatus::NotFound)
            }
        };

        let mut lua_args: Vec<Arg> = Vec::with_capacity(args.len());
        for arg in args {
            lua_args.push(arg);
        }

        if let Definition::None = context {
            // TODO make configurable
            info!("Excluding empty context")
        } else {
            lua_args.push(Arg::Definition(context));
        }

        if let Arg::None = previous_value {
            // TODO make configurable
            info!("Excluding previous value")
        } else {
            lua_args.push(previous_value);
        }

        let result = function.call(Variadic::from_iter(lua_args))?;
        Ok(RuntimeStatus::Ok(result))
    }).unwrap_or(RuntimeStatus::Err("Lua Execution Failed".to_string()));
    value
}

pub struct LuaModule {
    pub(crate) name: String,
    pub(crate) lua: Lua,
    pub(crate) source_files: Vec<PathBuf>,
    pub(crate) input_files: HashMap<String, Vec<File>>
}

impl LuaModule {
    pub fn new(
        name: String,
        source_files: Vec<PathBuf>,
        input_files: HashMap<String, Vec<File>>
    ) -> Box<dyn Module> {
        Box::new(LuaModule {
            name,
            input_files,
            lua: Lua::new(),
            source_files,
        })
    }
}

impl Module for LuaModule {
    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn function_call(&self, name: &str, arguments: Vec<Argument>, definition: rigz_core::Definition, prior_result: Argument) -> RuntimeStatus<Argument> {
        match invoke_function(&self.lua, name, to_args(arguments), definition.into(), prior_result.into()) {
            RuntimeStatus::Ok(a) => RuntimeStatus::Ok(a.into()),
            RuntimeStatus::NotFound => RuntimeStatus::NotFound,
            RuntimeStatus::Err(e) => RuntimeStatus::Err(e),
        }
    }

    fn initialize(&self) -> RuntimeStatus<()> {
        if self.source_files.is_empty() {
            warn!("No source files configured for module {}", self.name)
        }

        if self.input_files.is_empty() {
            debug!("No input files passed into module {}", self.name)
        }

        match self.lua.scope(|_| {
            let global = self.lua.globals();
            global.set("__module_name", self.name.as_str())?;
            Ok(())
        }) {
            Ok(_) => {
                RuntimeStatus::Ok(())
            }
            Err(e) => {
                RuntimeStatus::Err(format!("Initialization Failed: {} - {}", self.name, e))
            }
        }
    }
}

impl From<Argument> for Arg {
    fn from(value: Argument) -> Self {
        match value {
            Argument::None => Arg::None,
            Argument::Int(i) => Arg::Int(i),
            Argument::Long(l) => Arg::Long(l),
            Argument::Float(f) => Arg::Float(f),
            Argument::Double(d) => Arg::Double(d),
            Argument::Bool(b) => Arg::Bool(b),
            Argument::String(s) => Arg::String(s),
            Argument::Object(o) => Arg::Object(to_context(o)),
            Argument::List(l) => Arg::List(to_args(l)),
            Argument::FunctionCall(f) => Arg::FunctionCall(f.into()),
            Argument::Definition(d) => Arg::Definition(d.into()),
            Argument::Error(e) => Arg::Error(e),
            Argument::File(f) => Arg::File(f),
        }
    }
}

impl Into<Argument> for Arg {
    fn into(self) -> Argument {
        match self {
            Arg::None => Argument::None,
            Arg::Int(i) => Argument::Int(i),
            Arg::Long(l) => Argument::Long(l),
            Arg::Float(f) => Argument::Float(f),
            Arg::Double(d) => Argument::Double(d),
            Arg::Bool(b) => Argument::Bool(b),
            Arg::String(s) => Argument::String(s),
            Arg::Object(o) => Argument::Object(to_object(o)),
            Arg::List(l) => Argument::List(to_arguments(l)),
            Arg::FunctionCall(f) => Argument::FunctionCall(f.into()),
            Arg::Definition(d) => Argument::Definition(d.into()),
            Arg::Error(e) => Argument::Error(e),
            Arg::File(f) => Argument::File(f)
        }
    }
}

fn to_arguments(args: Vec<Arg>) -> Vec<Argument> {
    let mut v = Vec::with_capacity(args.len());
    for arg in args {
        v.push(arg.into());
    }
    v
}

fn to_args(args: Vec<Argument>) -> Vec<Arg> {
    let mut v = Vec::with_capacity(args.len());
    for arg in args {
        v.push(arg.into());
    }
    v
}

fn to_object(args: HashMap<String, Arg>) -> HashMap<String, Argument> {
    let mut v = HashMap::with_capacity(args.len());
    for (k, arg) in args {
        v.insert(k, arg.into());
    }
    v
}

fn to_context(args: HashMap<String, Argument>) -> HashMap<String, Arg> {
    let mut v = HashMap::with_capacity(args.len());
    for (k, arg) in args {
        v.insert(k, arg.into());
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let lua = Lua::new();
        let result = invoke_function(
            &lua,
            "print",
            vec![Arg::String("Hello World".into())],
            Definition::None,
            Arg::None,
        );
        assert_eq!(result, RuntimeStatus::Ok(Arg::None));
    }
}

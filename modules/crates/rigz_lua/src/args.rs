use mlua::{Error, FromLua, IntoLua, Lua, Value};
use rigz_core::{Argument, RigzFile};
use std::collections::HashMap;

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
                        Value::Error(Error::RuntimeError(format!(
                            "{} is not implemented yet",
                            arg
                        )))
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
            rigz_core::Definition::Many(l) => Definition::Many(to_args(l)),
        }
    }
}

impl Into<rigz_core::Definition> for Definition {
    fn into(self) -> rigz_core::Definition {
        match self {
            Definition::None => rigz_core::Definition::None,
            Definition::Some(o) => rigz_core::Definition::One(to_object(o)),
            Definition::Many(l) => rigz_core::Definition::Many(to_arguments(l)),
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
            Arg::File(f) => Argument::File(f),
        }
    }
}

pub(crate) fn to_arguments(args: Vec<Arg>) -> Vec<Argument> {
    let mut v = Vec::with_capacity(args.len());
    for arg in args {
        v.push(arg.into());
    }
    v
}

pub(crate) fn to_args(args: Vec<Argument>) -> Vec<Arg> {
    let mut v = Vec::with_capacity(args.len());
    for arg in args {
        v.push(arg.into());
    }
    v
}

pub(crate) fn to_object(args: HashMap<String, Arg>) -> HashMap<String, Argument> {
    let mut v = HashMap::with_capacity(args.len());
    for (k, arg) in args {
        v.insert(k, arg.into());
    }
    v
}

pub(crate) fn to_context(args: HashMap<String, Argument>) -> HashMap<String, Arg> {
    let mut v = HashMap::with_capacity(args.len());
    for (k, arg) in args {
        v.insert(k, arg.into());
    }
    v
}

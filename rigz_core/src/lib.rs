use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Result;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum Argument {
    None,
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Bool(bool),
    String(String),
    File(RigzFile),
    Object(HashMap<String, Argument>),
    List(Vec<Argument>),
    FunctionCall(FunctionCall),
    Definition(Definition),
    Error(String),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RigzFile {
    pub file: PathBuf,
    #[serde(skip_serializing, skip_deserializing)]
    internal: Option<File>,
}

impl RigzFile {}

impl Display for RigzFile {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.file.to_str().unwrap_or("<invalid-utf>"))
    }
}

impl Clone for RigzFile {
    fn clone(&self) -> Self {
        let file = self.file.clone();
        if file.exists() {
            RigzFile {
                internal: Some(File::open(&file).expect("Failed to open file")),
                file,
            }
        } else {
            RigzFile {
                file,
                internal: None,
            }
        }
    }
}

impl PartialEq for RigzFile {
    fn eq(&self, other: &Self) -> bool {
        self.file == other.file
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[repr(C)]
pub enum Definition {
    None,
    One(HashMap<String, Argument>),
    Many(Vec<Argument>),
}

impl Display for Argument {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Argument::None => write!(f, "none"),
            Argument::Int(i) => write!(f, "{}", i),
            Argument::Long(l) => write!(f, "{}", l),
            Argument::Float(fl) => write!(f, "{}", fl),
            Argument::Double(d) => write!(f, "{}", d),
            Argument::Bool(b) => write!(f, "{}", b),
            Argument::String(s) => write!(f, "{}", s),
            Argument::Object(o) => write!(f, "{:?}", o),
            Argument::List(l) => write!(f, "{:?}", l),
            Argument::FunctionCall(fc) => write!(f, "{:?}", fc),
            Argument::Definition(d) => write!(f, "{:?}", d),
            Argument::Error(e) => write!(f, "Error: {}", e),
            Argument::File(file) => write!(f, "{}", file),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct FunctionCall {
    pub name: String,
    pub args: Vec<Argument>,
    pub definition: Definition,
}

#[derive(Debug, PartialEq)]
pub enum RuntimeStatus<T> {
    Ok(T),
    NotFound,
    Err(String),
}

pub trait Module {
    fn name(&self) -> &str;

    fn root(&self) -> PathBuf;

    fn function_call(
        &self,
        name: &str,
        arguments: Vec<Argument>,
        definition: Definition,
        prior_result: Argument,
    ) -> RuntimeStatus<Argument>;

    fn initialize(&self) -> RuntimeStatus<()> {
        RuntimeStatus::NotFound
    }
}

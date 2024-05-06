use std::collections::HashMap;
use std::ffi::{c_char, c_int, c_void};
use std::fmt::Result;
use std::fmt::{Display, Formatter};
use std::str::Utf8Error;
use log::{error, warn};

#[derive(Clone, Debug)]
#[repr(C)]
pub enum Argument {
    None(),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Bool(bool),
    String(StrSlice),
    Object(ArgumentMap),
    List(ArgumentVector),
    FunctionCall(Function),
    Error(StrSlice),
}

impl Display for Argument {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Argument::None() => write!(f, "none"),
            Argument::Int(i) => write!(f, "{}", i),
            Argument::Long(l) => write!(f, "{}", l),
            Argument::Float(fl) => write!(f, "{}", fl),
            Argument::Double(d) => write!(f, "{}", d),
            Argument::Bool(b) => write!(f, "{}", b),
            Argument::String(s) => write!(f, "{}", s),
            Argument::Object(o) => write!(f, "{}", o),
            Argument::List(l) => write!(f, "{}", l),
            Argument::FunctionCall(fc) => write!(f, "{:?}", fc),
            Argument::Error(e) => write!(f, "Error: {}", e),
        }
    }
}

#[no_mangle]
pub extern "C" fn argument_to_str(argument: Argument) -> StrSlice {
    argument.to_string().into()
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct StrSlice {
    pub ptr: *const u8,
    pub len: usize,
}

impl Display for StrSlice {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if self.ptr.is_null() {
            return write!(f, "<null pointer>");
        }

        let slice = unsafe { std::slice::from_raw_parts(self.ptr, self.len) };
        let string = std::str::from_utf8(slice).unwrap_or_else(|e| {
            error!("Invalid UTF-8 in StrSlice: {}", e);
            "<invalid utf-8>"
        });
        write!(f, "{}", string)
    }
}

impl Into<String> for StrSlice {
    fn into(self) -> String {
        let slice = unsafe { std::slice::from_raw_parts(self.ptr, self.len) };
        std::str::from_utf8(slice)
            .unwrap_or("<invalid utf-8>")
            .to_string()
    }
}

impl From<&str> for StrSlice {
    fn from(value: &str) -> Self {
        StrSlice {
            ptr: value.as_ptr(),
            len: value.len(),
        }
    }
}

impl From<String> for StrSlice {
    fn from(value: String) -> Self {
        let len = value.len();
        let boxed_str = Box::leak(value.into_boxed_str());
        StrSlice {
            ptr: boxed_str.as_ptr(),
            len
        }
    }
}

impl From<&String> for StrSlice {
    fn from(value: &String) -> Self {
        StrSlice {
            ptr: value.as_ptr(),
            len: value.len(),
        }
    }
}

#[derive(Clone, Debug)]
#[repr(C)]
pub enum ArgumentDefinition {
    Empty(),
    One(ArgumentMap),
    Many(ArgumentVector),
}

impl Display for ArgumentDefinition {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            ArgumentDefinition::Empty() => write!(f, "none"),
            ArgumentDefinition::One(map) => write!(f, "{}", map),
            ArgumentDefinition::Many(vec) => write!(f, "{}", vec),
        }
    }
}

#[no_mangle]
pub extern "C" fn definition_to_str(definition: ArgumentDefinition) -> StrSlice {
    definition.to_string().into()
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct Function {
    pub a: i32,
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct ArgumentMap {
    pub keys: *const *mut c_char,
    pub values: *const Argument,
    pub len: usize,
}

impl Display for ArgumentMap {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let keys_slice = unsafe { std::slice::from_raw_parts(self.keys, self.len) };
        let keys: Vec<String> = keys_slice
            .iter()
            .map(|&key_ptr| {
                let key_cstr = unsafe { std::ffi::CStr::from_ptr(key_ptr) };
                key_cstr.to_string_lossy().into_owned()
            })
            .collect();

        let values_slice = unsafe { std::slice::from_raw_parts(self.values, self.len) };
        let values: Vec<Argument> = values_slice.to_vec();

        for (key, value) in keys.iter().zip(values.iter()) {
            write!(f, "{}: {}, ", key, value)?;
        }

        Ok(())
    }
}

impl ArgumentMap {
    // Function to convert a Rust HashMap to Map
    pub fn from_hashmap(map: HashMap<String, Argument>) -> Self {
        let len = map.len();
        let mut keys = Vec::with_capacity(len);
        let mut values = Vec::with_capacity(len);

        for (key, value) in map {
            let key_cstring = std::ffi::CString::new(key).expect("Failed to create CString");
            keys.push(key_cstring.into_raw());

            values.push(value);
        }

        let keys_ptr = keys.as_ptr();
        let values_ptr = values.as_ptr();

        std::mem::forget(keys);
        std::mem::forget(values);

        ArgumentMap {
            keys: keys_ptr,
            values: values_ptr,
            len,
        }
    }

    pub fn to_hashmap(self) -> HashMap<String, Argument> {
        let mut map = HashMap::new();

        unsafe {
            let keys = std::slice::from_raw_parts(self.keys, self.len);
            let values = std::slice::from_raw_parts(self.values, self.len);

            for i in 0..self.len {
                let key = std::ffi::CStr::from_ptr(keys[i])
                    .to_string_lossy()
                    .into_owned();
                let value = values[i].clone();
                map.insert(key, value);
            }
        }

        map
    }
}

impl From<HashMap<String, Argument>> for ArgumentMap {
    fn from(value: HashMap<String, Argument>) -> Self {
        Self::from_hashmap(value)
    }
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct ArgumentVector {
    pub ptr: *const Argument,
    pub len: usize,
}

#[no_mangle]
pub extern "C" fn arguments_to_str(arguments: ArgumentVector) -> StrSlice {
    arguments.to_string().into()
}

impl Display for ArgumentVector {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let slice = unsafe { std::slice::from_raw_parts(self.ptr, self.len) };
        let formatted_values: Vec<String> = slice.iter().map(|arg| format!("{}", arg)).collect();
        write!(f, "[{}]", formatted_values.join(", "))
    }
}

impl ArgumentVector {
    pub fn to_vec(self) -> Vec<Argument> {
        unsafe {
            let slice = std::slice::from_raw_parts(self.ptr, self.len);
            slice.to_vec()
        }
    }
}

impl From<Vec<Argument>> for ArgumentVector {
    fn from(value: Vec<Argument>) -> Self {
        ArgumentVector {
            ptr: value.as_ptr(),
            len: value.len(),
        }
    }
}

// #[derive(Clone, Debug)]
// #[repr(C)]
// pub struct LibraryVector {
//     pub ptr: *const Library,
//     pub len: usize,
// }
//
// #[no_mangle]
// pub extern "C" fn module_count(v: LibraryVector) -> usize {
//     v.len // Needed for bingen
// }
//
// impl From<Vec<Library>> for LibraryVector {
//     fn from(value: Vec<Library>) -> Self {
//         LibraryVector {
//             ptr: value.as_ptr(),
//             len: value.len(),
//         }
//     }
// }

#[derive(Clone)]
#[repr(C)]
pub struct Library {
    pub name: StrSlice,
    pub handle: *const c_void,
    pub format: FunctionFormat,
    pub pass_through: *const fn(StrSlice, ArgumentVector, ArgumentDefinition, Argument) -> RuntimeStatus,
}

#[repr(C)]
pub struct RuntimeStatus {
    pub status: c_int,
    pub value: Argument,
    pub error_message: *const c_char,
}

#[derive(Clone, Default)]
#[repr(C)]
pub enum FunctionFormat {
    #[default]
    FIXED, // function name matches libary declaration - fn(ArgumentVector, ArgumentDefinition, Argument) RuntimeStatus
    PASS, // pass through - fn(StrSlice, ArgumentVector, ArgumentDefinition, Argument) RuntimeStatus
    // DYNAMIC TODO: call raw signatures directly, ie: fn add(i32, i32) -> i32
}

#[no_mangle]
pub extern "C" fn create_library(name: StrSlice, handle: *const c_void, format: FunctionFormat, pass_through: *const fn(StrSlice, ArgumentVector, ArgumentDefinition, Argument) -> RuntimeStatus) -> Library {
    Library { name, handle, format, pass_through }
}


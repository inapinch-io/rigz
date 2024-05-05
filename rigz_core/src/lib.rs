use std::collections::HashMap;
use std::ffi::c_char;
use std::fmt::Result;
use std::fmt::{Display, Formatter};

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
        // Convert raw pointer to a slice
        let slice = unsafe { std::slice::from_raw_parts(self.ptr, self.len) };
        // Convert slice to a string
        let string = std::str::from_utf8(slice).unwrap_or("<invalid utf-8>");
        // Write the string to the formatter
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
        StrSlice {
            ptr: value.as_ptr(),
            len: value.len(),
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
    pub keys: *mut *mut c_char,
    pub values: *mut Argument, // Pointer to array of values
    pub len: usize,            // Length of the map
}

impl Display for ArgumentMap {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        // Convert raw pointer to a slice of raw pointers
        let keys_slice = unsafe { std::slice::from_raw_parts(self.keys, self.len) };
        // Iterate over the keys and format them
        let keys: Vec<String> = keys_slice
            .iter()
            .map(|&key_ptr| {
                // Convert each raw pointer to a CStr and then to a String
                let key_cstr = unsafe { std::ffi::CStr::from_ptr(key_ptr) };
                key_cstr.to_string_lossy().into_owned()
            })
            .collect();

        // Convert values to a Vec<Argument>
        let values_slice = unsafe { std::slice::from_raw_parts(self.values, self.len) };
        let values: Vec<Argument> = values_slice.to_vec();

        // Iterate over keys and values and format them together
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

            // Assuming Argument is a simple type or another C-compatible struct
            values.push(value); // Push the value directly
        }

        // Transfer ownership of vectors to raw pointers
        let keys_ptr = keys.as_mut_ptr();
        let values_ptr = values.as_mut_ptr();

        // Prevent vectors from deallocating memory
        std::mem::forget(keys);
        std::mem::forget(values);

        ArgumentMap {
            keys: keys_ptr,
            values: values_ptr,
            len,
        }
    }

    // Function to convert Map back to Rust HashMap
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

impl Display for ArgumentVector {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        // Convert raw pointer to a slice of Arguments
        let slice = unsafe { std::slice::from_raw_parts(self.ptr, self.len) };
        // Format each Argument and join them together with ", "
        let formatted_values: Vec<String> = slice.iter().map(|arg| format!("{}", arg)).collect();
        // Join the formatted values and write them to the formatter
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

use std::collections::HashMap;

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
    Object(Map),
    List(Vector),
    FunctionCall(Function),
    Symbol(StrSlice),
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct StrSlice {
    pub ptr: *const u8,
    pub len: usize,
}

impl From<&str> for StrSlice {
    fn from(value: &str) -> Self {
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
    One(Map),
    Many(Vector),
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct Function {
    pub a: i32,
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct Map {
    pub keys: *mut *mut std::os::raw::c_char, // Pointer to array of C strings (keys)
    pub values: *mut Argument,                // Pointer to array of values
    pub len: usize,                           // Length of the map
}

impl Map {
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

        Map {
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

impl From<HashMap<String, Argument>> for Map {
    fn from(value: HashMap<String, Argument>) -> Self {
        Self::from_hashmap(value)
    }
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct Vector {
    pub ptr: *const Argument,
    pub len: usize,
}

impl Vector {
    pub fn to_vec(self) -> Vec<Argument> {
        unsafe {
            let slice = std::slice::from_raw_parts(self.ptr, self.len);
            slice.to_vec()
        }
    }
}

impl From<Vec<Argument>> for Vector {
    fn from(value: Vec<Argument>) -> Self {
        Vector {
            ptr: value.as_ptr(),
            len: value.len(),
        }
    }
}

/*
   Not used, but needed one of the following for cbindgen to find types
    - https://github.com/mozilla/cbindgen/blob/master/docs.md#writing-your-c-api
*/
#[no_mangle]
pub extern "C" fn echo(argument: Argument, argument_definition: ArgumentDefinition) {
    println!(
        "Argument {:?}, Definition {:?}",
        argument, argument_definition
    )
}

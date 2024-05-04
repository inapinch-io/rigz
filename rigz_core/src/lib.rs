#[derive(Debug)]
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

#[derive(Debug)]
#[repr(C)]
pub struct StrSlice {
    ptr: *const u8,
    len: usize,
}

impl From<&str> for StrSlice {
    fn from(value: &str) -> Self {
        StrSlice {
            ptr: value.as_ptr(),
            len: value.len()
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub enum ArgumentDefinition {
    Empty(),
    One(Map),
    Many(Vector),
}

#[derive(Debug)]
#[repr(C)]
pub struct Function {
    a: i32
}

#[derive(Debug)]
#[repr(C)]
pub struct Map {
    a: i32
}

#[derive(Debug)]
#[repr(C)]
pub struct Vector {
    a: i32
}

/*
    Not used, but needed one of the following for cbindgen to find types
     - https://github.com/mozilla/cbindgen/blob/master/docs.md#writing-your-c-api
 */
#[no_mangle] pub extern fn echo(argument: Argument, argument_definition: ArgumentDefinition) {
    println!("Argument {:?}, Definition {:?}", argument, argument_definition)
}
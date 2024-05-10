# rigz_lua

The core module used for rigz currently, uses [mlua](https://github.com/mlua-rs/mlua/tree/master) to power the runtime.

```shell
cargo add rigz_lua
```

```rust
use std::collections::HashMap;
use std::path::PathBuf;
use rigz_lua::LuaModule;

fn main() {
    let modules: HashMap<String, Box<dyn Module>> = HashMap::new();
    
    let module_root: PathBuf = PathBuf::from("/path/to/module");
    let source_files = vec![PathBuf::from("module/path/to/file.lua")];
    let input_files: HashMap<String, Vec<File>> = HashMap::new();
    let config: Option<serde_value::Value> = None;
    let lua = LuaModule::new("hello_world", module_root, source_files, input_files, config);
    
    modules.insert(lua.name(), lua)
}
```
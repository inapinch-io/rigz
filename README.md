# rigz

A functional language written in Rust and Zig, meant to be rigged together with all functionality provided via modules.

## Installation
TODO

## Usage

### Console
Open a REPL
```shell
rigz console
```

### Init
Create a new rigz project
```shell
rigz init
```

### Run
Run a rigz project based on default config or `RIGZ_CONFIG` environment variable
```shell
rigz run
```

### Test
Test rigz code based 
```shell
rigz test <test_directory>
```

## Features
- Everything is a function_call or a value, no expressions
- Minimal standard library, can be excluded
- Supply functions by importing or writing modules (dynamic C compatible libraries)

## Inspiration
I was mainly inspired by Ruby, CSS, and Terraform when creating this language. When asking
myself how I'd want to create a terraform alternative, just as a thought exercise, I settled on 
everything being a function call. I'd been thinking of the idea for about a year and 
after a few failed attempts, here we are. 

### Why Rust & Zig?
I wanted to learn Zig, FFI, and get better at Rust. If I went full Rust it looked like I'd end up using 
[libloading](https://docs.rs/libloading/latest/libloading/) and I knew I'd want to add support for other formats 
(wasm, jars, who knows); Zig also seems like it'd be great for what I'm trying to do (C-interop, WASM, simpler) but 
Rust has its areas to shine too (enums, pest, clap, anyhow, JNI), so merging them together felt like the "only option" 
(even though this could've been much simpler).

## Goals
- Declarative functional language with no GC
- Modules written in other languages

## Contributions
Yes please! There is a ton of work to do and a lot that I'm learning, so I'd welcome Suggestions, Bug Fixes, and 
Roadmap contributions (see below). For anything else please start with an Issue, and we'll make sure it's something 
the language should support.

## Roadmap
1. LSP
2. File Types: yaml, toml, hcl, opentofu, pkl
3. Script Modules: shell, lua, python, ruby, js
4. Query Modules: jq, xpath, html, AST/ANTLR
5. Utils Modules: HTTP, GraphQL, sqlite, events, matcher
6. Library Modules: wasm, jars, jvm scripting module, erlang
7. Projects
   - polc - Policy Engine with rigz
   - shortkey - inspired by [autohotkey](autohotkey.com)
8. Hosted rigz (serverless & long running)
9. glue, this was the last attempt [aq/psh](https://gitlab.com/magicfoodhand/aq_cli)(I couldn't decide on a name) but 
shows the syntax, fully interpreted functional language meant as a shell alternative (very similar philosophy, this time 
with expressions).
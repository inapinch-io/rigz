# rigz

A functional language written in Rust and Zig, all functionality is provided via modules.

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

## Goals
- Declarative functional language with no GC
- Modules written in other languages

## Roadmap
1. LSP
2. File Types: yaml, toml, hcl, opentofu
3. Script Modules: shell, lua, python, ruby, js
4. Query Modules: jq, xpath, html, AST/ANTLR, etc
5. Utils Modules: HTTP, GraphQL, sqlite, events, matcher
6. Support wasm for dynamic libraries
7. Support jars for dynamic libraries, jvm scripting module
8. Projects
   - polc - Policy Engine with rigz
   - shortkey - inspired by [autohotkey](autohotkey.com)
9. Hosted rigz (serverless & long running)
10. glue, fully interpreted language meant as a shell alternative
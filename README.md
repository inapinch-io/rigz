# rigz

A functional language written in Rust, meant to be rigged together with all functionality provided via modules.

## Language

Everything is a function call or a value; in this case allow, :valid_account, & variables are functions while the objects
passed in are the definition (Empty/None, One/Object, Many/List).

- generated with `tree-sitter highlight main.rigz | ansi2html`

<!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 4.01 Transitional//EN" "http://www.w3.org/TR/html4/loose.dtd">
<html>
<head>
<meta http-equiv="Content-Type" content="text/html; charset=utf-8">
<title></title>
<style type="text/css">
.ansi2html-content { display: inline; white-space: pre-wrap; word-wrap: break-word; }
.body_foreground { color: #AAAAAA; }
.body_background { background-color: #000000; }
.inv_foreground { color: #000000; }
.inv_background { background-color: #AAAAAA; }
.ansi1 { font-weight: bold; }
.ansi38-26 { color: #005fd7; }
.ansi38-94 { color: #875f00; }
.ansi38-28 { color: #008700; }
</style>
</head>
<body class="body_foreground body_background" style="font-size: normal;" >
<pre class="ansi2html-content">
<span class="ansi38-26">allow</span>(<span class="ansi38-26">abc</span>) {
    <span class="ansi38-26">variables</span> {
        account = <span class="ansi38-26">:valid_account</span> 
    }
}

<span class="ansi38-26">testing</span> <span class="ansi38-28">"hi"</span> [
<span class="ansi1 ansi38-94">  true</span>,
<span class="ansi1 ansi38-94">  false</span>,
<span class="ansi1 ansi38-94">  none</span>,
<span class="ansi1 ansi38-94">  123.456</span>,
]
</pre>
</body>

</html>


## Installation
```shell
cargo install rigz
```

## Usage

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

## Not Implemented Yet

### Console
Open a REPL
```shell
rigz console
```

### Test
Test rigz code based 
```shell
rigz test <test_directory>
```

## Features
- Everything is a function_call or a value, no expressions
- Minimal standard library, can be excluded
- Supply functions by importing or writing modules (LuaModule or custom types that implement Module)

## Inspiration
I was mainly inspired by Ruby, CSS, and Terraform when creating this language. When asking
myself how I'd want to create a terraform alternative, just as a thought exercise, I settled on 
everything being a function call. I'd been thinking of the idea for about a year and 
after a few failed attempts, here we are.

## Goals
- Declarative functional language with no GC
- Modules written in other languages

## Contributions
Yes please! There is a ton of work to do and a lot that I'm learning, so I'd welcome Suggestions, Bug Fixes, and 
Roadmap contributions (see below). For anything else please start with an Issue, and we'll make sure it's something 
the language should support.

## Roadmap
1. LSP
2. polc (Project/module) - Policy Engine backed by rigz
3. File Types (read/write): yaml, toml, hcl, opentofu, pkl
4. Script Modules: shell, python, ruby, js
5. Query Modules: jq, xpath, html, AST/ANTLR
6. Utils Modules: HTTP, GraphQL, sqlite, events, matcher
7. Library Modules: wasm, jars, jvm scripting module, erlang
8. Module Registry
9. shortkey (Project/module) - inspired by [autohotkey](autohotkey.com)
10. Hosted rigz (serverless & long running)
11. glue, this was the last attempt [aq/psh](https://gitlab.com/magicfoodhand/aq_cli)(I couldn't decide on a name) but 
shows the syntax, fully interpreted functional language meant as a shell alternative (very similar philosophy, this time 
with expressions). Ultimately the reason I decided to create rigz, believe it or not this was a way to start with 
something simpler.
# rigz parse
AST Parsing for rigz, see [grammar.pest](src/grammar.pest) for the full grammar.

## Example Syntax

Hello World
```rigz
puts 'Hello World'
```

Fictional Policy Language
```rigz
allow {
    variables {
        account = :valid_account 
    }
}
```

Fictional Policy Language (Part 2)
```rigz
deny {
    variables {
        account = unless :valid_account 
    }
}
```
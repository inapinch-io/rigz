pub mod parse;
pub mod run;

use std::collections::HashMap;
use anyhow::Result;
use rigz_parse::AST;
use crate::parse::{parse_source_files, ParseOptions};

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

pub struct Options {
    pub parse: ParseOptions,
}

pub struct Runtime {
    ast: AST,
    symbols: HashMap<String, Symbol>
}

impl Runtime {
    pub fn get_function(&self, name: &String) -> Option<Function> {
        None
    }
}

#[derive(Debug)]
pub struct Function {

}

#[derive(Debug)]
pub struct Symbol {

}

pub fn initialize(options: Options) -> Result<Runtime> {
    let mut ast = AST::init();
    parse_source_files(&mut ast, options.parse)?;
    let mut symbols = HashMap::new();
    Ok(Runtime { ast, symbols })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

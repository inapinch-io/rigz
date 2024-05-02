pub mod parse;
pub mod run;

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
}

pub fn initialize(options: Options) -> Result<Runtime> {
    let mut ast = AST::init();
    parse_source_files(&mut ast, options.parse)?;

    Ok(Runtime { ast })
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

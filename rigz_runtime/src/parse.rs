use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use anyhow::anyhow;
use glob::{glob_with, MatchOptions};
use serde::Deserialize;
use rigz_parse::{AST, parse, ParseConfig};

#[derive(Clone, Default, Deserialize)]
pub struct ParseOptions {
    pub use_64_bit_numbers: bool,
    pub source_files_patterns: Vec<String>,
    pub glob_options: GlobOptions,
}

// Options copied from https://github.com/rust-lang/glob/blob/master/src/lib.rs#L1041-L1061
#[derive(Clone, Default, Deserialize)]
pub struct GlobOptions {
    pub case_sensitive: bool,
    pub require_literal_separator: bool,
    pub require_literal_leading_dot: bool,
}

impl Into<MatchOptions> for GlobOptions {
    fn into(self) -> MatchOptions {
        MatchOptions {
            case_sensitive: self.case_sensitive,
            require_literal_separator: self.require_literal_separator,
            require_literal_leading_dot: self.require_literal_leading_dot,
        }
    }
}

fn find_source_files(patterns: Vec<String>, match_options: MatchOptions) -> anyhow::Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    for pattern in patterns {
        let results = glob_with(pattern.as_str(), match_options)
            .expect(format!("Failed to read pattern: {}", pattern.as_str()).as_str());

        for result in results {
            match result {
                Ok(path) => {
                    paths.push(path);
                }
                Err(e) => {
                    return Err(anyhow!("Pattern Failed {} {}", pattern, e))
                }
            }
        }
    }
    Ok(paths)
}

pub(crate) fn parse_source_files(ast: &mut AST, parse_options: ParseOptions) -> anyhow::Result<()> {
    let ast_config = ParseConfig {
        use_64_bit_numbers: parse_options.use_64_bit_numbers,
    };
    for path in find_source_files(parse_options.source_files_patterns, parse_options.glob_options.into())? {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        ast.merge(parse(contents, &ast_config)?);
    }
    Ok(())
}
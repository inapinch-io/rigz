use anyhow::anyhow;
use glob::{glob_with, MatchOptions};
use log::warn;
use rigz_parse::{parse, ParseConfig, AST};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

#[derive(Clone, Default, Deserialize)]
pub struct ParseOptions {
    pub use_64_bit_numbers: Option<bool>,
    pub source_files: Vec<String>,
    pub glob_options: Option<GlobOptions>,
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

fn find_source_files(
    patterns: Vec<String>,
    match_options: MatchOptions,
) -> anyhow::Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    let patterns = if patterns.is_empty() {
        warn!("No `parse.source_files` provided, using default of *.rigz");
        vec!["*.rigz".to_string()]
    } else {
        patterns
    };
    for pattern in patterns {
        let results = glob_with(pattern.as_str(), match_options)
            .expect(format!("Failed to read pattern: {}", pattern.as_str()).as_str());

        for result in results {
            match result {
                Ok(path) => {
                    paths.push(path);
                }
                Err(e) => return Err(anyhow!("Pattern Failed {} {}", pattern, e)),
            }
        }
    }
    Ok(paths)
}

pub(crate) fn parse_source_files(
    parse_options: ParseOptions,
) -> anyhow::Result<HashMap<String, AST>> {
    let mut asts = HashMap::new();
    let ast_config = ParseConfig {
        use_64_bit_numbers: parse_options.use_64_bit_numbers.unwrap_or(false),
    };
    let glob = parse_options
        .glob_options
        .unwrap_or(GlobOptions::default())
        .into();
    for path in find_source_files(parse_options.source_files, glob)? {
        let mut file = File::open(&path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let filename = path
            .file_name()
            .map(|s| s.to_str().expect("Failed to convert OsStr to string"))
            .expect(format!("Failed to get filename for {:?}", path).as_str());
        asts.insert(filename.to_string(), parse(contents, &ast_config)?);
    }
    Ok(asts)
}

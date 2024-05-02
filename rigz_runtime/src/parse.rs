use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use anyhow::anyhow;
use glob::{glob_with, MatchOptions};
use rigz_parse::{AST, parse, ParseConfig};

#[derive(Clone)]
pub struct ParseOptions {
    use_64_bit_numbers: bool,
    source_files_patterns: Vec<String>,
    match_options: MatchOptions,
}

fn load_source_files(patterns: Vec<String>, match_options: MatchOptions) -> anyhow::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for pattern in patterns {
        let results = glob_with(pattern.as_str(), match_options)
            .expect(format!("Failed to read pattern: {}", pattern.as_str()).as_str());

        for result in results {
            match result {
                Ok(path) => {
                    files.push(path);
                }
                Err(e) => {
                    return Err(anyhow!("Pattern Failed {} {}", pattern, e))
                }
            }
        }
    }
    Ok(files)
}

pub(crate) fn parse_source_files(ast: &mut AST, parse_options: ParseOptions) -> anyhow::Result<()> {
    let ast_config = ParseConfig {
        use_64_bit_numbers: parse_options.use_64_bit_numbers,
    };
    for path in load_source_files(parse_options.source_files_patterns, parse_options.match_options)? {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        ast.merge(parse(contents, &ast_config)?);
    }
    Ok(())
}
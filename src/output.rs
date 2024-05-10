use anyhow::anyhow;
use log::info;
use rigz_runtime::run::RunResult;

#[derive(Clone, Default, Debug)]
pub enum OutputFormat {
    #[default]
    PRINT,
    JSON,
    LOG,
}

pub fn handle_result(output: Option<String>, result: RunResult) -> anyhow::Result<()> {
    let format = match output {
        None => OutputFormat::default(),
        Some(f) => {
            let fmt = f.trim().to_lowercase();
            if fmt == "json" {
                OutputFormat::JSON
            } else {
                return Err(anyhow!("Invalid Format: `{}`", fmt));
            }
        }
    };
    match format {
        OutputFormat::PRINT => {
            println!("Results:");
            for (file, value) in result.value {
                println!("\t{}: {}", file, value)
            }
        }
        OutputFormat::LOG => {
            info!("Results:");
            for (file, value) in result.value {
                info!("\t{}: {}", file, value)
            }
        }
        _ => match format {
            OutputFormat::JSON => {
                let contents = serde_json::to_string_pretty(&result.value)?;
                println!("{}", contents)
            }
            _ => return Err(anyhow!("Unsupported Output Format {:?}", format)),
        },
    }
    Ok(())
}

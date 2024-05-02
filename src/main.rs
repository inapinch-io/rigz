use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(next_line_help = true)]
pub struct CLI {
    config_path: PathBuf,
}

fn main() {
    println!("Hello, world!");
}

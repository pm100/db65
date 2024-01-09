#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_lossless)]
use crate::shell::Shell;
use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
mod cpu;
mod debugdb;
mod debugger;
mod dis;
mod execute;
mod expr;
mod loader;
mod paravirt;
mod parsedb;
mod shell;
mod syntax;

pub mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "FILE")]
    binary: Option<PathBuf>,

    #[arg(short, long, value_name = "FILE")]
    command_file: Option<PathBuf>,

    #[arg(short, long)]
    set_exit: bool,
    #[arg(last = true)]
    args: Vec<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    println!(
        "db65 {} ({})",
        built_info::PKG_VERSION,
        built_info::GIT_COMMIT_HASH_SHORT.unwrap_or_default()
    );
    let mut sh = Shell::new();
    sh.shell(cli.command_file, &cli.args)?;

    Ok(())
}

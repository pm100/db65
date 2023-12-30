use crate::shell::Shell;
use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
mod cpu;
mod debugger;
mod dis;
mod execute;
mod loader;
mod paravirt;
mod shell;
mod syntax;
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

    let mut sh = Shell::new();
    sh.shell(cli.command_file, cli.args)?;

    Ok(())
}

mod cpu;
mod debugger;
mod dis;
mod execute;
mod loader;
mod paravirt;
mod shell;

use anyhow::Result;
use std::path::PathBuf;

use crate::shell::Shell;
use clap::Parser;
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Optional name to operate on
    name: Option<String>,

    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    command_file: Option<PathBuf>,

    /// Turn debugging information on
    #[arg(short, long)]
    set_exit: bool,
    // #[command(subcommand)]
    // command: Option<Commands>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    // if  cli.set_exit
    let mut sh = Shell::new();
    sh.shell(cli.command_file)?;

    Ok(())
}

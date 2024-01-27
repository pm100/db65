#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_lossless)]
use crate::{
    log::{init_log, set_say_cb},
    shell::Shell,
};
use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

mod log;
mod db {
    pub mod debugdb;
    pub mod parsedb;
    pub mod setupdb;
    pub mod util;
}
mod debugger {
    pub mod core;
    pub mod cpu;
    pub mod execute;
    pub mod intercepts;
    pub mod loader;
    pub mod paravirt;
}
mod dis;

mod about;
mod expr;
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
    init_log();
    set_say_cb(|s| println!("{}", s));
    println!(
        "db65 sim6502 debugger {} ({})",
        built_info::PKG_VERSION,
        built_info::GIT_COMMIT_HASH_SHORT.unwrap_or_default()
    );
    println!("use 'help' to get help for commands and 'about' for more information");
    let mut sh = Shell::new();
    sh.shell(cli.command_file, &cli.args)?;
    Ok(())
}

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_lossless)]
use crate::{log::init_log, shell::Shell};
use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

#[macro_export]
macro_rules! tracexx {
    ($fmt:literal, $($arg:expr),*) => {
        #[cfg(debug_assertions)]
        {
            if cfg!(test){
                println!($fmt, $($arg),*);
            } else {
                log::trace!($fmt, $($arg),*);
            }
        }
    };
    ($msg:expr) => {
        #[cfg(debug_assertions)]
        {
            if cfg!(test){
                println!($msg);
            } else {
                log::trace!($msg);
            }
        }
    };
}

mod log;
mod db {
    pub mod debugdb;
    pub mod parsedb;
    pub mod setupdb;
}
mod debugger {
    pub mod cpu;
    pub mod debugger;
    pub mod execute;
    pub mod intercepts;
    pub mod loader;
    pub mod paravirt;
}
mod dis;

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
    println!(
        "db65 sim6502 debugger {} ({})",
        built_info::PKG_VERSION,
        built_info::GIT_COMMIT_HASH_SHORT.unwrap_or_default()
    );
    let mut sh = Shell::new();
    sh.shell(cli.command_file, &cli.args)?;
    Ok(())
}

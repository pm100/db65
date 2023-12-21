use std::os::raw::c_char;
mod cpu;
mod execute;
mod loader;
mod shell;

use crate::cpu::Sim;
use anyhow::Result;
fn main() -> Result<()> {
    let (sp, run, cpu) = loader::load_code().unwrap();
    println!("sp={:x}, entry={:x}, cpu={}", sp, run, cpu);
    execute::execute(sp, run);

    Ok(())
}

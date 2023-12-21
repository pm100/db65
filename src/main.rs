mod cpu;
mod debugger;
mod execute;
mod loader;
mod shell;

use anyhow::Result;

use crate::shell::Shell;
fn main() -> Result<()> {
    let mut sh = Shell::new();
    sh.shell();
    let (sp, run, cpu) = loader::load_code().unwrap();
    println!("sp={:x}, entry={:x}, cpu={}", sp, run, cpu);
    execute::execute(sp, run);

    Ok(())
}

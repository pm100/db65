mod cpu;
mod debugger;
mod execute;
mod loader;
mod paravirt;
mod shell;

use anyhow::Result;

use crate::shell::Shell;
fn main() -> Result<()> {
    let mut sh = Shell::new();

    // let (sp, run, cpu) = loader::load_code().unwrap();
    //println!("sp={:x}, entry={:x}, cpu={}", sp, run, cpu);
    // execute::execute(sp, run);
    sh.shell();
    Ok(())
}

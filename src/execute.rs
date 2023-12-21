use anyhow::Result;

use crate::cpu::Sim;
pub fn execute(sp: u8, start: u16) -> Result<()> {
    Sim::reset();
    //Sim::write_sp(sp);
    Sim::write_word(0xFFFC, start);
    Sim::reset();
    for _i in 0..700 {
        println!("PC:{:04x}, A:{:02x}", Sim::read_pc(), Sim::read_ac());
        Sim::execute_insn();
        if Sim::exit_done() {
            break;
        }
    }
    Ok(())
}

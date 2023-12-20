use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::os::raw::c_char;
mod cpu;
mod loader;
/*
extern "C" {
    pub fn ExecuteInsn() -> u32;

}
extern "C" {
    pub fn Reset();

}
extern "C" {
    pub fn ReadRegisters() -> *mut CPURegs;

}
#[no_mangle]
extern "C" fn MemWriteByte(_addr: u32, _val: u8) {
    println!("MemWriteByte {:04x} {:02x}", _addr, _val);
}

#[no_mangle]
extern "C" fn MemReadWord(_addr: u32) -> u32 {
    println!("MemReadWord {:04x}", _addr);
    return 0;
}
#[no_mangle]
extern "C" fn MemReadByte(_addr: u32) -> u8 {
    println!("MemReadByte {:04x}", _addr);
    return 0;
}
#[no_mangle]
extern "C" fn MemReadZPWord(_addr: u32) -> u32 {
    println!("MemReadZPWord");
    return 0;
}
#[no_mangle]
extern "C" fn Warning(_format: *const c_char, x: u32, y: u32) -> u32 {
    println!("Warning");
    return 0;
}
#[no_mangle]
extern "C" fn Error(_format: *const c_char, x: u32, y: u32) -> u32 {
    println!("Error");
    return 0;
}
#[repr(C)]
pub struct CPURegs {
    ac: u32, /* Accumulator */
    xr: u32, /* X register */
    yr: u32, /* Y register */
    zr: u32, /* Z register */
    sr: u32, /* Status register */
    sp: u32, /* Stackpointer */
    pc: u32, /* Program counter */
}
#[no_mangle]

extern "C" fn ParaVirtHooks(regs: *mut CPURegs) {
    println!("MemzpReadByte");
}
*/
use crate::cpu::Sim;
use anyhow::Result;
fn main() -> Result<()> {
    // `()` can be used when no completer is required
    let mut rl = DefaultEditor::new()?;
    #[cfg(feature = "with-file-history")]
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }
    let (sp, run) = loader::load_code().unwrap();
    // unsafe {
    Sim::reset();
    Sim::execute_insn();
    //let regs = crate::cpu::Sim::read_registers();
    println!("ac: {:02x}", Sim::read_ac());
    println!("xr: {:02x}", Sim::read_xr());
    println!("yr: {:02x}", Sim::read_yr());
    println!("zr: {:02x}", Sim::read_zr());
    println!("sr: {:02x}", Sim::read_sr());
    println!("sp: {:02x}", Sim::read_sp());
    println!("pc: {:02x}", Sim::read_pc());
    //}
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                println!("Line: {}", line);
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    #[cfg(feature = "with-file-history")]
    rl.save_history("history.txt");
    Ok(())
}

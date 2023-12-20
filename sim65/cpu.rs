struct Cpu {
    ram: [u8; 65536],
    // registers
    ac: u8,
    xr: u8,
    yr: u8,
    zr: u8,
    sr: u8,
    sp: u8,
    pc: u16,
}
use std::os::raw::c_char;

// our callable functions into sim65
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

// callback from sim65 to us
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

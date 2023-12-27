use std::os::raw::c_char;

use crate::paravirt::ParaVirt;
static mut THECPU: Cpu = Cpu {
    ram: [0; 65536],
    shadow: [0; 65536],
    regs: std::ptr::null_mut(),
    sp65_addr: 0,
    exit: false,
    exit_code: 0,
    memcheck: None,
};
struct Cpu {
    ram: [u8; 65536],
    shadow: [u8; 65536],
    // registers
    regs: *mut CPURegs,
    exit: bool,
    exit_code: u8,
    sp65_addr: u8,
    memcheck: Option<u16>,
}

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

// callback from sim65 to us
#[no_mangle]
extern "C" fn MemWriteByte(_addr: u32, _val: u8) {
    // println!("MemWriteByte {:04x} {:02x}", _addr, _val);
    unsafe {
        THECPU.write_byte(_addr as u16, _val);
        THECPU.shadow[_addr as usize] = 1;
    }
}

#[no_mangle]
extern "C" fn MemReadWord(addr: u32) -> u32 {
    unsafe {
        let w = THECPU.read_word(addr as u16) as u32;
        if THECPU.shadow[addr as usize] == 0 {
            //println!("MemReadByte {:04x} = {:02x}", addr, b);
            THECPU.memcheck = Some(addr as u16);
        } else if THECPU.shadow[(addr + 1) as usize] == 0 {
            //println!("MemReadByte {:04x} = {:02x}", addr, b);
            THECPU.memcheck = Some(addr as u16 + 1);
        }
        //   println!("MemReadWord {:04x} = {:04x}", addr, w);
        w
    }
}
#[no_mangle]
extern "C" fn MemReadByte(addr: u32) -> u8 {
    unsafe {
        let b = THECPU.read_byte(addr as u16);
        if THECPU.shadow[addr as usize] == 0 {
            //println!("MemReadByte {:04x} = {:02x}", addr, b);
            THECPU.memcheck = Some(addr as u16);
        }
        // println!("MemReadByte {:04x} = {:02x}", addr, b);
        b
    }
}
#[no_mangle]
extern "C" fn MemReadZPWord(mut addr: u8) -> u16 {
    //println!("MemReadZPWord");
    unsafe {
        let b1 = THECPU.read_byte(addr as u16) as u16;
        addr = addr.wrapping_add(1);
        let b2 = THECPU.read_byte(addr as u16) as u16;
        return b1 | (b2 << 8);
    }
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
#[no_mangle]

extern "C" fn ParaVirtHooks(_regs: *mut CPURegs) {
    let pc = Sim::read_pc();
    //  println!("==>ParHook {:04x}", pc);

    ParaVirt::pv_hooks();
}
#[repr(C)]
pub struct CPURegs {
    pub ac: u32, /* Accumulator */
    pub xr: u32, /* X register */
    pub yr: u32, /* Y register */
    pub zr: u32, /* Z register */
    pub sr: u32, /* Status register */
    pub sp: u32, /* Stackpointer */
    pub pc: u32, /* Program counter */
}
pub struct Sim;
impl Sim {
    pub fn sp65_addr(v: u8) {
        unsafe {
            THECPU.sp65_addr = v;
        }
    }
    pub fn get_memcheck() -> Option<u16> {
        unsafe { THECPU.memcheck }
    }
    pub fn clear_memcheck() {
        unsafe {
            THECPU.memcheck = None;
        }
    }
    pub fn set_exit(code: u8) {
        unsafe {
            THECPU.exit = true;
            THECPU.exit_code = code;
        }
    }
    pub fn get_sp65_addr() -> u8 {
        unsafe { THECPU.sp65_addr }
    }
    pub fn exit_done() -> Option<u8> {
        unsafe {
            return if THECPU.exit {
                Some(THECPU.exit_code)
            } else {
                None
            };
        }
    }
    pub fn reset() {
        unsafe {
            THECPU.regs = ReadRegisters();
            THECPU.exit = false;
            Reset();
        }
    }
    pub fn execute_insn() -> u32 {
        unsafe { ExecuteInsn() }
    }
    pub fn write_ac(v: u8) {
        unsafe {
            (*THECPU.regs).ac = v as u32;
        }
    }
    pub fn write_xr(v: u8) {
        unsafe {
            (*THECPU.regs).xr = v as u32;
        }
    }
    pub fn write_sp(v: u8) {
        unsafe {
            (*THECPU.regs).sp = v as u32;
        }
    }
    pub fn write_yr(v: u8) {
        unsafe {
            (*THECPU.regs).yr = v as u32;
        }
    }
    pub fn write_zr(v: u8) {
        unsafe {
            (*THECPU.regs).zr = v as u32;
        }
    }
    pub fn write_sr(v: u8) {
        unsafe {
            (*THECPU.regs).sr = v as u32;
        }
    }
    pub fn write_pc(v: u16) {
        unsafe {
            (*THECPU.regs).pc = v as u32;
        }
    }
    pub fn read_ac() -> u8 {
        unsafe { (*THECPU.regs).ac as u8 }
    }
    pub fn read_xr() -> u8 {
        unsafe { (*THECPU.regs).xr as u8 }
    }
    pub fn read_yr() -> u8 {
        unsafe { (*THECPU.regs).yr as u8 }
    }
    pub fn read_zr() -> u8 {
        unsafe { (*THECPU.regs).zr as u8 }
    }
    pub fn read_sr() -> u8 {
        unsafe { (*THECPU.regs).sr as u8 }
    }
    pub fn read_sp() -> u8 {
        unsafe { (*THECPU.regs).sp as u8 }
    }
    pub fn read_pc() -> u16 {
        unsafe { (*THECPU.regs).pc as u16 }
    }
    pub fn write_byte(addr: u16, val: u8) {
        unsafe {
            THECPU.write_byte(addr, val);
            THECPU.shadow[addr as usize] = 1;
        }
    }
    pub fn write_word(addr: u16, val: u16) {
        unsafe {
            THECPU.write_word(addr, val);
        }
    }

    pub fn read_byte(addr: u16) -> u8 {
        unsafe { THECPU.read_byte(addr) }
    }
    pub fn read_word(addr: u16) -> u16 {
        unsafe { THECPU.read_word(addr) }
    }
}

impl Cpu {
    fn read_byte(&mut self, addr: u16) -> u8 {
        self.ram[addr as usize]
    }
    fn read_word(&mut self, addr: u16) -> u16 {
        let b1 = self.ram[addr as usize] as u16;
        let b2 = self.ram[(addr + 1) as usize] as u16;
        b1 | (b2 << 8)
    }
    fn write_byte(&mut self, addr: u16, val: u8) {
        self.ram[addr as usize] = val;
    }
    fn write_word(&mut self, addr: u16, val: u16) {
        self.ram[addr as usize] = (val & 0xff) as u8;
        self.ram[(addr + 1) as usize] = (val >> 8) as u8;
    }
}
#[test]
fn regreadwrite() {
    Sim::reset();
    Sim::write_ac(1);
    Sim::write_xr(2);
    Sim::write_yr(3);
    Sim::write_zr(4);
    Sim::write_sr(5);
    Sim::write_sp(6);
    Sim::write_pc(0x7777);

    assert_eq!(Sim::read_ac(), 1);
    assert_eq!(Sim::read_xr(), 2);
    assert_eq!(Sim::read_yr(), 3);
    assert_eq!(Sim::read_zr(), 4);
    assert_eq!(Sim::read_sr(), 5);
    assert_eq!(Sim::read_sp(), 6);
    assert_eq!(Sim::read_pc(), 0x7777);
}

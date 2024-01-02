/*
    Wrapper around the sim65 emulator.

    Provides the calls from db65 to 6502.c
    - ExecuteInsn to execute one instruction
    - ReadRegisters to get a pointer to the register block.
    - Reset to reset the cpu

    Provides the service routines that 6502.c needs
    - read and write ram
    - paravirt call backs
    - runtime warnings and errors

    Lots of unsafe code here because
    - we are doing c calls
    - we have a read / write singleton


*/

use crate::paravirt::ParaVirt;
use bitflags::bitflags;
use std::{ffi::CStr, fmt, os::raw::c_char};
#[derive(PartialEq, Copy, Clone)]
pub enum SystemType {
    Sim6502,
    Sim65c02,
    Kim1,
}

// the one cpu instance
// this is because the calls to us are 'naked' c calls
static mut THECPU: Cpu = Cpu {
    ram: [0; 65536],
    shadow: [0; 65536],
    regs: std::ptr::null_mut(),
    sp65_addr: 0,
    exit: false,
    exit_code: 0,
    memcheck: None,
    arg_array: Vec::new(),
    memhits: [(false, 0); 6],
    memhitcount: 0,
    paracall: false,
    ram_top: 0xffff,
    system: SystemType::Sim6502,
    brk: false,
};
pub struct Cpu {
    ram: [u8; 65536],          // the actual 6502 ram
    shadow: [u8; 65536],       // a shadow of the ram, used for memcheck
    regs: *mut CPURegs,        // a pointer to the register block
    exit: bool,                // set to true when the 6502 wants to exit
    exit_code: u8,             // the exit code
    sp65_addr: u8,             // the location of the cc65 'stack' pointer
    memcheck: Option<u16>,     // the address of the last memcheck failure
    arg_array: Vec<String>,    // the command line arguments
    memhits: [(bool, u16); 6], // used for data watches
    memhitcount: u8,           // entry count in hit array for this instruction
    pub paracall: bool,        // we just did a pv call
    ram_top: u16,              // the top of ram
    pub system: SystemType,    // what are we emulating
    pub brk: bool,             // break
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

extern "C" {
    static mut CPU: u16;
    // decalred static in 6502.c so we cant do this
    // static mut Regs: CPURegs;
}

// callback from sim65 to us
#[no_mangle]
extern "C" fn MemWriteByte(addr: u32, val: u8) {
    unsafe {
        THECPU.inner_write_byte(addr as u16, val);
        THECPU.shadow[addr as usize] = 1;
        THECPU.memhits[THECPU.memhitcount as usize] = (true, addr as u16);
        THECPU.memhitcount += 1;
    }
}
#[no_mangle]
extern "C" fn MemReadWord(addr: u32) -> u32 {
    unsafe {
        let w = THECPU.inner_read_word(addr as u16) as u32;
        if THECPU.shadow[addr as usize] == 0 {
            THECPU.memcheck = Some(addr as u16);
        } else if THECPU.shadow[(addr + 1) as usize] == 0 {
            THECPU.memcheck = Some(addr as u16 + 1);
        }
        THECPU.memhits[THECPU.memhitcount as usize] = (false, addr as u16);
        THECPU.memhits[(THECPU.memhitcount + 1) as usize] = (false, (addr + 1) as u16);
        THECPU.memhitcount += 2;

        w
    }
}
#[no_mangle]
extern "C" fn MemReadByte(addr: u32) -> u8 {
    unsafe {
        let b = THECPU.inner_read_byte(addr as u16);
        if THECPU.shadow[addr as usize] == 0 {
            THECPU.memcheck = Some(addr as u16);
        }
        THECPU.memhits[THECPU.memhitcount as usize] = (false, addr as u16);
        THECPU.memhitcount += 1;
        b
    }
}
#[no_mangle]
extern "C" fn MemReadZPWord(mut addr: u8) -> u16 {
    unsafe {
        let b1 = THECPU.inner_read_byte(addr as u16) as u16;
        addr = addr.wrapping_add(1);
        let b2 = THECPU.inner_read_byte(addr as u16) as u16;
        THECPU.memhits[THECPU.memhitcount as usize] = (false, addr as u16);
        THECPU.memhits[(THECPU.memhitcount + 1) as usize] = (false, (addr + 1) as u16);
        THECPU.memhitcount += 2;
        b1 | (b2 << 8)
    }
}
#[no_mangle]
extern "C" fn Warning(_format: *const c_char, _x: u32, _y: u32) -> u32 {
    println!("Warning");
    0
}
#[no_mangle]
extern "C" fn Error(format: *const c_char, _x: u32, _y: u32) -> u32 {
    let c_str = unsafe {
        assert!(!format.is_null());

        CStr::from_ptr(format)
    };
    println!("Error {:} {:02x} {:04x}", c_str.to_str().unwrap(), _x, _y);
    unsafe {
        THECPU.brk = true;
    }
    0
}
#[no_mangle]
extern "C" fn ParaVirtHooks(_regs: *mut CPURegs) {
    if ParaVirt::pv_hooks() {
        unsafe {
            // we need to know that a PV call was executed
            // thats becuase the stack will have been changed by the call
            // but there was no rts instruction,
            THECPU.paracall = true;
        }
    }
}
// the structure we gtr a pointer to with ReadRegisters
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

impl Cpu {
    pub fn set_cpu(cpu: u16) {
        unsafe {
            CPU = cpu;
        }
    }
    pub fn break_hit() -> bool {
        unsafe {
            let b = THECPU.brk;
            THECPU.brk = false;
            b
        }
    }
    pub fn sp65_addr(v: u8) {
        unsafe {
            THECPU.sp65_addr = v;
        }
    }
    pub fn was_paracall() -> bool {
        unsafe { THECPU.paracall }
    }
    pub fn set_system(system: SystemType) {
        unsafe {
            THECPU.system = system;
            if SystemType::Kim1 == system {
                THECPU.ram_top = 0x8000;
            }
        }
    }
    pub fn post_inst_reset() {
        unsafe {
            THECPU.memhitcount = 0;
            THECPU.paracall = false;
            THECPU.memcheck = None;
        }
    }

    pub fn get_memhitcount() -> u8 {
        unsafe { THECPU.memhitcount }
    }
    pub fn get_memhits() -> [(bool, u16); 6] {
        unsafe { THECPU.memhits }
    }
    pub fn get_arg_count() -> u8 {
        unsafe { THECPU.arg_array.len() as u8 }
    }
    pub fn get_arg(i: u8) -> &'static str {
        unsafe { THECPU.arg_array[i as usize].as_str() }
    }
    pub fn push_arg(v: &str) {
        unsafe {
            THECPU.arg_array.push(v.to_owned());
        }
    }
    pub fn get_memcheck() -> Option<u16> {
        unsafe { THECPU.memcheck }
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
            if THECPU.exit {
                Some(THECPU.exit_code)
            } else {
                None
            }
        }
    }
    pub fn reset() {
        unsafe {
            THECPU.regs = ReadRegisters();
            THECPU.exit = false;
            THECPU.memhitcount = 0;
            THECPU.arg_array.clear();
            Reset();
            THECPU.memcheck = None;
            THECPU.paracall = false;
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
    #[allow(dead_code)]
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
            THECPU.inner_write_byte(addr, val);
            THECPU.shadow[addr as usize] = 1;
        }
    }
    pub fn write_word(addr: u16, val: u16) {
        unsafe {
            THECPU.inner_write_word(addr, val);
        }
    }

    pub fn read_byte(addr: u16) -> u8 {
        unsafe { THECPU.inner_read_byte(addr) }
    }
    pub fn read_word(addr: u16) -> u16 {
        unsafe { THECPU.inner_read_word(addr) }
    }

    fn inner_read_byte(&mut self, addr: u16) -> u8 {
        if addr > self.ram_top || (self.system == SystemType::Kim1 && addr == 0x1740) {
            return 0xff;
        }
        self.ram[addr as usize]
    }
    fn inner_read_word(&mut self, addr: u16) -> u16 {
        let b1 = self.ram[addr as usize] as u16;
        let b2 = self.ram[(addr + 1) as usize] as u16;
        b1 | (b2 << 8)
    }
    fn inner_write_byte(&mut self, addr: u16, val: u8) {
        self.ram[addr as usize] = val;
    }
    fn inner_write_word(&mut self, addr: u16, val: u16) {
        self.ram[addr as usize] = (val & 0xff) as u8;
        self.ram[(addr + 1) as usize] = (val >> 8) as u8;
    }
}

bitflags! {
    #[derive(Copy, Clone, Default)]
   pub(crate) struct Status:u8{
        const CARRY =       0b0000_0001;
        const ZERO =        0b0000_0010;
        const IDISABLE =    0b0000_0100;
        const DECIMAL =     0b0000_1000;
        const BREAK =       0b0001_0000;
        const UNUSED =      0b0010_0000;
        const OVF =         0b0100_0000;
        const NEGATIVE =    0b1000_0000;

    }
}

impl fmt::Debug for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut str = String::new();
        if self.contains(Status::NEGATIVE) {
            str.push('N');
        } else {
            str.push('n');
        };
        if self.contains(Status::OVF) {
            str.push('O');
        } else {
            str.push('o');
        };
        str.push('-');
        if self.contains(Status::BREAK) {
            str.push('B');
        } else {
            str.push('b');
        };

        if self.contains(Status::DECIMAL) {
            str.push('D');
        } else {
            str.push('d');
        };
        if self.contains(Status::IDISABLE) {
            str.push('I');
        } else {
            str.push('i');
        };
        if self.contains(Status::CARRY) {
            str.push('C');
        } else {
            str.push('c');
        };
        if self.contains(Status::ZERO) {
            str.push('Z');
        } else {
            str.push('z');
        };
        write!(f, "{}", str)
    }
}

#[test]
fn regreadwrite() {
    Cpu::reset();
    Cpu::write_ac(1);
    Cpu::write_xr(2);
    Cpu::write_yr(3);
    Cpu::write_zr(4);
    Cpu::write_sr(5);
    Cpu::write_sp(6);
    Cpu::write_pc(0x7777);

    assert_eq!(Cpu::read_ac(), 1);
    assert_eq!(Cpu::read_xr(), 2);
    assert_eq!(Cpu::read_yr(), 3);
    assert_eq!(Cpu::read_zr(), 4);
    assert_eq!(Cpu::read_sr(), 5);
    assert_eq!(Cpu::read_sp(), 6);
    assert_eq!(Cpu::read_pc(), 0x7777);
}

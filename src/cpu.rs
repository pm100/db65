use std::os::raw::c_char;
static mut THECPU: Cpu = Cpu {
    ram: [0; 65536],
    regs: std::ptr::null_mut(),
};
struct Cpu {
    ram: [u8; 65536],
    // registers
    regs: *mut CPURegs,
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
    println!("MemWriteByte {:04x} {:02x}", _addr, _val);
    unsafe {
        THECPU.write_memory(_addr as u16, _val);
    }
}

#[no_mangle]
extern "C" fn MemReadWord(_addr: u32) -> u32 {
    println!("MemReadWord {:04x}", _addr);
    unsafe {
        return THECPU.read_memory(_addr as u16) as u32;
    }
}
#[no_mangle]
extern "C" fn MemReadByte(_addr: u32) -> u8 {
    println!("MemReadByte {:04x}", _addr);
    unsafe {
        return THECPU.read_memory(_addr as u16);
    }
}
#[no_mangle]
extern "C" fn MemReadZPWord(mut addr: u8) -> u16 {
    println!("MemReadZPWord");
    unsafe {
        let b1 = THECPU.read_memory(addr as u16) as u16;
        addr = addr.wrapping_add(1);
        let b2 = THECPU.read_memory(addr as u16) as u16;
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

extern "C" fn ParaVirtHooks(regs: *mut CPURegs) {
    println!("ParHook");
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
pub struct Sim {}
impl Sim {
    pub fn reset() {
        unsafe {
            THECPU.regs = ReadRegisters();
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
    pub fn write_memory(addr: u16, val: u8) {
        unsafe {
            THECPU.write_memory(addr, val);
        }
    }
    pub fn read_memory(addr: u16) -> u8 {
        unsafe { THECPU.read_memory(addr) }
    }
}

impl Cpu {
    fn read_memory(&mut self, addr: u16) -> u8 {
        self.ram[addr as usize]
    }
    fn write_memory(&mut self, addr: u16, val: u8) {
        self.ram[addr as usize] = val;
    }
    fn reset(&mut self) {
        unsafe {
            Reset();
        }
    }
    fn execute_insn(&mut self) -> u32 {
        unsafe { ExecuteInsn() }
    }
    fn read_registers(&mut self) -> *mut CPURegs {
        unsafe { ReadRegisters() }
    }
    fn print_registers(&mut self) {
        let regs = self.read_registers();
        unsafe {
            println!("ac: {:02x}", (*regs).ac);
            println!("xr: {:02x}", (*regs).xr);
            println!("yr: {:02x}", (*regs).yr);
            println!("zr: {:02x}", (*regs).zr);
            println!("sr: {:02x}", (*regs).sr);
            println!("sp: {:02x}", (*regs).sp);
            println!("pc: {:02x}", (*regs).pc);
        }
    }
    fn run(sp: u16, start: u16) {}
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

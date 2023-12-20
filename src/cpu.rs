use std::os::raw::c_char;
static mut THECPU: Cpu = Cpu {
    ram: [0; 65536],
    ac: 0,
    xr: 0,
    yr: 0,
    zr: 0,
    sr: 0,
    sp: 0,
    pc: 0,
};
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
            Reset();
        }
    }
    pub fn execute_insn() -> u32 {
        unsafe { ExecuteInsn() }
    }
    pub fn read_registers() {
        unsafe {
            let regs = ReadRegisters();
            THECPU.ac = (*regs).ac as u8;
            THECPU.xr = (*regs).xr as u8;
            THECPU.yr = (*regs).yr as u8;
            THECPU.zr = (*regs).zr as u8;
            THECPU.sr = (*regs).sr as u8;
            THECPU.sp = (*regs).sp as u8;
            THECPU.pc = (*regs).pc as u16;
        }
    }
    pub fn read_ac() -> u8 {
        unsafe { THECPU.ac }
    }
    pub fn read_xr() -> u8 {
        unsafe { THECPU.xr }
    }
    pub fn read_yr() -> u8 {
        unsafe { THECPU.yr }
    }
    pub fn read_zr() -> u8 {
        unsafe { THECPU.zr }
    }
    pub fn read_sr() -> u8 {
        unsafe { THECPU.sr }
    }
    pub fn read_sp() -> u8 {
        unsafe { THECPU.sp }
    }
    pub fn read_pc() -> u16 {
        unsafe { THECPU.pc }
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
    fn new() -> Cpu {
        Cpu {
            ram: [0; 65536],
            ac: 0,
            xr: 0,
            yr: 0,
            zr: 0,
            sr: 0,
            sp: 0,
            pc: 0,
        }
    }
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
}

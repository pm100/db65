use std::{
    collections::HashMap,
    fs::File,
    io::{self, Write},
};

use once_cell::sync::Lazy;

use crate::cpu::Sim;
static PV_FILES: Lazy<HashMap<u16, File>> = Lazy::new(|| {
    println!("initializing");
    HashMap::new()
});
//static PV_FILES: HashMap<u16, File>;
static PV_HOOKS: [fn(); 6] = [
    ParaVirt::pv_open,
    ParaVirt::pv_close,
    ParaVirt::pv_read,
    ParaVirt::pv_write,
    ParaVirt::pv_args,
    ParaVirt::pv_exit,
];
const PARAVIRT_BASE: u16 = 0xFFF4;
pub struct ParaVirt;
impl ParaVirt {
    pub fn pv_init() {}
    fn pop_arg() -> u16 {
        let sp65_addr = Sim::get_sp65_addr();
        let sp65 = Sim::read_word(sp65_addr as u16);
        let val = Sim::read_word(sp65);
        Sim::write_word(sp65_addr as u16, sp65 + 2);
        val
    }
    fn pop() -> u8 {
        //  return MemReadByte (0x0100 + (++Regs->SP & 0xFF));
        let sp = Sim::read_sp();
        let val = Sim::read_byte(0x0100 | sp as u16);
        Sim::write_sp(sp.wrapping_add(1));
        val
    }
    fn set_ax(val: u16) {
        //     Regs->AC = Val & 0xFF;
        //     Val >>= 8;
        //     Regs->XR = Val;
        Sim::write_ac(val as u8);
        Sim::write_xr(((val >> 8) & 0xff) as u8);
    }
    fn get_ax() -> u16 {
        // return Regs->AC + (Regs->XR << 8);load tes
        let ac = Sim::read_ac() as u16;
        let xr = Sim::read_xr() as u16;
        ac | (xr << 8)
    }
    fn pv_open() {
        //     Regs->AC = open ((const char *) (Mem + Regs->XR), Regs->AC);
        //     Regs->XR = errno;
        /*       let addr = ParaVirt::get_ax();
        let name = Sim::read_string(addr);
        let mode = ParaVirt::pop();
        let fd = match mode {
            0 => {
                let f = File::open(name);
                if f.is_err() {
                    ParaVirt::set_ax(0);
                    return;
                }
                let fd = f.unwrap().into_raw_fd();
                fd
            }
            1 => {
                let f = File::create(name);
                if f.is_err() {
                    ParaVirt::set_ax(0);
                    return;
                }
                let fd = f.unwrap().into_raw_fd();
                fd
            }
            2 => {
                let f = OpenOptions::new().append(true).open(name);
                if f.is_err() {
                    ParaVirt::set_ax(0);
                    return;
                }
                let fd = f.unwrap().into_raw_fd();
                fd
            }
            _ => {
                ParaVirt::set_ax(0);
                return;
            }
        };
        ParaVirt::set_ax(fd as u16);
        */
    }
    fn pv_close() {
        //     Regs->AC = close (Regs->AC);
        //     Regs->XR = errno;
        let fd = ParaVirt::pop_arg();
        let res = unsafe { 42 };
        ParaVirt::set_ax(res as u16);
    }
    fn pv_read() {
        //     Regs->AC = read (Regs->AC, Mem + Regs->XR, Regs->YR);
        //     Regs->XR = errno;
        let fd = ParaVirt::pop_arg();
        let addr = ParaVirt::get_ax();
        let count = ParaVirt::pop_arg();
        let mut buf = vec![0; count as usize];
        let res = unsafe {
            // libc::read(
            //     fd as i32,
            //     buf.as_mut_ptr() as *mut libc::c_void,
            //     count as usize,
            // )
            42
        };
        if res < 0 {
            ParaVirt::set_ax(0);
            return;
        }
        for i in 0..res {
            Sim::write_byte(addr + i as u16, buf[i as usize]);
        }
        ParaVirt::set_ax(res as u16);
    }
    fn pv_write() {
        //     Regs->AC = write (Regs->AC, Mem + Regs->XR, Regs->YR);
        //     Regs->XR = errno;
        let count = ParaVirt::get_ax();
        let addr = ParaVirt::pop_arg();
        let fd = ParaVirt::pop_arg();

        let mut buf = vec![0; count as usize];
        for i in 0..count {
            buf[i as usize] = Sim::read_byte(addr + i as u16);
        }
        let res = std::io::stdout().write(&buf).unwrap();

        ParaVirt::set_ax(res as u16);
    }
    fn pv_args() {
        //     Regs->AC = argc;
        //     Regs->XR = argv;
        //     Regs->YR = envp;
        // let argc = Sim::argc();
        // let argv = Sim::argv();
        // let envp = Sim::envp();
        // ParaVirt::set_ax(argc);
        // ParaVirt::set_ax(argv);
        // ParaVirt::set_ax(envp);
    }
    fn pv_exit() {
        //     exit (Regs->AC);
        let code = ParaVirt::pop();
        Sim::set_exit(code);
    }

    pub fn pv_hooks() {
        let pc = Sim::read_pc();
        if pc < PARAVIRT_BASE || pc >= PARAVIRT_BASE + PV_HOOKS.len() as u16 {
            return;
        }

        /* Call paravirtualization hook */
        PV_HOOKS[(pc - PARAVIRT_BASE) as usize]();
        let lo = Self::pop();
        let hi = Self::pop();
        Sim::write_pc(lo as u16 | ((hi as u16) << 8));
    }
}

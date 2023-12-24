use core::panic;
use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{self, Read, Write},
};

use once_cell::sync::Lazy;

use crate::cpu::Sim;
static mut PV_FILES: Lazy<HashMap<u16, File>> = Lazy::new(|| {
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
    fn pop_arg(incr: u16) -> u16 {
        let sp65_addr = Sim::get_sp65_addr();
        let sp65 = Sim::read_word(sp65_addr as u16);
        let val = Sim::read_word(sp65);
        Sim::write_word(sp65_addr as u16, sp65 + incr);
        val
    }
    fn pop() -> u8 {
        //  return MemReadByte (0x0100 + (++Regs->SP & 0xFF));
        let sp = Sim::read_sp();
        let newsp = sp.wrapping_add(1);
        let val = Sim::read_byte(0x0100 | newsp as u16);
        Sim::write_sp(newsp);
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
        let mut mode = Self::pop_arg(Sim::read_yr() as u16 - 4);
        let flags = Self::pop_arg(2);
        let mut name = Self::pop_arg(2);
        if (Sim::read_yr() - 4 < 2) {
            /* If the caller didn't supply the mode
             ** argument, use a reasonable default.
             */
            mode = 0x01 | 0x02;
        }
        let mut name_buf = Vec::new();

        loop {
            let c = Sim::read_byte(name);
            if c == 0 {
                break;
            }
            name_buf.push(c);
            name += 1;
        }
        let name_str = String::from_utf8(name_buf).unwrap();
        let mut opt = OpenOptions::new();
        match flags & 0x03 {
            0x01 => opt.read(true),

            0x02 => opt.write(true),

            0x03 => opt.read(true).write(true),
            _ => panic!("invalid flags"),
        };
        if (flags & 0x10) != 0 {
            opt.create(true);
        }
        if (flags & 0x20) != 0 {
            opt.truncate(true);
        }
        if (flags & 0x40) != 0 {
            opt.append(true);
        }
        if (flags & 0x80) != 0 {
            opt.create_new(true);
        }
        if let Ok(fd) = opt.open(name_str) {
            unsafe {
                let fno = PV_FILES.len() as u16 + 3;
                PV_FILES.insert(fno, fd);
                Self::set_ax(fno);
            }
        } else {
            Self::set_ax(0xffff);
        }
    }
    fn pv_close() {
        //     Regs->AC = close (Regs->AC);
        //     Regs->XR = errno;
        let fd = ParaVirt::pop_arg(2);

        let res = unsafe {
            if let Some(file) = PV_FILES.get(&fd) {
                PV_FILES.remove(&fd);

                0
            } else {
                -1
            }
        };
        Self::set_ax(res as u16);
    }
    fn pv_read() {
        let addr = ParaVirt::pop_arg(2);
        let fd = ParaVirt::pop_arg(2);

        let count = ParaVirt::get_ax();
        let mut buf = vec![0; count as usize];
        let res = if fd == 0 {
            std::io::stdin().read(&mut buf).unwrap() as u16
        } else {
            unsafe {
                if let Some(mut file) = PV_FILES.get(&fd) {
                    file.read(&mut buf).unwrap() as u16
                } else {
                    Self::set_ax(0xffff as u16);
                    return;
                }
            }
        };

        for i in 0..res {
            Sim::write_byte(addr + i as u16, buf[i as usize]);
        }
        Self::set_ax(res as u16);
    }
    fn pv_write() {
        //     Regs->AC = write (Regs->AC, Mem + Regs->XR, Regs->YR);
        //     Regs->XR = errno;
        let count = ParaVirt::get_ax();
        let addr = ParaVirt::pop_arg(2);
        let fd = ParaVirt::pop_arg(2);

        let mut buf = vec![0; count as usize];
        for i in 0..count {
            buf[i as usize] = Sim::read_byte(addr + i as u16);
        }
        let res = match fd {
            1 => std::io::stdout().write(&buf).unwrap(),
            2 => std::io::stderr().write(&buf).unwrap(),
            _ => unsafe {
                let mut file = PV_FILES.get(&fd).unwrap();
                file.write(&buf).unwrap()
            },
        };
        //Sim::set_exit(0);
        Self::set_ax(res as u16);
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
        Sim::write_pc((lo as u16 | ((hi as u16) << 8)) + 1);
    }
}

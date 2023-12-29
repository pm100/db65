use core::panic;
use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{stderr, stdout, Read, Write},
};

use once_cell::sync::Lazy;

use crate::cpu::Cpu;
static mut PV_FILES: Lazy<HashMap<u16, File>> = Lazy::new(|| HashMap::new());

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
    pub fn _pv_init() {}
    fn pop_arg(incr: u16) -> u16 {
        let sp65_addr = Cpu::get_sp65_addr();
        let sp65 = Cpu::read_word(sp65_addr as u16);
        let val = Cpu::read_word(sp65);
        Cpu::write_word(sp65_addr as u16, sp65 + incr);
        val
    }
    fn pop() -> u8 {
        let sp = Cpu::read_sp();
        let newsp = sp.wrapping_add(1);
        let val = Cpu::read_byte(0x0100 | newsp as u16);
        Cpu::write_sp(newsp);
        val
    }
    fn set_ax(val: u16) {
        Cpu::write_ac(val as u8);
        Cpu::write_xr(((val >> 8) & 0xff) as u8);
    }
    fn get_ax() -> u16 {
        let ac = Cpu::read_ac() as u16;
        let xr = Cpu::read_xr() as u16;
        ac | (xr << 8)
    }
    fn pv_open() {
        let mut mode = Self::pop_arg(Cpu::read_yr() as u16 - 4);
        let flags = Self::pop_arg(2);
        let mut name = Self::pop_arg(2);
        if Cpu::read_yr() - 4 < 2 {
            /* If the caller didn't supply the mode
             ** argument, use a reasonable default.
             */
            mode = 0x01 | 0x02;
        }
        let mut name_buf = Vec::new();

        loop {
            let c = Cpu::read_byte(name);
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
            if let Some(_file) = PV_FILES.get(&fd) {
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
            Cpu::write_byte(addr + i as u16, buf[i as usize]);
        }
        Self::set_ax(res as u16);
    }
    fn pv_write() {
        let count = ParaVirt::get_ax();
        let addr = ParaVirt::pop_arg(2);
        let fd = ParaVirt::pop_arg(2);

        let mut buf = vec![0; count as usize];
        for i in 0..count {
            buf[i as usize] = Cpu::read_byte(addr + i as u16);
        }
        let res = match fd {
            1 => {
                let count = stdout().write(&buf).unwrap();
                stdout().flush().unwrap();
                count
            }
            2 => {
                let count = stderr().write(&buf).unwrap();
                stderr().flush().unwrap();
                count
            }

            _ => unsafe {
                let mut file = PV_FILES.get(&fd).unwrap();
                file.write(&buf).unwrap()
            },
        };

        Self::set_ax(res as u16);
    }
    fn pv_args() {
        // where the caller wants the pointer to arg array
        let caller_arg_addr = Self::get_ax();
        let sp65_addr = Cpu::get_sp65_addr() as u16;
        let mut sp65 = Cpu::read_word(sp65_addr);
        let argcount = Cpu::get_arg_count() as u16;
        // points to array of pointers to argv[n]
        let mut arg_ptr_storage = sp65 - ((Cpu::get_arg_count() + 1) * 2) as u16;
        // store that address of argv table where caller asked for it
        Cpu::write_word(caller_arg_addr, arg_ptr_storage);
        sp65 = arg_ptr_storage;
        // copy the host os arguments contents over
        // sp65 is decremented for each one
        for i in 0..Cpu::get_arg_count() {
            let current_arg = Cpu::get_arg(i);
            let arg_len = current_arg.len() as u16;
            sp65 -= arg_len + 1;
            let bytes = current_arg.as_bytes();
            for j in 0..arg_len {
                Cpu::write_byte(sp65 + j, bytes[j as usize]);
            }
            Cpu::write_byte(sp65 + arg_len, 0);
            Cpu::write_word(arg_ptr_storage, sp65);
            arg_ptr_storage += 2;
        }

        Cpu::write_word(arg_ptr_storage, sp65_addr);

        Cpu::write_word(sp65_addr, sp65);
        Self::set_ax(argcount);
    }
    fn pv_exit() {
        let code = ParaVirt::pop();
        Cpu::set_exit(code);
    }

    pub fn pv_hooks() {
        let pc = Cpu::read_pc();
        if pc < PARAVIRT_BASE || pc >= PARAVIRT_BASE + PV_HOOKS.len() as u16 {
            return;
        }

        /* Call paravirtualization hook */
        PV_HOOKS[(pc - PARAVIRT_BASE) as usize]();
        let lo = Self::pop();
        let hi = Self::pop();
        Cpu::write_pc((lo as u16 | ((hi as u16) << 8)) + 1);
    }
}

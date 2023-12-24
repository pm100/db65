use anyhow::Result;
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use crate::{cpu::Sim, execute::StopReason, loader};
pub struct Debugger {
    symbols: HashMap<String, u16>,
    pub break_points: HashMap<u16, BreakPoint>,
    loader_sp: u8,
    loader_start: u16,
}
pub struct BreakPoint {
    addr: u16,
    symbol: String,
    number: u16,
}
impl Debugger {
    pub fn new() -> Self {
        Sim::reset();
        Self {
            symbols: HashMap::new(),
            break_points: HashMap::new(),
            loader_sp: 0,
            loader_start: 0,
        }
    }
    pub fn set_break(&mut self, addr_str: &str) -> Result<()> {
        let mut bp_addr;
        let sym = self.symbols.get(addr_str);
        let mut save_sym = String::new();
        if sym.is_some() {
            save_sym = addr_str.to_string();
            bp_addr = *sym.unwrap();
        } else {
            if addr_str.chars().next().unwrap() == '$' {
                let rest = addr_str[1..].to_string();
                bp_addr = u16::from_str_radix(&rest, 16)?;
            } else {
                bp_addr = u16::from_str_radix(addr_str, 16)?;
            }
        }
        self.break_points.insert(
            bp_addr,
            BreakPoint {
                addr: bp_addr,
                symbol: save_sym,
                number: 42,
            },
        );
        Ok(())
    }
    pub fn load_ll(&mut self, file: &Path) -> Result<()> {
        let f = File::open(file)?;
        //let re = Regex::new("a")
        let mut reader = BufReader::new(f);
        loop {
            let mut line = String::new();
            match reader.read_line(&mut line)? {
                0 => break,
                _len => {
                    //al 000000 .sp
                    let mut spl = line.split(" ");
                    let _al = spl.next();
                    let addr_str = spl.next().unwrap().trim_end();
                    let mut name = spl.next().unwrap().trim_end();
                    let addr = u16::from_str_radix(addr_str, 16).unwrap();
                    println!("sym {} = {:04x}", name, addr);
                    self.symbols.insert(name.to_string(), addr);
                }
            }
        }
        Ok(())
    }
    pub fn load_code(&mut self, file: &Path) -> Result<()> {
        let (sp65_addr, run, cpu, size) = loader::load_code(file)?;
        println!("size={:x}, entry={:x}, cpu={}", size, run, cpu);
        Sim::sp65_addr(sp65_addr);

        self.loader_start = run;

        Ok(())
    }
    pub fn get_breaks(&self) -> Vec<u16> {
        self.break_points.iter().map(|bp| bp.1.addr).collect()
    }
    pub fn go(&mut self) -> Result<()> {
        self.core_run()?;
        Ok(())
    }
    fn state(&self) {
        println!("pc={:04x}", Sim::read_pc());
    }
    pub fn next(&mut self) -> Result<()> {
        let reason = self.execute(1)?;
        match reason {
            StopReason::BreakPoint => {
                println!("breakpoint");
            }
            StopReason::Exit => {
                println!("exit");
            }
            StopReason::Count => {
                println!("count");
            }
        }
        self.state();
        Ok(())
    }
    fn core_run(&mut self) -> Result<()> {
        let reason = self.execute(0)?; // 0 = forever
        match reason {
            StopReason::BreakPoint => {
                println!("breakpoint");
            }
            StopReason::Exit => {
                println!("exit");
            }
            StopReason::Count => {
                println!("count");
            }
        }
        self.state();
        Ok(())
    }
    pub fn run(&mut self) -> Result<()> {
        //  Sim::write_sp(self.loader_sp);
        Sim::write_word(0xFFFC, self.loader_start);
        Sim::reset();
        self.core_run()?;
        Ok(())
    }
}

use anyhow::Result;
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};
pub struct Debugger {
    symbols: HashMap<String, u16>,
    break_points: HashMap<u16, BreakPoint>,
}
pub struct BreakPoint {
    addr: u16,
    symbol: String,
    number: u16,
}
impl Debugger {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            break_points: HashMap::new(),
        }
    }
    pub fn set_break(&mut self, addr_str: &str) {
        let mut bp_addr;
        let sym = self.symbols.get(addr_str);
        let mut save_sym = String::new();
        if sym.is_some() {
            save_sym = addr_str.to_string();
            bp_addr = *sym.unwrap();
        } else {
            if addr_str.chars().next().unwrap() == '$' {
                let rest = addr_str[1..].to_string();
                bp_addr = u16::from_str_radix(&rest, 16).unwrap();
            } else {
                bp_addr = u16::from_str_radix(addr_str, 16).unwrap();
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
    pub fn get_breaks(&self) -> Vec<u16> {
        self.break_points.iter().map(|bp| bp.1.addr).collect()
    }
}

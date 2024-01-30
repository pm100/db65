/*
    reads binary and loads it into RAM
*/

use anyhow::{bail, Result};
use std::fs::File;
use std::io::{BufReader, Bytes, Read};
use std::path::Path;

use crate::debugger::cpu::Cpu;
use crate::say;


static HEADER: &[u8] = &[0x73, 0x69, 0x6D, 0x36, 0x35];
pub fn load_code(file: &Path) -> Result<(u8, u16, u8, u16)> {
    let f = File::open(file)?;
    let reader = BufReader::new(f);
    let mut bytes = reader.bytes();

    for hb in HEADER {
        let b = bytes.next().unwrap()?;
        if b != *hb {
            bail!("invalid header");
        }
    }

    let b = bytes.next().unwrap()?;
    if b != 2 {
        bail!("invalid header");
    }
    let cpu = bytes.next().unwrap()?;
    if cpu != 0 && cpu != 1 {
        bail!("invalid header");
    }
    Cpu::set_cpu(cpu as u16);
    // sp65_addr is the location of the cc65 'stack pointer'
    // not to be confused with the 6502 sp

    let sp65_addr = bytes.next().unwrap()?;
    let mut load = get_u16(&mut bytes)?;
    let run = get_u16(&mut bytes)?;
    let mut count = 0;
    loop {
        let b = bytes.next();
        if b.is_none() {
            break;
        }

        Cpu::write_byte(load, b.unwrap()?);

        load += 1;
        count += 1;
    }
    let cpu_str = if cpu == 0 { "6502" } else { "65c02" };
    say!("Loaded {} bytes, cpu={}", count, cpu_str);
    Ok((sp65_addr, run, cpu, count))
}
fn get_u16(bytes: &mut Bytes<BufReader<File>>) -> Result<u16> {
    let b1 = bytes.next().unwrap()? as u16;
    let b2 = bytes.next().unwrap()? as u16;
    Ok(b1 | (b2 << 8))
}

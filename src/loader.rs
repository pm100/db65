use anyhow::{bail, Result};
use std::fs::File;
use std::io::{BufReader, Bytes, Read};

use crate::cpu::Sim;
static HEADER: &'static [u8] = &[0x73, 0x69, 0x6D, 0x36, 0x35];
pub fn load_code() -> Result<(u8, u16, u8)> {
    let f = File::open("asmtest")?;
    let reader = BufReader::new(f);
    let mut bytes = reader.bytes();

    for i in 0..5 {
        let b = bytes.next().unwrap()?;
        if b != HEADER[i] {
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
    let sp = bytes.next().unwrap()?;
    let mut load = get_u16(&mut bytes)?;
    let run = get_u16(&mut bytes)?;

    loop {
        let b = bytes.next();
        if b.is_none() {
            break;
        }
        Sim::write_byte(load, b.unwrap()?);
        load += 1;
    }

    Ok((sp, run, cpu))
}
fn get_u16(bytes: &mut Bytes<BufReader<File>>) -> Result<u16> {
    let b1 = bytes.next().unwrap()? as u16;
    let b2 = bytes.next().unwrap()? as u16;
    Ok(b1 | (b2 << 8))
}

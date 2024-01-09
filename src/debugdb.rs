use anyhow::{anyhow, bail, Result};
//pub const NO_PARAMS:  = [];
use rusqlite::{params, Connection, Result as SqlResult};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::BufReader,
};

pub struct DebugData {
    pub conn: Connection,
}
impl DebugData {
    pub fn new() -> Result<DebugData> {
        let dbfile = "dbg.db";
        if fs::metadata(dbfile).is_ok() {
            std::fs::remove_file("dbg.db")?;
        }
        let mut ret = Self {
            conn: Connection::open("dbg.db")?,
        };
        ret.create_tables()?;
        Ok(ret)
    }

    pub fn get_symbols(&self, filter: Option<&String>) -> Result<Vec<(String, u16)>> {
        let mut v = Vec::new();
        let mut stmt = self
            .conn
            .prepare("select symdef.name, symdef.val from symdef")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<usize, String>(0)?, row.get::<usize, i64>(1)?))
        })?;
        for row in rows {
            let (name, val) = row?;
            if let Some(filter) = filter {
                if name.contains(filter) {
                    v.push((name, val as u16));
                }
            } else {
                v.push((name, u16::try_from(val).unwrap()));
            }
        }
        Ok(v)
    }

    pub fn get_symbol(&self, name: &str) -> Result<Option<u16>> {
        let mut stmt = self
            .conn
            .prepare("select symdef.val from symdef where symdef.name = ?1")?;
        let mut rows = stmt.query(params![name])?;
        if let Some(row) = rows.next()? {
            let val: i64 = row.get(0)?;
            return Ok(Some(val as u16));
        }
        Ok(None)
    }

    pub fn find_symbol(&self, addr: u16) -> Result<Option<String>> {
        let mut stmt = self
            .conn
            .prepare("select symdef.name from symdef where symdef.val = ?1 and seg not null")?;
        let mut rows = stmt.query(params![addr])?;
        if let Some(row) = rows.next()? {
            let val: String = row.get(0)?;
            return Ok(Some(val));
        }
        Ok(None)
    }
}

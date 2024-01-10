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

    pub fn get_symbols(&self, filter: Option<&String>) -> Result<Vec<(String, u16, String)>> {
        let mut v = Vec::new();
        let mut stmt = self
            .conn
            .prepare_cached("select name, val , module from symbol")?;
        let rows = stmt.query_map([], |row| {
            let name = row.get::<usize, String>(0)?;
            let val = row.get::<usize, i64>(1)? as u16;
            let mut module = row.get::<usize, String>(2)?;
            module = module.strip_suffix(".o").unwrap_or(&module).to_string();
            Ok((name, val, module))
        })?;
        for row in rows {
            let (name, val, module) = row?;
            if let Some(filter) = filter {
                if !name.contains(filter) {
                    continue;
                }
            }
            v.push((name, val, module));
        }
        Ok(v)
    }
    pub fn get_symbol(&self, name: &str) -> Result<Vec<(String, u16, String)>> {
        let mut v = Vec::new();

        let (module, name) = if name.contains('.') {
            let mut split = name.split('.');
            (
                split.next().unwrap().to_string(),
                split.next().unwrap().to_string(),
            )
        } else {
            (String::new(), name.to_string())
        };
        let mut stmt = self
            .conn
            .prepare_cached("select name, val , module from symbol where name = ?1")?;
        let rows = stmt.query_map([name], |row| {
            let name = row.get::<usize, String>(0)?;
            let val = row.get::<usize, i64>(1)? as u16;
            let mut module = row.get::<usize, String>(2)?;
            module = module.strip_suffix(".o").unwrap_or(&module).to_string();
            Ok((name, val, module))
        })?;
        for row in rows {
            let (name, val, m) = row?;
            if module.len() > 0 && module != m {
                continue;
            }
            v.push((name, val, m));
        }
        Ok(v)
    }
    pub fn get_symbolx(&self, name: &str) -> Result<Option<u16>> {
        let mut stmt = self
            .conn
            .prepare_cached("select symdef.val from symdef where symdef.name = ?1")?;
        let mut rows = stmt.query(params![name])?;
        if let Some(row) = rows.next()? {
            let val: i64 = row.get(0)?;
            return Ok(Some(val as u16));
        }
        Ok(None)
    }

    pub fn find_symbol(&self, addr: u16) -> Result<Option<String>> {
        let mut stmt = self.conn.prepare_cached(
            "select symdef.name from symdef where symdef.val = ?1 and seg not null",
        )?;
        let mut rows = stmt.query(params![addr])?;
        if let Some(row) = rows.next()? {
            let val: String = row.get(0)?;
            return Ok(Some(val));
        }
        Ok(None)
    }
}

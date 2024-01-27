use anyhow::{anyhow, bail, Result};
use evalexpr::Value;
//pub const NO_PARAMS:  = [];
use crate::db::util::Extract;
use crate::debugger::debugger::{HLSym, SegChunk, Segment, Symbol, SymbolType};
use crate::log::say;
use rusqlite::{
    params,
    types::{Null, Value as SqlValue},
    Connection,
};
use std::{
    collections::{BTreeMap, HashMap},
    fs::{self, File},
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};
#[derive(Debug)]
pub struct SourceInfo {
    pub file_id: i64,
    pub line: String,
    pub line_no: i64,
    pub seg: u8,
    pub addr: u16,
    pub absaddr: u16,
}
#[derive(Debug)]

pub struct SourceFile {
    pub file_id: i64,
    pub short_name: String,
    pub full_path: PathBuf,
    pub loaded: bool,
}
pub struct DebugData {
    pub conn: Connection,
    name: String,
    pub cc65_dir: Option<PathBuf>,
}

impl DebugData {
    pub fn new(name: &str) -> Result<DebugData> {
        if fs::metadata(name).is_ok() {
            std::fs::remove_file(name)?;
        }
        let mut ret = Self {
            conn: Connection::open(name)?,
            cc65_dir: None,
            name: name.to_string(),
        };
        ret.create_tables()?;
        Ok(ret)
    }

    pub fn clear(&mut self) -> Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute("delete from symdef", [])?;
        tx.execute("delete from symref", [])?;
        tx.execute("delete from line", [])?;
        tx.execute("delete from file", [])?;
        tx.execute("delete from source", [])?;
        tx.execute("delete from source_line", [])?;
        tx.execute("delete from segment", [])?;
        tx.execute("delete from span", [])?;
        tx.execute("delete from scope", [])?;
        tx.execute("delete from csymbol", [])?;
        tx.execute("delete from module", [])?;
        tx.commit()?;

        Ok(())
    }
    fn convert_symbol_type(s: &str) -> SymbolType {
        match s {
            "lab" => SymbolType::Label,
            "equ" => SymbolType::Equate,
            "c" => SymbolType::CSymbol,
            _ => {
                //  unreachable!("unknown symbol type");
                //  bail!("unknown symbol type");
                SymbolType::Unknown
            } //SymbolType::Unknown,
        }
    }
    pub fn get_symbols(&self, filter: Option<&String>) -> Result<Vec<Symbol>> {
        let mut v = Vec::new();
        let mut stmt = self
            .conn
            .prepare_cached("select name, val , module, type from symbol")?;
        let rows = stmt.query_map([], |row| {
            let name = row.get::<usize, String>(0)?;
            let val = row.get::<usize, i64>(1)? as u16;
            let module = row.get::<usize, Option<String>>(2)?;
            let sym_type = Self::convert_symbol_type(row.get::<usize, String>(3)?.as_str());

            Ok((name, val, module, sym_type))
        })?;
        for row in rows {
            let (name, value, module, sym_type) = row?;
            if let Some(filter) = filter {
                if !name.contains(filter) {
                    continue;
                }
            }
            // linker defined symbols have no module names assocaited with them
            let module = if let Some(m) = module {
                m
            } else {
                String::new()
            };
            v.push(Symbol {
                name,
                value,
                module,
                sym_type,
            });
        }
        Ok(v)
    }
    pub fn load_expr_symbols(&mut self, sym_tab: &mut HashMap<String, Value>) -> Result<()> {
        let mut stmt = self
            .conn
            .prepare_cached("select name, val , module from symbol")?;
        let rows = stmt.query_map([], |row| {
            let name = row.get::<usize, String>(0)?;
            let val = row.get::<usize, i64>(1)? as u16;

            let module = row.get::<usize, Option<String>>(2)?;

            Ok((name, val, module))
        })?;
        sym_tab.clear();
        for row in rows {
            let (name, val, _module) = row?;
            sym_tab.insert(name, Value::Int(val as i64));
        }

        Ok(())
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
            let module = row.get::<usize, Option<String>>(2)?;
            let m = if let Some(m) = module {
                m.strip_suffix(".o").unwrap_or(&m).to_string()
            } else {
                String::new()
            };

            Ok((name, val, m))
        })?;
        for row in rows {
            let (name, val, m) = row?;
            if !module.is_empty() && module != m {
                continue;
            }
            v.push((name, val, m));
        }
        Ok(v)
    }

    pub fn load_files(&mut self, file_table: &mut HashMap<i64, SourceFile>) -> Result<()> {
        let rows = self.query_db(&[], "select name,id from file")?;

        for row in rows {
            let name = row[0].vto_string()?;
            let id = row[1].vto_i64()?;
            let path = Path::new(&name);
            if let Some(p) = self.find_file(path)? {
                let sf = SourceFile {
                    file_id: id,
                    short_name: p.file_name().unwrap().to_str().unwrap().to_string(),
                    full_path: p.to_path_buf(),
                    loaded: false,
                };
                file_table.insert(id, sf);
                // println!("found file {}", p.display());
            } else {
                say(&format!("can't find file {}", name));
            }
        }
        Ok(())
    }

    pub fn find_symbol_by_addr(&self, addr: u16) -> Result<Vec<Symbol>> {
        let ans = self.query_db(
            params![addr],
            "select name, scope,type,val,seg from symdef where symdef.val = ?1 and seg not null",
        )?;
        let mut v = Vec::new();
        for row in ans {
            let name = row[0].vto_string()?;
            // let addr = row[1].vto_i64()?;
            //  let module = row[2].vto_string()?;
            let scope = row[1].vto_i64()?;
            let type_ = row[2].vto_string()?;
            let val = row[3].vto_i64()? as u16;
            let seg = row[4].vto_i64()? as u8;
            v.push(Symbol {
                name,
                value: val,
                module: String::new(),
                sym_type: Self::convert_symbol_type(&type_),
            });
        }
        Ok(v)
    }
    pub fn set_cc65_dir(&mut self, dir: &Path) -> Result<()> {
        self.cc65_dir = Some(dir.to_path_buf());
        Ok(())
    }
    pub fn load_all_cfiles(&mut self) -> Result<()> {
        let rows = self.query_db(
            &[],
            "select name,file.id from file,cline where cline.file = file.id group by name",
        )?;
        if self.cc65_dir.is_none() {
            self.cc65_dir = self.guess_cc65_dir()?;
        }
        for row in rows {
            if let SqlValue::Text(name) = &row[0] {
                let path = Path::new(&name);

                self.load_source_file(path)?;
            }
        }
        Ok(())
    }

    pub fn load_source_file(&mut self, file: &Path) -> Result<()> {
        println!("load source file {}", file.display());

        let full_path = if let Some(p) = self.find_file(file)? {
            p
        } else {
            say(&format!("can't find file {}", file.display()));
            return Ok(());
        };

        let fd = File::open(full_path)?;
        let mut reader = BufReader::new(fd);
        let mut line = String::new();
        let mut lineno = 0;
        let file_name = file.file_name().ok_or(anyhow!("bad file name"))?.to_str();

        let row = self
            .conn
            .prepare_cached("select id from file where name = ?1")?
            .query_row(params![file.to_str().unwrap()], |row| {
                row.get::<usize, i64>(0)
            })?;

        self.conn.execute(
            "insert into source (file_id,name) values(?1, ?2)",
            params![row, file_name],
        )?;
        let mut map = BTreeMap::new();
        self.get_source_file_lines(row, &mut map)?;
        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare_cached(
            "insert into source_line (file, line_no, line, seg,addr,absaddr) values(?1, ?2, ?3, ?4,?5, ?6)",
        )?;
            loop {
                line.clear();
                let len = reader.read_line(&mut line)?;
                if len == 0 {
                    break;
                }
                lineno += 1;
                if let Some(inf) = map.get(&lineno) {
                    stmt.execute(params![
                        row,
                        lineno,
                        line.trim_end(),
                        inf.seg,
                        inf.addr,
                        inf.absaddr
                    ])?;
                } else {
                    stmt.execute(params![row, lineno, line.trim_end(), Null, Null, Null])?;
                }
            }
        }
        tx.commit()?;
        Ok(())
    }
    pub fn load_all_source_files(&mut self, map: &mut BTreeMap<u16, SourceInfo>) -> Result<()> {
        let rows = self.query_db(&[], "select distinct file from cline")?;

        for row in rows {
            if let SqlValue::Integer(id) = &row[0] {
                self.get_source_file_lines2(*id, map)?;
            }
        }
        Ok(())
    }
    fn get_source_file_lines(&self, file: i64, hash: &mut BTreeMap<i64, SourceInfo>) -> Result<()> {
        let mut stmt = self.conn.prepare_cached(
            "select line,seg,addr, (cline.addr+ segment.start) as absaddr
             from  cline, segment   where cline.file = ?1 and cline.seg = segment.id",
        )?;
        let rows = stmt.query_map(params![file], |row| {
            let line_no = row.get::<usize, i64>(0)?;
            let seg = row.get::<usize, i64>(1)?;
            let addr = row.get::<usize, u16>(2)?;
            let absaddr = row.get::<usize, u16>(3)?;
            Ok(SourceInfo {
                line_no,
                seg: seg as u8,
                addr,
                absaddr,
                line: String::new(),
                file_id: file,
            })
        })?;
        for row in rows {
            let info = row?;
            hash.insert(info.line_no, info);
        }
        Ok(())
    }
    pub fn get_source(&self, file: i64, from: i64, to: i64) -> Result<Vec<String>> {
        let sql =
    "select line  from source_line where file = ?1 and line_no >= ?2 and line_no <= ?3 order by line_no";
        let rows = self.query_db(params![file, from, to], sql)?;
        let r = rows
            .iter()
            .map(|row| row[0].vto_string().unwrap())
            .collect();
        Ok(r)
    }

    fn get_source_file_lines2(
        &self,
        file: i64,
        hash: &mut BTreeMap<u16, SourceInfo>,
    ) -> Result<()> {
        let mut stmt = self.conn.prepare_cached(
            "select line,seg,addr, (cline.addr+ segment.start) as absaddr
             from  cline, segment   where cline.file = ?1 and cline.seg = segment.id",
        )?;
        let rows = stmt.query_map(params![file], |row| {
            let line_no = row.get::<usize, i64>(0)?;
            let seg = row.get::<usize, i64>(1)?;
            let addr = row.get::<usize, u16>(2)?;
            let absaddr = row.get::<usize, u16>(3)?;
            Ok(SourceInfo {
                line_no,
                seg: seg as u8,
                addr,
                absaddr,
                line: String::new(),
                file_id: file,
            })
        })?;
        for row in rows {
            let info = row?;
            hash.insert(info.absaddr, info);
        }
        Ok(())
    }
    pub fn find_source_line_by_line_no(
        &self,
        file: i64,
        line_no: i64,
    ) -> Result<Option<SourceInfo>> {
        let sql = "select * from (select * from source_line where file=?1  and absaddr not null order by line_no asc) where line_no
        >= ?2 limit 1;";
        let mut stmt = self.conn.prepare_cached(sql)?;
        match stmt.query_row(params![file, line_no], |row| {
            let file = row.get::<usize, i64>(1)?;
            let line = row.get::<usize, String>(2)?;
            let line_no = row.get::<usize, i64>(3)?;
            let seg = row.get::<usize, i64>(4)?;
            let addr = row.get::<usize, u16>(5)?;
            let absaddr = row.get::<usize, u16>(6)?;
            Ok(SourceInfo {
                line,
                line_no,
                file_id: file,
                seg: seg as u8,
                addr,
                absaddr,
            })
        }) {
            Ok(info) => Ok(Some(info)),
            Err(e) => match e {
                rusqlite::Error::QueryReturnedNoRows => Ok(None),
                _ => Err(e.into()),
            },
        }
    }
    pub fn find_source_line(&self, addr: u16) -> Result<Option<SourceInfo>> {
        let sql =
            "select * from (select * from source_line order by absaddr desc) where absaddr <= ?1 limit 1";
        let mut stmt = self.conn.prepare_cached(sql)?;
        match stmt.query_row(params![addr], |row| {
            let file = row.get::<usize, i64>(1)?;
            let line = row.get::<usize, String>(2)?;
            let line_no = row.get::<usize, i64>(3)?;
            let seg = row.get::<usize, i64>(4)?;
            let addr = row.get::<usize, u16>(5)?;
            let absaddr = row.get::<usize, u16>(6)?;
            Ok(SourceInfo {
                line,
                line_no,
                file_id: file,
                seg: seg as u8,
                addr,
                absaddr,
            })
        }) {
            Ok(info) => Ok(Some(info)),
            Err(e) => match e {
                rusqlite::Error::QueryReturnedNoRows => Ok(None),
                _ => Err(e.into()),
            },
        }
    }
    pub fn load_seg_list(&mut self, seg_list: &mut Vec<Segment>) -> Result<()> {
        for seg in self
            .conn
            .prepare("select * from segment")?
            .query_map(params![], |row| {
                let seg = row.get::<usize, i64>(0)?;
                let name = row.get::<usize, String>(1)?;
                let start = row.get::<usize, u16>(2)?;
                let size = row.get::<usize, u16>(3)?;
                let seg_type = row.get::<usize, u8>(5)?;

                Ok(Segment {
                    id: seg as u8,
                    name,
                    start,
                    size,
                    modules: Vec::new(),
                    seg_type,
                })
            })?
        {
            seg_list.push(seg?);
        }
        for chunk in self
            .conn
            .prepare(
                "select module.name, module.id, line.file, seg,min(start),sum(span.size)
                 from span,line,module,file
                  where file.id = line.file and span.aline = line.id and module.file = file.id
                     group by line.file,seg",
            )?
            .query_map(params![], |row| {
                let name = row.get::<usize, String>(0)?;
                let id = row.get::<usize, i64>(1)?;
                let file = row.get::<usize, i64>(2)?;
                let seg = row.get::<usize, i64>(3)?;
                let start = row.get::<usize, u16>(4)?;
                let size = row.get::<usize, u16>(5)?;
                Ok((name, id, file, seg, start, size))
            })?
        {
            let (name, id, _file, segid, start, size) = chunk?;
            if let Some(seg) = seg_list.iter_mut().find(|s| s.id == segid as u8) {
                seg.modules.push(SegChunk {
                    offset: start,
                    module: id as i32,
                    module_name: name,
                    size: size,
                });
            } else {
                bail!("bad segid {}", segid);
            }
        }
        Ok(())
    }
    pub fn find_assembly_line(&self, addr: u16) -> Result<Option<SourceInfo>> {
        let sql = "select * from
            (select file,line,seg,addr, (aline.addr+ segment.start) as absaddr
            from  aline, segment where segment.id = aline.seg order by absaddr desc)
            where absaddr <= ?1 limit 1";

        self.internal_find_line(addr, sql)
    }
    fn internal_find_line(&self, addr: u16, sql: &str) -> Result<Option<SourceInfo>> {
        let mut stmt = self.conn.prepare_cached(sql)?;
        match stmt.query_row(params![addr], |row| {
            let file = row.get::<usize, i64>(0)?;
            let line_no = row.get::<usize, i64>(1)?;
            let seg = row.get::<usize, u8>(2)?;
            let addr = row.get::<usize, u16>(3)?;
            let absaddr = row.get::<usize, u16>(4)?;
            Ok(SourceInfo {
                line: String::new(),
                line_no,
                file_id: file,
                seg,
                addr,
                absaddr,
            })
        }) {
            Ok(info) => Ok(Some(info)),
            Err(e) => match e {
                rusqlite::Error::QueryReturnedNoRows => Ok(None),
                _ => Err(e.into()),
            },
        }
    }
    pub fn get_file_line(&self, file: i64, line_no: i64) -> Result<Option<String>> {
        let sql = "select line from source_line where file = ?1 and line_no = ?2";
        let mut stmt = self.conn.prepare_cached(sql)?;
        match stmt.query_row(params![file, line_no], |row| {
            let line = row.get::<usize, String>(0)?;
            Ok(line)
        }) {
            Ok(info) => Ok(Some(info)),
            Err(e) => match e {
                rusqlite::Error::QueryReturnedNoRows => Ok(None),
                _ => Err(e.into()),
            },
        }
    }
    pub fn find_c_line(&self, addr: u16) -> Result<Option<SourceInfo>> {
        let sql = "select * from
            (select file,line,seg,addr, (cline.addr+ segment.start) as absaddr
            from  cline, segment where segment.id = cline.seg order by absaddr desc)
            where absaddr <= ?1 limit 1";
        let mut si = self.internal_find_line(addr, sql)?;
        if let Some(ref mut cline) = &mut si {
            if let Some(l) = self.get_file_line(cline.file_id, cline.line_no)? {
                cline.line = l;
            }
        }
        Ok(si)
    }

    pub fn find_scope(&self, seg: i64, addr: u16) -> Result<Option<i64>> {
        let sql = "select span.scope from span  where span.scope not null and span.seg=?1 and span.start <= ?2 and span.start + span.size > ?2";
        let mut stmt = self.conn.prepare_cached(sql)?;
        match stmt.query_row(params![seg, addr], |row| {
            let id = row.get::<usize, i64>(0)?;
            Ok(id)
        }) {
            Ok(id) => Ok(Some(id)),
            Err(e) => match e {
                rusqlite::Error::QueryReturnedNoRows => Ok(None),
                _ => Err(e.into()),
            },
        }
    }
    pub fn find_csym(&self, name: &str, scope: i64) -> Result<Option<HLSym>> {
        let sql = "select scope, sc,sym,offset from csymbol  where csymbol.scope =?1 and name = ?2";
        let mut stmt = self.conn.prepare_cached(sql)?;
        match stmt.query_row(params![scope, name], |row| {
            // type integer,
            // sc text,
            // sym integer,
            // offset integer
            let scope = row.get::<usize, i64>(0)?;
            let sc = row.get::<usize, String>(1)?;
            let sym = row.get::<usize, i64>(2)?;
            let offset = row.get::<usize, i64>(3)?;
            Ok(HLSym {
                name: name.to_string(),
                type_: sc,
                scope,
                seg: 0,
                value: offset,
            })
        }) {
            Ok(id) => Ok(Some(id)),
            Err(e) => match e {
                rusqlite::Error::QueryReturnedNoRows => Ok(None),
                _ => Err(e.into()),
            },
        }
    }
}

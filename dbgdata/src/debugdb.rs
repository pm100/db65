use anyhow::{anyhow, bail, Result};
use rusqlite::Transaction;
use util::{say, verbose};

use crate::util::Extract;
use rusqlite::{
    params,
    types::{Null, Value as SqlValue},
    Connection,
};
use std::cell::Cell;
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
    pub loaded: Cell<bool>,
    pub failed: bool,
}
pub struct HLSym {
    pub name: String,
    pub value: i64,
    pub type_: String,
    pub seg: u8,
    pub scope: i64,
}
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SymbolType {
    Unknown,
    Equate,
    Label,
    CSymbol,
}
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Symbol {
    pub name: String,
    pub value: u16,
    pub module: String,
    pub sym_type: SymbolType,
}
pub struct SegChunk {
    pub offset: u16,
    pub module: i32,
    pub module_name: String,
    pub size: u16,
}
pub struct Segment {
    pub id: u8,       // number in db
    pub name: String, // name in db
    pub start: u16,   // start address
    pub size: u16,    // end address
    pub seg_type: u8, // type in db
    pub modules: Vec<SegChunk>,
}
pub enum SegmentType {
    Code = 0,
    ReadOnly = 1,
    ReadWrite = 2,
    Zp = 3,
    //  Bss = 4,
    // OverWrite = 5,
}
pub struct DebugData {
    pub conn: Connection,
    pub(crate) file_table: HashMap<i64, SourceFile>,
    pub cc65_dir: Option<PathBuf>,
}

impl DebugData {
    pub fn new(name: &str) -> Result<DebugData> {
        if fs::metadata(name).is_ok() {
            std::fs::remove_file(name)?;
        }
        let mut ret = Self {
            conn: Connection::open(name)?,
            file_table: HashMap::new(),
            cc65_dir: None,
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
    pub fn lookup_file_by_id(&self, file_id: i64) -> Option<&SourceFile> {
        self.file_table.get(&file_id)
    }
    pub fn lookup_file_by_name(&self, name: &str) -> Option<&SourceFile> {
        self.file_table.iter().find_map(|(_id, file)| {
            if file.short_name == name {
                Some(file)
            } else {
                None
            }
        })
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

    pub fn load_files(&mut self) -> Result<()> {
        let rows = self.query_db(&[], "select name,id from file")?;
        if self.cc65_dir.is_none() {
            self.cc65_dir = self.guess_cc65_dir()?;
        }
        let mut failed = 0;
        for row in rows {
            let name = row[0].vto_string()?;
            let id = row[1].vto_i64()?;
            let path = Path::new(&name);
            let mut sf = SourceFile {
                file_id: id,
                short_name: path.file_name().unwrap().to_str().unwrap().to_string(),
                full_path: PathBuf::new(),
                loaded: Cell::new(false),
                failed: false,
            };
            if let Some(p) = self.find_file(path)? {
                verbose!("found file {}", p.display());
                sf.full_path = p;
            } else {
                failed += 1;
                verbose!("can't find file {}", name);
                sf.failed = true;
            }
            self.file_table.insert(id, sf);
        }
        if failed > 0 {
            verbose!("Failed to find {} files", failed);
            say!("Some runtime files were not found, use the 'set -s' option to set the cc65 directory");
            say!(
                "'set -v on' to see the list of missing files; 'about ccode' for more information"
            );
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
            let _scope = row[1].vto_i64()?;
            let type_ = row[2].vto_string()?;
            let val = row[3].vto_i64()? as u16;
            let _seg = row[4].vto_i64()? as u8;
            v.push(Symbol {
                name,
                value: val,
                module: String::new(),
                sym_type: Self::convert_symbol_type(&type_),
            });
        }
        Ok(v)
    }
    pub fn set_cc65_dir(&mut self, dir: &PathBuf) -> Result<()> {
        self.cc65_dir = Some(dir.clone());
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
            let id = row[1].vto_i64()?;
            self.load_source_file(id)?;
        }
        Ok(())
    }

    pub fn load_source_file(&self, file_id: i64) -> Result<()> {
        let path;
        if let Some(fi) = self.file_table.get(&file_id) {
            if fi.loaded.get() {
                return Ok(());
            }
            if fi.failed {
                // we could not find the file, so don't try again
                return Ok(());
            }
            path = fi.full_path.clone();
        } else {
            bail!("bad file id {}", file_id);
        };
        verbose!("load source file {}", path.display());
        let full_path = if let Some(p) = self.find_file(&path)? {
            p
        } else {
            say!("can't find file {}", path.display());
            return Ok(());
        };

        let fd = File::open(full_path)?;
        let mut reader = BufReader::new(fd);
        let mut line = String::new();
        let mut lineno = 0;
        let file_name = path.file_name().ok_or(anyhow!("bad file name"))?.to_str();

        // let row = self
        //     .conn
        //     .prepare_cached("select id from file where name = ?1")?
        //     .query_row(params![path.to_str().unwrap()], |row| {
        //         row.get::<usize, i64>(0)
        //     })?;

        self.conn.execute(
            "insert into source (file_id,name) values(?1, ?2)",
            params![file_id, file_name],
        )?;
        let mut map = BTreeMap::new();
        self.get_source_file_lines(file_id, &mut map)?;
        let tx = Transaction::new_unchecked(&self.conn, rusqlite::TransactionBehavior::Deferred)?;
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
                        file_id,
                        lineno,
                        line.trim_end(),
                        inf.seg,
                        inf.addr,
                        inf.absaddr
                    ])?;
                } else {
                    stmt.execute(params![file_id, lineno, line.trim_end(), Null, Null, Null])?;
                }
            }
        }
        tx.commit()?;
        if let Some(ref mut fi) = self.file_table.get(&file_id) {
            fi.loaded.set(true);
        }
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
                    size,
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

        let si = self.internal_find_line(addr, sql)?;
        if si.is_none() {
            return Ok(None);
        }
        let mut si = si.unwrap();

        self.load_source_file(si.file_id)?;

        if let Some(l) = self.get_file_line(si.file_id, si.line_no)? {
            si.line = l;
        }

        Ok(Some(si))
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
            let _sym = row.get::<usize, i64>(2)?;
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

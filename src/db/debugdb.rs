use anyhow::{anyhow, bail, Result};
use evalexpr::Value;
//pub const NO_PARAMS:  = [];
use rusqlite::{
    params,
    types::{Null, Value as SqlValue},
    Connection, ToSql,
};
use std::{
    collections::{BTreeMap, HashMap},
    fs::{self, File},
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use crate::debugger::{SegChunk, Segment};
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
pub struct AddrSource {
    pub file_id: i64,
    pub line_no: i64,
    pub seg: u8,
    pub addr: u16,
    pub absaddr: u16,
}
pub struct DebugData {
    pub conn: Connection,
    cc65_dir: Option<PathBuf>,
}
impl DebugData {
    pub fn new() -> Result<DebugData> {
        let dbfile = "dbg.db";
        if fs::metadata(dbfile).is_ok() {
            std::fs::remove_file("dbg.db")?;
        }
        let mut ret = Self {
            conn: Connection::open("dbg.db")?,
            cc65_dir: None,
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
            let module = row.get::<usize, String>(2)?;

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
    pub fn load_expr_symbols(&mut self, sym_tab: &mut HashMap<String, Value>) -> Result<()> {
        let mut stmt = self
            .conn
            .prepare_cached("select name, val , module from symbol")?;
        let rows = stmt.query_map([], |row| {
            let name = row.get::<usize, String>(0)?;
            let val = row.get::<usize, i64>(1)? as u16;
            let module = row.get::<usize, String>(2)?;

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
            let mut module = row.get::<usize, String>(2)?;
            module = module.strip_suffix(".o").unwrap_or(&module).to_string();
            Ok((name, val, module))
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

    pub fn find_symbolx(&self, addr: u16) -> Result<Option<String>> {
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
    pub fn find_symbol(&self, addr: u16) -> Result<Option<String>> {
        let ans = self.query_db(
            params![addr],
            "select symdef.name from symdef where symdef.val = ?1 and seg not null",
        )?;

        for row in ans {
            if let SqlValue::Text(s) = &row[0] {
                return Ok(Some(s.to_string()));
            }
        }
        Ok(None)
    }

    pub fn load_all_cfiles(&mut self) -> Result<()> {
        let rows = self.query_db(
            &[],
            "select name,file.id from file,cline where cline.file = file.id group by name",
        )?;
        self.cc65_dir = self.guess_cc65_dir()?;
        for row in rows {
            if let SqlValue::Text(name) = &row[0] {
                let path = Path::new(&name);

                self.load_source_file(path)?;
            }
        }
        Ok(())
    }

    fn find_file(&self, file: &Path) -> Result<Option<PathBuf>> {
        // find a file somewhere
        if file.is_absolute() {
            if file.exists() {
                return Ok(Some(file.to_path_buf()));
            }
        } else {
            if let Some(cc65) = &self.cc65_dir {
                let mut p = PathBuf::new();
                p.push(cc65);
                p.push("libsrc");
                p.push(file);
                if p.exists() {
                    return Ok(Some(p));
                }
            }
            if file.exists() {
                return Ok(Some(file.to_path_buf()));
            }
        }
        Ok(None)
    }
    pub fn load_source_file(&mut self, file: &Path) -> Result<()> {
        println!("load source file {}", file.display());

        let full_path = if let Some(p) = self.find_file(file)? {
            p
        } else {
            bail!("can't find file {}", file.display())
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
    pub fn load_all_source_files(&mut self, map: &mut BTreeMap<u16, AddrSource>) -> Result<()> {
        let rows = self.query_db(&[], "select distinct file from cline")?;

        for row in rows {
            if let SqlValue::Integer(id) = &row[0] {
                self.get_source_file_lines2(*id, map)?;
            }
        }
        Ok(())
    }
    pub fn get_source_file_lines(
        &self,
        file: i64,
        hash: &mut BTreeMap<i64, SourceInfo>,
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
            hash.insert(info.line_no, info);
        }
        Ok(())
    }

    pub fn get_source_file_lines2(
        &self,
        file: i64,
        hash: &mut BTreeMap<u16, AddrSource>,
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
            Ok(AddrSource {
                line_no,
                seg: seg as u8,
                addr,
                absaddr,

                file_id: file,
            })
        })?;
        for row in rows {
            let info = row?;
            hash.insert(info.absaddr, info);
        }
        Ok(())
    }

    pub fn find_source_line(&self, addr: u16) -> Result<Option<SourceInfo>> {
        let sql =
            "select * from (select * from source_line order by absaddr desc) where absaddr <= ?1 limit 1";
        let mut stmt = self.conn.prepare_cached(sql)?;
        match stmt.query_row(params![addr], |row| {
            // id integer primary key,
            // file integer,
            // line text not null,
            // line_no integer,
            // seg integer,
            // addr integer,
            // absaddr integer
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
            Err(e) => {
                println!("{:?}", e);
                Ok(None)
            }
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
                let end = row.get::<usize, u16>(3)?;

                Ok(Segment {
                    id: seg as u8,
                    name,
                    start,
                    end,
                    modules: Vec::new(),
                    seg_type: 0,
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
            let (name, id, _file, segid, start, _size) = chunk?;
            if let Some(seg) = seg_list.iter_mut().find(|s| s.id == segid as u8) {
                seg.modules.push(SegChunk {
                    offset: start,
                    module: id as i32,
                    module_name: name,
                });
            } else {
                println!("bad segid {}", segid);
            }
        }
        Ok(())
    }

    fn guess_cc65_dir(&self) -> Result<Option<PathBuf>> {
        let ans = self.query_db(&[], "select name from file")?;
        for row in ans {
            if let SqlValue::Text(s) = &row[0] {
                let path = Path::new(s);
                if path.is_absolute() {
                    if let Some(p) = path.parent() {
                        if p.ends_with("include") {
                            return Ok(Some(p.parent().unwrap().to_path_buf()));
                        }
                    }
                }
            }
        }
        Ok(None)
    }

    // general purpose query function
    fn query_db(&self, params: &[&dyn ToSql], query: &str) -> Result<Vec<Vec<SqlValue>>> {
        let mut stmt = self.conn.prepare_cached(query)?;
        let cols = stmt.column_count();

        for (i, p) in params.iter().enumerate() {
            stmt.raw_bind_parameter(i + 1, p)?;
        }
        let mut rows = stmt.raw_query();
        let mut result = Vec::new();
        while let Some(r) = rows.next()? {
            let mut row_vec = Vec::new();
            for i in 0..cols {
                let val = r.get::<usize, SqlValue>(i).unwrap();
                row_vec.push(val);
            }
            result.push(row_vec);
        }

        Ok(result)
    }
}

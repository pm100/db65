use anyhow::{anyhow, bail, Result};
//pub const NO_PARAMS:  = [];
use rusqlite::{params, Connection, Result as SqlResult};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::BufReader,
};

use crate::debugdb::DebugData;

#[derive(Debug)]
pub struct CsymRecord {
    pub id: i64,
    pub name: String,
    pub scope: i64,
    pub type_: i64,
    pub sc: String,
    pub sym: i64,
    pub offset: i64,
}
#[derive(Debug)]
pub struct FileRecord {
    pub id: i64,
    pub name: String,
    pub size: i64,
    pub mod_time: i64,
    pub module: Vec<i64>,
}
#[derive(Debug)]
pub struct LineRecord {
    pub id: i64,
    pub file: i64,
    pub line: i64,
    pub type_: i64,
    pub count: i64,
    pub span: Vec<i64>,
}
#[derive(Debug)]
pub struct ModuleRecord {
    pub id: i64,
    pub name: String,
    pub file: i64,
    pub lib: i64,
}
#[derive(Debug)]
pub struct SegmentRecord {
    pub id: i64,
    pub name: String,
    pub start: i64,
    pub size: i64,
    pub addrsize: String,
    pub type_: String,
    pub oname: String,
    pub ooffs: i64,
}
#[derive(Debug)]
pub struct SpanRecord {
    pub id: i64,
    pub seg: i64,
    pub start: i64,
    pub size: i64,
    pub type_: i64,
}
#[derive(Debug)]
pub struct ScopeRecord {
    pub id: i64,
    pub name: String,
    pub module: i64,
    pub type_: String,
    pub size: i64,
    pub parent: i64,
    pub sym: i64,
    pub span: Vec<i64>,
}
#[derive(Debug)]
pub struct SymbolRecord {
    pub id: i64,
    pub name: String,
    pub addrsize: String,
    pub scope: i64,
    pub parent: i64,
    pub ref_: Vec<i64>,
    pub def: Vec<i64>,
    pub type_: String,
    pub exp: i64,
    pub val: u16,
    pub seg: Option<i64>,
    pub size: i64,
}
impl DebugData {
    pub fn create_tables(&mut self) -> Result<()> {
        self.conn.execute(
            "create table symdef (
             id integer primary key,
             name text not null ,
             addrsize text,
                scope integer,
                def integer,
                type text,
                exp integer,
                val integer,
                seg integer,
                 size integer,
                 parent integer
            

         )",
            [],
        )?;

        self.conn.execute(
            "create table symref (
             id integer primary key,
             name text not null ,
             addrsize text,
                scope integer,
                def integer,
                type integer,
                exp integer,
                val integer,
                seg integer,
              size integer,
                 parent integer
            

         )",
            [],
        )?;
        self.conn.execute(
            "create table line (
            id integer primary key,
             file integer,
            line_no integer ,
             type integer,
             count integer
         )",
            [],
        )?;
        self.conn.execute(
            "create table file (
            id integer primary key,
             name text,
            size integer ,
             mod_time integer
             
         )",
            [],
        )?;
        self.conn.execute(
            "create table module (
            id integer primary key,
             name text,
            file integer ,
             lib integer
             
         )",
            [],
        )?;

        self.conn.execute(
            "create table segment (
    id integer primary key,
     name text,
    start integer ,
     size integer,
     addrsize integer,
        type integer,
        oname integer,
        ooffs integer        

     
 )",
            [],
        )?;
        self.conn.execute(
            "create table span (
    id integer primary key,
     seg integer,
    start integer ,
            
        size integer,
        type integer
     
    )",
            [],
        )?;

        self.conn.execute(
            "create table scope (
    id integer primary key,
        name text,
        module integer,
        type integer,
        size integer,
        parent integer,
        sym integer,
        span integer         
    )",
            [],
        )?;

        self.conn.execute(
            "create table csymbol (
    id integer primary key,
        name text,
        scope integer,
        type integer,
        sc text,
        sym integer,
        offset integer
    )",
            [],
        )?;
        Ok(())
    }
    pub fn parse(&mut self, reader: &mut BufReader<File>) -> Result<()> {
        let tx = self.conn.transaction()?;
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .flexible(true)
            .from_reader(reader);
        for result in rdr.records() {
            let record = result?;
            //println!("{:?}", record);
            let hdr = record.get(0).ok_or(anyhow!("bad file"))?;
            match Self::parse_eq(hdr)?.0.as_str() {
                "version\tmajor" => println!("version"),
                "info\tcsym" => println!("information"),
                "csym\tid" => {
                    let csym = Self::parse_csym(&record)?;
                    tx.execute(
                        "INSERT INTO csymbol 
                             values (?1,?2,?3,?4,?5, ?6 ,?7)",
                        params![
                            csym.id,
                            csym.name,
                            csym.scope,
                            csym.type_,
                            csym.sc,
                            csym.sym,
                            csym.offset
                        ],
                    )?;
                }
                "file\tid" => {
                    let file = Self::parse_file(&record)?;
                    tx.execute(
                        "INSERT INTO file 
                             values (?1,?2,?3,?4)",
                        params![file.id, file.name, file.size, file.mod_time],
                    )?;
                }
                "lib\tid" => println!("lib"),
                "line\tid" => {
                    let line = Self::parse_line(&record)?;

                    tx.execute(
                        "INSERT INTO line (                id ,
                            file,
                           line_no  ,
                            type, 
                            count ) 
                             values (?1,?2,?3,?4,?5)",
                        params![line.id, line.file, line.line, line.type_, line.count],
                    )?;
                }
                "mod\tid" => {
                    let module = Self::parse_mod(&record)?;
                    tx.execute(
                        "INSERT INTO module (                id ,
                            name,
                           file  ,
                            lib ) 
                             values (?1,?2,?3,?4)",
                        params![module.id, module.name, module.file, module.lib],
                    )?;
                }
                "seg\tid" => {
                    let seg = Self::parse_seg(&record)?;
                    tx.execute(
                        "INSERT INTO segment (                id ,
                            name,
                           start  ,
                            size ,
                            addrsize,
                            type,
                            oname,
                            ooffs ) 
                             values (?1,?2,?3,?4,?5,?6,?7,?8)",
                        params![
                            seg.id,
                            seg.name,
                            seg.start,
                            seg.size,
                            seg.addrsize,
                            seg.type_,
                            seg.oname,
                            seg.ooffs
                        ],
                    )?;
                }
                "span\tid" => {
                    let span = Self::parse_span(&record)?;
                    tx.execute(
                        "INSERT INTO span (                id ,
                            seg,
                           start  ,
                            size ,
                            type ) 
                             values (?1,?2,?3,?4,?5)",
                        params![span.id, span.seg, span.start, span.size, span.type_],
                    )?;
                }
                "scope\tid" => {
                    let scope = Self::parse_scope(record)?;
                    tx.execute(
                        "INSERT INTO scope (                id ,
                            name,
                           module  ,
                            type ,
                            size,
                            parent,
                            sym,
                            span ) 
                             values (?1,?2,?3,?4,?5,?6,?7,?8)",
                        params![
                            scope.id,
                            scope.name,
                            scope.module,
                            scope.type_,
                            scope.size,
                            scope.parent,
                            scope.sym,
                            if scope.span.len() > 0 {
                                scope.span[0]
                            } else {
                                -1
                            }
                        ],
                    )?;
                }
                "sym\tid" => {
                    let sym = Self::parse_sym(record)?;
                    if sym.type_ == "imp" {
                        tx.execute(
                            "INSERT INTO symref (id, name,
                             scope , def ,type, exp,  val ,  seg , size , parent, addrsize ) 
                             values (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10, ?11)",
                            params![
                                sym.id,
                                sym.name,
                                sym.scope,
                                sym.def[0],
                                sym.type_,
                                sym.exp,
                                sym.val,
                                sym.seg,
                                sym.size,
                                sym.parent,
                                sym.addrsize
                            ],
                        )?;
                    } else {
                        tx.execute(
                            "INSERT INTO symdef (id, name,
                             scope , def ,type, exp,  val ,  seg , size , parent, addrsize ) 
                             values (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10, ?11)",
                            params![
                                sym.id,
                                sym.name,
                                sym.scope,
                                sym.def[0],
                                sym.type_,
                                sym.exp,
                                sym.val,
                                sym.seg,
                                sym.size,
                                sym.parent,
                                sym.addrsize
                            ],
                        )?;
                    }
                }
                _ => {} //println!("other"),
            };
        }
        // println!("csyms: {}", self.csyms.len());
        // println!("files: {}", self.files.len());
        // println!("lines: {}", self.lines.len());
        // println!("modules: {}", self.modules.len());
        // println!("segments: {}", self.segments.len());
        // println!("spans: {}", self.spans.len());
        // println!("scopes: {}", self.scopes.len());
        // println!("symbols: {}", self.symbols.len());
        // for sym in &self.symbols {
        //     let syms = self.symlk.entry(sym.name.clone()).or_insert(Vec::new());
        //     syms.push(sym.id as usize);
        // }
        tx.commit()?;
        Ok(())
    }

    fn parse_eq(string: &str) -> Result<(String, String)> {
        let mut iter = string.split('=');
        let key = iter.next().unwrap();
        let value = iter.next().unwrap();
        //let value = parse_value(value)?;
        Ok((key.to_string(), value.to_string()))
    }

    fn get_number(string: &String) -> Result<i64> {
        if let Ok(number) = string.parse::<i64>() {
            return Ok(number);
        }
        Err(anyhow::anyhow!(
            "unexpected field format (wanted number): {:?}",
            string
        ))?
    }
    fn get_string(string: &String) -> Result<String> {
        if string.starts_with('"') && string.ends_with('"') {
            Ok(string[1..string.len() - 1].to_string())
        } else {
            Ok(string.to_string())
        }
    }
    fn get_num_array(string: &String) -> Result<Vec<i64>> {
        if string.contains('+') {
            let mut array = Vec::new();
            let iter = string[0..string.len()].split('+');
            for item in iter {
                array.push(item.parse::<i64>()?);
            }
            Ok(array)
        } else {
            Ok(vec![string.parse::<i64>()?])
        }
    }
    fn get_hex_num(string: &String) -> Result<i64> {
        if let Some(hex) = string.strip_prefix('$').or_else(|| {
            string
                .strip_prefix("0x")
                .or_else(|| string.strip_prefix("0X"))
        }) {
            return Ok(i64::from_str_radix(hex, 16)?);
        }
        Err(anyhow::anyhow!(
            "unexpected field format (wanted hex number): {:?}",
            string
        ))?
    }
    fn get_hex_addr(string: &String) -> Result<u16> {
        if let Some(hex) = string.strip_prefix('$').or_else(|| {
            string
                .strip_prefix("0x")
                .or_else(|| string.strip_prefix("0X"))
        }) {
            return Ok(u32::from_str_radix(hex, 16)? as u16);
        }
        Err(anyhow::anyhow!(
            "unexpected field format (wanted hex addr): {:?}",
            string
        ))?
    }
    fn parse_csym(record: &csv::StringRecord) -> Result<CsymRecord> {
        // csym	id=0,name="printf",scope=0,type=0,sc=ext,sym=16
        // csym	id=1,name="main",scope=1,type=0,sc=ext,sym=15
        // csym	id=2,name="argc",scope=1,type=0,sc=auto,offs=2
        // csym	id=3,name="argv",scope=1,type=0,sc=auto
        // csym	id=4,name="i",scope=1,type=0,sc=auto,offs=-2

        let mut rec = CsymRecord {
            id: 0,
            name: String::new(),
            scope: 0,
            type_: 0,
            sc: String::new(),
            sym: 0,
            offset: 0,
        };
        for next_pr in record.iter() {
            let next = Self::parse_eq(next_pr)?;
            match next.0.as_str() {
                "csym\tid" => rec.id = Self::get_number(&next.1)?,
                "name" => rec.name = Self::get_string(&next.1)?.to_string(),
                "scope" => rec.scope = Self::get_number(&next.1)?,
                "sym" => rec.sym = Self::get_number(&next.1)?,
                "type" => rec.type_ = Self::get_number(&next.1)?,
                "offs" => rec.offset = Self::get_number(&next.1)?,
                "sc" => rec.sc = Self::get_string(&next.1)?.to_string(),
                _ => bail!("unexpected field: {}", next.0),
            }
        }
        // println!("{:?}", rec);
        Ok(rec)
    }

    fn parse_file(record: &csv::StringRecord) -> Result<FileRecord> {
        // file	id=0,name="argtest.s",size=1968,mtime=0x659B1882,mod=0
        // file	id=1,name="c:\tools\cc65\asminc/longbranch.mac",size=2632,mtime=0x65652373,mod=0
        // file	id=2,name="argtest.c",size=187,mtime=0x658E6156,mod=0
        // file	id=3,name="c:\tools\cc65\include/stdio.h",size=6870,mtime=0x65652376,mod=0
        // file	id=4,name="/home/runner/work/cc65/cc65/asminc/longbranch.mac",size=2632,mtime=0x6564C9BD,mod=2+3
        // file	id=5,name="/home/runner/work/cc65/cc65/asminc/stdio.inc",size=3411,mtime=0x6564C9BD,mod=1
        // file	id=6,name="/home/runner/work/cc65/cc65/asminc/errno.inc",size=1363,mtime=0x6564C9BD,mod=5+19+21

        let mut rec = FileRecord {
            id: 0,
            name: String::new(),
            size: 0,
            mod_time: 0,
            module: Vec::new(),
        };

        for next_pr in record.iter() {
            let next = Self::parse_eq(next_pr)?;
            match next.0.as_str() {
                "file\tid" => rec.id = Self::get_number(&next.1)?,
                "name" => rec.name = Self::get_string(&next.1)?.to_string(),
                "size" => rec.size = Self::get_number(&next.1)?,
                "mtime" => rec.mod_time = Self::get_hex_num(&next.1)?,
                "mod" => rec.module = Self::get_num_array(&next.1)?,
                _ => bail!("unexpected field: {}", next.0),
            }
        }
        //  println!("{:?}", rec);
        Ok(rec)
    }

    fn parse_line(record: &csv::StringRecord) -> Result<LineRecord> {
        // lib	id=0,name="c:\tools\cc65\lib/sim6502.lib"
        // line	id=0,file=0,line=25
        // line	id=1,file=0,line=19
        // line	id=2,file=1,line=20,type=2,count=1,span=23
        // line	id=3,file=0,line=85,span=45
        // line	id=4,file=0,line=11
        // line	id=5,file=0,line=45,span=3
        // line	id=6,file=1,line=16,type=2,count=1
        // line	id=7,file=0,line=80,span=40

        let mut rec = LineRecord {
            id: 0,
            file: 0,
            line: 0,
            type_: 0,
            count: 0,
            span: Vec::new(),
        };
        for next_pr in record.iter() {
            let next = Self::parse_eq(next_pr)?;
            match next.0.as_str() {
                "line\tid" => rec.id = Self::get_number(&next.1)?,
                "file" => rec.file = Self::get_number(&next.1)?,
                "line" => rec.line = Self::get_number(&next.1)?,
                "count" => rec.count = Self::get_number(&next.1)?,
                "span" => rec.span = Self::get_num_array(&next.1)?,
                "type" => rec.type_ = Self::get_number(&next.1)?,
                _ => bail!("unexpected field: {}", next.0),
            }
        }
        //println!("{:?}", rec);
        Ok(rec)
    }

    fn parse_mod(record: &csv::StringRecord) -> Result<ModuleRecord> {
        // mod	id=0,name="argtest.o",file=0
        // mod	id=1,name="_file.o",file=9,lib=0
        // mod	id=2,name="_hextab.o",file=12,lib=0
        // mod	id=3,name="_longminstr.o",file=15,lib=0
        // mod	id=4,name="_printf.o",file=17,lib=0
        // mod	id=5,name="_seterrno.o",file=18,lib=0
        // mod	id=6,name="add.o",file=19,lib=0
        // mod	id=7,name="addeqsp.o",file=20,lib=0
        // mod	id=8,name="addysp.o",file=21,lib=0

        let mut rec = ModuleRecord {
            id: 0,
            name: String::new(),
            file: 0,
            lib: 0,
        };
        for next_pr in record.iter() {
            let next = Self::parse_eq(next_pr)?;
            match next.0.as_str() {
                "mod\tid" => rec.id = Self::get_number(&next.1)?,
                "name" => rec.name = Self::get_string(&next.1)?.to_string(),
                "file" => rec.file = Self::get_number(&next.1)?,
                "lib" => rec.lib = Self::get_number(&next.1)?,
                _ => bail!("unexpected field: {}", next.0),
            }
        }
        //println!("{:?}", rec);
        Ok(rec)
    }

    fn parse_seg(record: &csv::StringRecord) -> Result<SegmentRecord> {
        // seg	id=0,name="CODE",start=0x000239,size=0x08CE,addrsize=absolute,type=ro,oname="argtest",ooffs=69
        // seg	id=1,name="RODATA",start=0x000B07,size=0x00BA,addrsize=absolute,type=ro,oname="argtest",ooffs=2323
        // seg	id=2,name="BSS",start=0x000C13,size=0x0030,addrsize=absolute,type=rw
        // seg	id=3,name="DATA",start=0x000BC1,size=0x0052,addrsize=absolute,type=rw,oname="argtest",ooffs=2509
        // seg	id=4,name="ZEROPAGE",start=0x000000,size=0x001A,addrsize=zeropage,type=rw
        // seg	id=5,name="NULL",start=0x000000,size=0x0000,addrsize=absolute,type=rw
        // seg	id=6,name="ONCE",start=0x00021D,size=0x001C,addrsize=absolute,type=ro,oname="argtest",ooffs=41
        // seg	id=7,name="STARTUP",start=0x000200,size=0x001D,addrsize=absolute,type=ro,oname="argtest",ooffs=12
        // seg	id=8,name="EXEHDR",start=0x000000,size=0x000C,addrsize=absolute,type=ro,oname="argtest",ooffs=0

        let mut rec = SegmentRecord {
            id: 0,
            name: String::new(),
            start: 0,
            size: 0,
            addrsize: String::new(),
            type_: String::new(),
            oname: String::new(),
            ooffs: 0,
        };

        for next_pr in record.iter() {
            let next = Self::parse_eq(next_pr)?;
            match next.0.as_str() {
                "seg\tid" => rec.id = Self::get_number(&next.1)?,
                "name" => rec.name = Self::get_string(&next.1)?.to_string(),
                "start" => rec.start = Self::get_hex_num(&next.1)?,
                "size" => rec.size = Self::get_hex_num(&next.1)?,
                "addrsize" => rec.addrsize = Self::get_string(&next.1)?.to_string(),
                "type" => rec.type_ = Self::get_string(&next.1)?.to_string(),
                "oname" => rec.oname = Self::get_string(&next.1)?.to_string(),
                "ooffs" => rec.ooffs = Self::get_number(&next.1)?,
                _ => bail!("unexpected field: {}", next.0),
            }
        }
        //  println!("{:?}", rec);
        Ok(rec)
    }

    fn parse_span(record: &csv::StringRecord) -> Result<SpanRecord> {
        // span	id=0,seg=1,start=0,size=10,type=1
        // span	id=1,seg=1,start=10,size=9,type=2
        // span	id=2,seg=0,start=0,size=3
        // span	id=3,seg=0,start=3,size=2
        // span	id=4,seg=0,start=5,size=2
        // span	id=5,seg=0,start=7,size=3
        // span	id=6,seg=0,start=10,size=2
        // span	id=7,seg=0,start=12,size=3
        // span	id=8,seg=0,start=15,size=3
        // span	id=9,seg=0,start=18,size=2
        // span	id=10,seg=0,start=20,size=3

        let mut rec = SpanRecord {
            id: 0,
            seg: 0,
            start: 0,
            size: 0,
            type_: 0,
        };
        for next_pr in record.iter() {
            let next = Self::parse_eq(next_pr)?;
            match next.0.as_str() {
                "span\tid" => rec.id = Self::get_number(&next.1)?,
                "seg" => rec.seg = Self::get_number(&next.1)?,
                "start" => rec.start = Self::get_number(&next.1)?,
                "size" => rec.size = Self::get_number(&next.1)?,
                "type" => rec.type_ = Self::get_number(&next.1)?,
                _ => bail!("unexpected field: {}", next.0),
            }
        }
        //  println!("{:?}", rec);
        Ok(rec)
    }
    fn parse_scope(reader: csv::StringRecord) -> Result<ScopeRecord> {
        // scope	id=15,name="callmain",mod=11,type=scope,size=23,parent=14,sym=247,span=608
        // scope	id=16,name="",mod=12,size=12,span=618+641+625
        // scope	id=17,name="initlib",mod=12,type=scope,size=12,parent=16,sym=282,span=618
        // scope	id=18,name="donelib",mod=12,type=scope,size=12,parent=16,sym=281,span=625
        // scope	id=19,name="condes",mod=12,type=scope,size=37,parent=16,sym=280,span=641

        let mut rec = ScopeRecord {
            id: 0,
            name: String::new(),
            module: 0,
            type_: String::new(),
            size: 0,
            parent: 0,
            sym: 0,
            span: Vec::new(),
        };

        for next_pr in reader.iter() {
            let next = Self::parse_eq(next_pr)?;
            match next.0.as_str() {
                "scope\tid" => rec.id = Self::get_number(&next.1)?,
                "name" => rec.name = Self::get_string(&next.1)?.to_string(),
                "mod" => rec.module = Self::get_number(&next.1)?,
                "type" => rec.type_ = Self::get_string(&next.1)?.to_string(),
                "size" => rec.size = Self::get_number(&next.1)?,
                "parent" => rec.parent = Self::get_number(&next.1)?,
                "sym" => rec.sym = Self::get_number(&next.1)?,
                "span" => rec.span = Self::get_num_array(&next.1)?,
                _ => bail!("unexpected field: {}", next.0),
            }
        }
        //println!("{:?}", rec);
        Ok(rec)
    }

    fn parse_sym(reader: csv::StringRecord) -> Result<SymbolRecord> {
        // sym	id=12,name="decsp2",addrsize=absolute,scope=1,def=35,ref=49,type=imp,exp=348
        // sym	id=13,name="S0001",addrsize=absolute,scope=0,def=0,ref=5+34,val=0xB11,seg=1,type=lab
        // sym	id=14,name="S0002",addrsize=absolute,scope=0,def=42,ref=19+60,val=0xB07,seg=1,type=lab
        // sym	id=15,name="_main",addrsize=absolute,size=111,scope=0,def=15,ref=1,val=0x239,seg=0,type=lab
        // sym	id=16,name="_printf",addrsize=absolute,scope=0,def=20,ref=31+62,type=imp,exp=620
        // sym	id=17,name="__SIM6502__",addrsize=zeropage,scope=0,def=12,val=0x1,type=equ
        // sym	id=18,name="_FPUSHBACK",addrsize=zeropage,scope=2,def=70,val=0x8,type=equ
        // sym	id=19,name="_FERROR",addrsize=zeropage,scope=2,def=83,val=0x4,type=equ
        // sym	id=20,name="_FEOF",addrsize=zeropage,scope=2,def=121,val=0x2,type=equ
        // sym	id=21,name="_FOPEN",addrsize=zeropage,scope=2,def=98,ref=68+77+87,val=0x1,type=equ
        // sym	id=22,name="_FCLOSED",addrsize=zeropage,scope=2,def=71,ref=118+118+118+118+118,val=0x0,type=equ

        let mut rec = SymbolRecord {
            id: 0,
            name: String::new(),
            addrsize: String::new(),
            scope: 0,
            ref_: Vec::new(),
            def: Vec::new(),
            type_: String::new(),
            exp: 0,
            val: 0,
            seg: None,
            size: 0,
            parent: 0,
        };

        for next_pr in reader.iter() {
            let next = Self::parse_eq(next_pr)?;
            match next.0.as_str() {
                "sym\tid" => rec.id = Self::get_number(&next.1)?,
                "name" => rec.name = Self::get_string(&next.1)?.to_string(),
                "addrsize" => rec.addrsize = Self::get_string(&next.1)?.to_string(),
                "scope" => rec.scope = Self::get_number(&next.1)?,
                "ref" => rec.ref_ = Self::get_num_array(&next.1)?,
                "def" => rec.def = Self::get_num_array(&next.1)?,
                "type" => rec.type_ = Self::get_string(&next.1)?.to_string(),
                "exp" => rec.exp = Self::get_number(&next.1)?,
                "val" => rec.val = Self::get_hex_addr(&next.1)?,
                "seg" => rec.seg = Some(Self::get_number(&next.1)?),
                "size" => rec.size = Self::get_number(&next.1)?,
                "parent" => rec.parent = Self::get_number(&next.1)?,
                _ => bail!("unexpected field: {}", next.0),
            }
        }
        // println!("{:?}", rec);
        Ok(rec)
    }
}

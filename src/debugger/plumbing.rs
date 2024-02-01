/*
    Simple plumbing code passing requests from shell to debug engine

*/

use std::{
    collections::{BTreeMap, HashMap},
    path::{Path, PathBuf},
};

use crate::db::debugdb::{SourceFile, SourceInfo};

use super::{
    core::{BreakPoint, Debugger, HeapBlock, Segment, StackFrame, Symbol, WatchPoint},
    cpu::Cpu,
};
use anyhow::{bail, Result};
impl Debugger {
    pub fn enable_stack_check(&mut self, enable: bool) {
        self.enable_stack_check = enable;
    }
    pub fn enable_mem_check(&mut self, enable: bool) {
        self.enable_mem_check = enable;
    }
    pub fn enable_heap_check(&mut self, enable: bool) {
        self.enable_heap_check = enable;
    }
    pub fn set_cc65_dir(&mut self, dir: &PathBuf) -> Result<()> {
        if !dir.exists() {
            bail!("{:?} does not exist", dir);
        }
        self.dbgdb.set_cc65_dir(dir)
    }
    pub fn get_cc65_dir(&self) -> &Path {
        self.dbgdb
            .cc65_dir
            .as_deref()
            .unwrap_or_else(|| Path::new(""))
    }
    pub fn find_source_line(&self, addr: u16) -> Result<Option<SourceInfo>> {
        self.dbgdb.find_source_line(addr)
    }
    pub fn get_addr_map(&self) -> &BTreeMap<u16, SourceInfo> {
        &self.source_info
    }
    pub fn set_dbgfile_suffix(&mut self, suffix: &str) {
        self.dbg_suffix = suffix.to_string();
    }
    pub fn get_source(&self, file: i64, from: i64, to: i64) -> Result<Vec<String>> {
        self.dbgdb.get_source(file, from, to)
    }
    pub fn get_segments(&self) -> &Vec<Segment> {
        &self.seg_list
    }
    pub fn get_dbg_symbols(&self, filter: Option<&String>) -> Result<Vec<Symbol>> {
        let s = self.dbgdb.get_symbols(filter)?;
        Ok(s)
    }
    pub fn get_breaks(&self) -> Result<&BTreeMap<u16, BreakPoint>> {
        Ok(&self.break_points)
    }
    pub fn get_watches(&self) -> Result<&BTreeMap<u16, WatchPoint>> {
        Ok(&self.watch_points)
    }
    pub fn get_heap_blocks(&self) -> &HashMap<u16, HeapBlock> {
        &self.heap_blocks
    }
    pub fn get_chunk(&self, addr: u16, mut len: u16) -> Result<Vec<u8>> {
        let mut v = Vec::new();
        let max_add = addr.saturating_add(len);
        len = max_add - addr;
        for i in 0..len {
            v.push(Cpu::read_byte(addr + i));
        }
        Ok(v)
    }
    pub fn write_byte(&mut self, addr: u16, val: u8) {
        Cpu::write_byte(addr, val);
    }
    pub fn read_pc(&self) -> u16 {
        Cpu::read_pc()
    }
    pub fn read_sp(&self) -> u8 {
        Cpu::read_sp()
    }
    pub fn read_ac(&self) -> u8 {
        Cpu::read_ac()
    }
    pub fn read_xr(&self) -> u8 {
        Cpu::read_xr()
    }
    pub fn read_yr(&self) -> u8 {
        Cpu::read_yr()
    }

    pub fn read_sr(&self) -> u8 {
        Cpu::read_sr()
    }
    pub fn write_ac(&self, v: u8) {
        Cpu::write_ac(v);
    }
    pub fn write_xr(&self, v: u8) {
        Cpu::write_xr(v);
    }
    pub fn write_yr(&self, v: u8) {
        Cpu::write_yr(v);
    }

    pub fn write_sr(&self, v: u8) {
        Cpu::write_sr(v);
    }
    pub fn write_sp(&self, v: u8) {
        Cpu::write_sp(v);
    }
    pub fn write_pc(&self, v: u16) {
        Cpu::write_pc(v);
    }
    pub fn read_stack(&self) -> &Vec<StackFrame> {
        &self.stack_frames
    }
    pub fn lookup_file_by_id(&self, file_id: i64) -> Option<&SourceFile> {
        self.dbgdb.lookup_file_by_id(file_id)
    }
    pub fn lookup_file_by_name(&self, name: &str) -> Option<&SourceFile> {
        self.dbgdb.lookup_file_by_name(name)
    }

    pub fn read_sp65(&self) -> u16 {
        let sp65_addr = Cpu::get_sp65_addr();
        Cpu::read_word(sp65_addr as u16)
    }

}

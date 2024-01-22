/*
The debugger core. It sits on top of the Cpu wrapper that has all the unsafe code
It does not do any ui. This allows for maybe a future gui to provide
the same functionality as the cli shell.

*/

use anyhow::{bail, Result};

use std::{
    collections::{BTreeMap, HashMap},
    fmt::Debug,
    fs::File,
    io::BufReader,
    path::Path,
};

use crate::{
    db::debugdb::{DebugData, SourceFile, SourceInfo},
    debugger::cpu::{Cpu, ShadowFlags},
    debugger::execute::StopReason,
    debugger::loader,
    expr::DB65Context,
};

pub enum SourceDebugMode {
    None,
    Next,
    Step,
}

pub enum DebugMode {
    Unknown,
    Source,
    Assembler,
}
#[derive(Debug)]
pub struct CodeLocation {
    pub module: Option<i32>,
    pub cfile: Option<i64>,
    pub cline: i64,
    pub ctext: Option<String>,
    pub afile: Option<i64>,
    pub aline: i64,
    pub seg: u8,
    pub offset: u16,
    pub absaddr: u16,
}
pub enum SymbolType {
    Unknown,
    Equate,
    Label,
    CSymbol,
}
pub struct Symbol {
    pub name: String,
    pub value: u16,
    pub module: String,
    pub sym_type: SymbolType,
}
type InterceptFunc = fn(&mut Debugger, bool) -> Result<Option<StopReason>>;
pub struct Debugger {
    pub break_points: HashMap<u16, BreakPoint>,
    pub(crate) watch_points: HashMap<u16, WatchPoint>,
    pub(crate) source_info: BTreeMap<u16, SourceInfo>,
    pub(crate) current_file: Option<i64>,
    pub(crate) next_bp: Option<u16>,
    pub(crate) call_intercepts: HashMap<u16, InterceptFunc>,
    loader_start: u16,
    pub(crate) source_mode: SourceDebugMode,
    pub(crate) dis_line: String,
    pub(crate) ticks: usize,
    pub(crate) stack_frames: Vec<StackFrame>,
    pub(crate) enable_stack_check: bool,
    pub(crate) enable_mem_check: bool,
    load_name: String,
    pub(crate) run_done: bool,
    pub(crate) expr_context: DB65Context,
    pub(crate) dbgdb: DebugData,
    pub(crate) seg_list: Vec<Segment>,
    pub(crate) heap_blocks: HashMap<u16, HeapBlock>,
    pub(crate) privileged_mode: bool,
    pub(crate) debug_mode: DebugMode,
    pub(crate) file_table: HashMap<i64, SourceFile>,
}

pub struct HeapBlock {
    pub addr: u16,
    pub size: u16,
    pub alloc_addr: u16,
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
#[derive(Debug)]
pub(crate) enum FrameType {
    Jsr((u16, u16, u8, u16)), // addr, return addr,sp,sp65
    Pha(u8),
    Php(u8),
}
#[derive(Debug)]
pub struct StackFrame {
    pub(crate) frame_type: FrameType,
    pub(crate) stop_on_pop: bool,
}
#[derive(Debug, Clone)]
pub struct BreakPoint {
    pub(crate) addr: u16,
    pub(crate) symbol: String,
    pub(crate) number: usize,
    pub(crate) temp: bool,
}
#[derive(Debug, Clone)]
pub enum WatchType {
    Read,
    Write,
    ReadWrite,
}
pub struct WatchPoint {
    pub(crate) addr: u16,
    pub(crate) symbol: String,
    pub(crate) number: usize,
    pub(crate) watch: WatchType,
}
impl Debugger {
    pub fn new() -> Self {
        Cpu::reset();
        Self {
            break_points: HashMap::new(),
            watch_points: HashMap::new(),
            source_info: BTreeMap::new(),
            current_file: None,
            loader_start: 0,
            dis_line: String::new(),
            ticks: 0,
            stack_frames: Vec::new(),
            enable_stack_check: false,
            enable_mem_check: false,
            next_bp: None,
            load_name: String::new(),
            run_done: false,
            expr_context: DB65Context::new(),
            dbgdb: DebugData::new().unwrap(),
            seg_list: Vec::new(),
            source_mode: SourceDebugMode::None,
            call_intercepts: HashMap::new(),
            heap_blocks: HashMap::new(),
            privileged_mode: false,
            debug_mode: DebugMode::Unknown,
            file_table: HashMap::new(),
        }
    }
    pub fn delete_breakpoint(&mut self, id_opt: Option<&String>) -> Result<()> {
        if let Some(id) = id_opt {
            if let Ok(num) = id.parse::<usize>() {
                if let Some(find) = self.break_points.iter().find_map(|bp| {
                    if bp.1.number == num {
                        Some(*bp.0)
                    } else {
                        None
                    }
                }) {
                    self.break_points.remove(&find);
                }
            }
            // else lookup symbol?
        } else {
            self.break_points.clear();
        };
        Ok(())
    }
    pub fn set_break(&mut self, addr_str: &str, temp: bool) -> Result<()> {
        let (bp_addr, save_sym) = self.convert_addr(addr_str)?;
        self.break_points.insert(
            bp_addr,
            BreakPoint {
                addr: bp_addr,
                symbol: save_sym,
                number: self.break_points.len() + 1,
                temp,
            },
        );
        Ok(())
    }
    pub fn enable_stack_check(&mut self, enable: bool) {
        self.enable_stack_check = enable;
    }
    pub fn enable_mem_check(&mut self, enable: bool) {
        self.enable_mem_check = enable;
    }
    pub fn set_watch(&mut self, addr_str: &str, wt: WatchType) -> Result<()> {
        let (wp_addr, save_sym) = self.convert_addr(addr_str)?;
        self.watch_points.insert(
            wp_addr,
            WatchPoint {
                addr: wp_addr,
                symbol: save_sym,
                number: self.watch_points.len() + 1,
                watch: wt,
            },
        );
        Ok(())
    }

    pub fn next_statement(&mut self) -> Result<StopReason> {
        self.source_mode = SourceDebugMode::Next;
        self.execute(0)
    }
    pub fn step_statement(&mut self) -> Result<StopReason> {
        self.source_mode = SourceDebugMode::Step;
        self.execute(0)
    }
    pub fn find_source_line(&self, addr: u16) -> Result<Option<SourceInfo>> {
        self.dbgdb.find_source_line(addr)
    }
    pub fn get_addr_map(&self) -> &BTreeMap<u16, SourceInfo> {
        &self.source_info
    }
    pub fn get_source(&self, file: i64, from: i64, to: i64) -> Result<Vec<String>> {
        self.dbgdb.get_source(file, from, to)
    }
    fn init_shadow(&self) -> Result<()> {
        let shadow = Cpu::get_shadow();
        for seg in self.seg_list.iter().filter(|s| s.name != "EXEHDR") {
            const RW: u8 = SegmentType::ReadWrite as u8;
            const RO: u8 = SegmentType::ReadOnly as u8;
            const ZP: u8 = SegmentType::Zp as u8;
            const CODE: u8 = SegmentType::Code as u8;
            print!(
                "{} {} {:04x}-{:04x} {}",
                seg.id, seg.name, seg.start, seg.size, seg.seg_type
            );
            match seg.seg_type {
                RW => {
                    println!("rw");
                    for i in seg.start..(seg.start + seg.size) {
                        shadow[i as usize] |= ShadowFlags::READ | ShadowFlags::WRITE;
                    }
                }
                RO => {
                    println!("ro");
                    for i in seg.start..(seg.start + seg.size) {
                        shadow[i as usize] |= ShadowFlags::READ;
                    }
                }
                ZP => {
                    println!("zp");
                    for i in seg.start..(seg.start + seg.size) {
                        shadow[i as usize] |= ShadowFlags::READ | ShadowFlags::WRITE;
                    }
                }
                CODE => {
                    println!("code");
                    for i in seg.start..(seg.start + seg.size) {
                        shadow[i as usize] |= ShadowFlags::EXECUTE | ShadowFlags::READ;
                    }
                }
                _ => {
                    println!("unknown segment type {}", seg.seg_type);
                }
            }
        }
        // hardware stack
        for i in 0x100..0x200 {
            shadow[i] = ShadowFlags::READ | ShadowFlags::WRITE;
        }
        // sp65 stack
        for i in (0xfff0 - 0x800)..0xfff0 {
            shadow[i] = ShadowFlags::READ | ShadowFlags::WRITE;
        }
        Ok(())
    }
    pub fn load_dbg(&mut self, file: &Path) -> Result<()> {
        let fd = File::open(file)?;
        let mut reader = BufReader::new(fd);
        self.dbgdb.parse(&mut reader)?;
        self.dbgdb
            .load_expr_symbols(&mut self.expr_context.symbols)?;
        self.dbgdb.load_seg_list(&mut self.seg_list)?;
        self.dbgdb.load_all_cfiles()?;
        self.source_info.clear();
        self.dbgdb.load_all_source_files(&mut self.source_info)?;
        self.load_intercepts()?;
        self.init_shadow()?;
        self.dbgdb.load_files(&mut self.file_table)?;
        Ok(())
    }
    pub fn load_source(&mut self, file: &Path) -> Result<()> {
        self.dbgdb.load_source_file(file)?;
        Ok(())
    }

    pub fn get_segments(&self) -> &Vec<Segment> {
        &self.seg_list
    }
    pub fn get_dbg_symbols(&self, filter: Option<&String>) -> Result<Vec<Symbol>> {
        let s = self.dbgdb.get_symbols(filter)?;
        Ok(s)
    }
    pub fn load_code(&mut self, file: &Path) -> Result<(u16, u16)> {
        let (sp65_addr, run, _cpu, size) = loader::load_code(file)?;
        // println!("size={:x}, entry={:x}, cpu={}", size, run, cpu);
        Cpu::sp65_addr(sp65_addr);
        let arg0 = file.file_name().unwrap().to_str().unwrap().to_string();
        self.load_name = arg0;
        self.loader_start = run;

        Ok((size, run))
    }
    pub fn get_breaks(&self) -> Vec<u16> {
        self.break_points.iter().map(|bp| bp.1.addr).collect()
    }
    pub fn get_watches(&self) -> Vec<u16> {
        self.watch_points.iter().map(|wp| wp.1.addr).collect()
    }
    pub fn go(&mut self) -> Result<StopReason> {
        if !self.run_done {
            self.run(vec![])
        } else {
            self.execute(0) // 0 = forever
        }
    }
    pub fn finish(&mut self) -> Result<StopReason> {
        for i in (0..self.stack_frames.len()).rev() {
            if let FrameType::Jsr(_) = self.stack_frames[i].frame_type {
                self.stack_frames[i].stop_on_pop = true;
            }
        }
        self.execute(0) // 0 = forever
    }
    pub fn next(&mut self) -> Result<StopReason> {
        let next_inst = Cpu::read_byte(Cpu::read_pc());

        if next_inst == 0x20 {
            // if the next instruction is a jsr then]
            // set a temp bp on the folloing inst and run
            let inst = Cpu::read_pc() + 3;
            self.next_bp = Some(inst);
            self.execute(0)
        } else {
            // otherwise execute one instruction
            self.execute(1)
        }
    }
    pub fn get_bp(&self, addr: u16) -> Option<&BreakPoint> {
        return self.break_points.get(&addr);
    }
    pub fn get_watch(&self, addr: u16) -> Option<&WatchPoint> {
        return self.watch_points.get(&addr);
    }
    pub fn step(&mut self) -> Result<StopReason> {
        self.execute(1)
    }
    pub fn get_heap_blocks(&self) -> &HashMap<u16, HeapBlock> {
        &self.heap_blocks
    }
    pub fn run(&mut self, cmd_args: Vec<&String>) -> Result<StopReason> {
        Cpu::write_word(0xFFFC, self.loader_start);
        Cpu::reset();
        Cpu::push_arg(&self.load_name);
        for arg in &cmd_args {
            Cpu::push_arg(arg)
        }
        self.stack_frames.clear();
        self.heap_blocks.clear();

        self.run_done = true;
        self.execute(0) // 0 = forever
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

    // converts a string representing an address into an address
    // if string contains ':' then its a source line
    // if string starts with '$' or 0x it is a hex number
    // if digits it is a decimal number
    // else its a symbol

    pub fn convert_addr(&self, addr_str: &str) -> Result<(u16, String)> {
        // source line?

        if addr_str.contains(':') {
            let mut parts = addr_str.split(':');
            let file = parts.next().unwrap();
            let line = parts.next().unwrap();
            let file_info = self
                .lookup_file_by_name(file)
                .ok_or_else(|| anyhow::anyhow!("File '{}' not found", file))?;
            let line_no = line.parse::<i64>()?;
            if let Some(addr) = self
                .dbgdb
                .find_source_line_by_line_no(file_info.file_id, line_no)?
            {
                return Ok((addr.absaddr, addr_str.to_string()));
            }
            bail!("Source line not found");
        }

        // is this a hex number?
        if let Some(hex) = addr_str.strip_prefix('$').or_else(|| {
            addr_str
                .strip_prefix("0x")
                .or_else(|| addr_str.strip_prefix("0X"))
        }) {
            return Ok((u16::from_str_radix(hex, 16)?, String::new()));
        }

        // a decimal number?
        if addr_str.chars().next().unwrap().is_ascii_digit() {
            return Ok((addr_str.parse::<u16>()?, String::new()));
        }

        let syms = self.dbgdb.get_symbol(addr_str)?;
        match syms.len() {
            0 => bail!("Symbol '{}' not found", addr_str),
            1 => Ok((syms[0].1, addr_str.to_string())),
            _ => bail!("Symbol '{}' is ambiguous", addr_str),
        }
    }

    // reverse of convert_addr.
    // tried to find a symbol matching an address
    // if not found it returns a numberic string
    pub fn symbol_lookup(&self, addr: u16) -> Result<String> {
        // for (name, sym_addr) in &self.symbols {
        //     if *sym_addr == addr {
        //         return name.to_string();
        //     }
        // }

        if let Some(sym) = self.dbgdb.find_symbol(addr)? {
            return Ok(sym);
        }
        Ok(format!("${:04x}", addr))
    }
    pub fn zp_symbol_lookup(&self, addr: u8) -> Result<String> {
        if let Some(sym) = self.dbgdb.find_symbol(addr as u16)? {
            return Ok(sym);
        }

        Ok(format!("${:02x}", addr))
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
    #[allow(dead_code)]
    pub fn read_zr(&self) -> u8 {
        Cpu::read_zr()
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
    pub fn write_zr(&self, v: u8) {
        Cpu::write_zr(v);
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

    pub fn find_module(&self, addr: u16) -> Option<&SegChunk> {
        if self.seg_list.is_empty() {
            return None;
        }
        let mut current = 0;
        for i in 1..self.seg_list.len() {
            if self.seg_list[i].start > addr {
                break;
            }
            current = i;
        }
        let seg = &self.seg_list[current];
        if seg.modules.is_empty() {
            return None;
        }
        let addr = addr - seg.start;
        let mut current_mod = 0;
        for i in 1..seg.modules.len() {
            if seg.modules[i].offset > addr {
                break;
            }
            current_mod = i;
        }
        Some(&seg.modules[current_mod])
    }

    pub fn where_are_we(&self, addr: u16) -> Result<CodeLocation> {
        let mut location = CodeLocation {
            module: None,
            cfile: None,
            cline: 0,
            ctext: None,
            afile: None,
            aline: 0,
            seg: 0,
            offset: 0,
            absaddr: addr,
        };
        // do we have any debug data at all?
        if self.seg_list.is_empty() {
            return Ok(location);
        }

        // find the segment

        let seghit = self
            .seg_list
            .iter()
            .enumerate()
            .find(|s| s.1.start <= addr && s.1.start + s.1.size > addr);

        if seghit.is_none() {
            return Ok(location);
        }

        let current = seghit.unwrap().0;

        // find the module slice in the segment
        let seg = &self.seg_list[current];
        if seg.modules.is_empty() {
            return Ok(location);
        }

        let rel_addr = addr - seg.start;
        let mut current_mod = 0;
        for i in 0..seg.modules.len() {
            if seg.modules[i].offset > rel_addr {
                break;
            }
            current_mod = i;
        }

        // we have segment and module
        location.seg = seg.id;
        location.module = Some(seg.modules[current_mod].module);

        // now find the assembly line
        if let Some(aline) = self.dbgdb.find_assembly_line(addr)? {
            location.aline = aline.line_no;
            location.afile = Some(aline.file_id);
            location.offset = aline.addr;
            location.absaddr = addr;
            location.seg = aline.seg;
        }

        // now find the c line
        if let Some(cline) = self.dbgdb.find_c_line(addr)? {
            // this will always find something but it might not be the right one

            let mstart = seg.modules[current_mod].offset + seg.start;
            let mend = mstart + seg.modules[current_mod].size;
            if cline.absaddr >= mstart && cline.absaddr < mend {
                location.cline = cline.line_no;
                location.cfile = Some(cline.file_id);
                location.offset = cline.addr;
                location.absaddr = addr;
                location.seg = cline.seg;
                location.ctext = Some(cline.line);
            }
        }

        Ok(location)
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
}

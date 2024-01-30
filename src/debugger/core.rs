/*
The debugger core. It sits on top of the Cpu wrapper that has all the unsafe code
It does not do any ui. This allows for maybe a future gui to provide
the same functionality as the cli shell.

*/

use anyhow::{bail, Result};
use evalexpr::Value;

use std::cell::RefCell;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{
    collections::{BTreeMap, HashMap},
    fmt::Debug,
    fs::File,
    io::BufReader,
    path::Path,
};

use crate::say;
use crate::{
    db::debugdb::{DebugData, SourceInfo},
    debugger::cpu::{Cpu, ShadowFlags},
    debugger::execute::StopReason,
    debugger::loader,
};

pub enum SourceDebugMode {
    None,
    Next,
    Step,
}

pub struct HLSym {
    pub name: String,
    pub value: i64,
    pub type_: String,
    pub seg: u8,
    pub scope: i64,
}

#[derive(Debug, Default)]
pub struct CodeLocation {
    pub module: Option<i32>,
    pub cfile: Option<i64>,
    pub cline: i64,
    pub ctext: Option<String>,
    pub afile: Option<i64>,
    pub aline: i64,
    pub atext: Option<String>,
    pub seg: u8,
    pub offset: u16,
    pub absaddr: u16,
    pub scope: Option<i64>,
    pub parent: String,
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
type InterceptFunc = fn(&mut Debugger, bool) -> Result<Option<StopReason>>;
pub struct Debugger {
    pub(crate) break_points: BTreeMap<u16, BreakPoint>,
    pub(crate) watch_points: BTreeMap<u16, WatchPoint>,
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
    pub(crate) enable_heap_check: bool,
    pub(crate) load_name: String,
    pub(crate) run_done: bool,
    pub(crate) dbgdb: DebugData,
    pub(crate) seg_list: Vec<Segment>,
    pub(crate) heap_blocks: HashMap<u16, HeapBlock>,
    pub(crate) privileged_mode: bool,

    pub(crate) regbank_addr: Option<u16>,
    pub(crate) regbank_size: Option<u16>,
    pub(crate) ctrlc: Arc<AtomicBool>,
    pub(crate) expr_value: RefCell<evalexpr::Value>,
    pub(crate) dbg_suffix: String,
    pub(crate) dbg_file: Option<PathBuf>,
}

pub struct HeapBlock {
    pub addr: u16,
    pub size: u16,
    pub alloc_addr: u16,
    pub realloc_size: Option<u16>,
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
pub struct JsrData {
    pub dest_addr: u16,
    pub call_addr: u16,
    pub sp: u8,
    pub sp65: u16,
}
#[derive(Debug)]
pub struct PushData {
    pub addr: u16,
    pub sp: u8,
    pub value: u8,
}
#[derive(Debug)]
pub(crate) enum FrameType {
    Jsr(JsrData), // addr, return addr,sp,sp65
    Pha(PushData),
    Php(PushData),
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
    pub(crate) temp: bool,
}
#[derive(Debug, Clone)]
pub enum WatchType {
    Read,
    Write,
    ReadWrite,
}
#[derive(Debug, Clone)]
pub struct WatchPoint {
    pub(crate) addr: u16,
    pub(crate) symbol: String,
    pub(crate) watch: WatchType,
}
impl Debugger {
    pub fn new() -> Self {
        Cpu::reset();
        let s = Self {
            break_points: BTreeMap::new(),
            watch_points: BTreeMap::new(),
            source_info: BTreeMap::new(),
            current_file: None,
            loader_start: 0,
            dis_line: String::new(),
            ticks: 0,
            stack_frames: Vec::new(),
            enable_stack_check: false,
            enable_mem_check: false,
            enable_heap_check: false,
            next_bp: None,
            load_name: String::new(),
            run_done: false,
            dbgdb: DebugData::new(".db65.db").unwrap(),
            seg_list: Vec::new(),
            source_mode: SourceDebugMode::None,
            call_intercepts: HashMap::new(),
            heap_blocks: HashMap::new(),
            privileged_mode: false,

            regbank_addr: None,
            regbank_size: None,
            ctrlc: Arc::new(AtomicBool::new(false)),
            expr_value: RefCell::new(Value::Int(0)),
            dbg_suffix: String::from(".dbg"),
            dbg_file: None,
        };
        let ctrlc = s.ctrlc.clone();
        ctrlc::set_handler(move || {
            ctrlc.store(true, Ordering::SeqCst);
        })
        .expect("Error setting Ctrl-C handler");
        s
    }
    pub fn delete_breakpoint(&mut self, id_opt: Option<&String>) -> Result<()> {
        if let Some(id) = id_opt {
            if let Ok(num) = id.parse::<usize>() {
                if let Some(find) = self.break_points.iter().map(|e| *e.0).nth(num - 1) {
                    self.break_points.remove(&find);
                }
            }
            // else lookup symbol?
        } else {
            self.break_points.clear();
        };
        Ok(())
    }
    pub fn delete_watchpoint(&mut self, id_opt: Option<&String>) -> Result<()> {
        if let Some(id) = id_opt {
            if let Ok(num) = id.parse::<usize>() {
                if let Some(find) = self.watch_points.iter().map(|e| *e.0).nth(num - 1) {
                    self.watch_points.remove(&find);
                }
            }
            // else lookup symbol?
        } else {
            self.watch_points.clear();
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
                temp,
            },
        );
        Ok(())
    }

    pub fn set_watch(&mut self, addr_str: &str, wt: WatchType) -> Result<()> {
        let (wp_addr, save_sym) = self.convert_addr(addr_str)?;
        self.watch_points.insert(
            wp_addr,
            WatchPoint {
                addr: wp_addr,
                symbol: save_sym,
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

    // because the intention is clearer my way :-)
    #[allow(clippy::needless_range_loop)]
    fn init_shadow(&self) -> Result<()> {
        let shadow = Cpu::get_shadow();
        for seg in self.seg_list.iter().filter(|s| s.name != "EXEHDR") {
            const RW: u8 = SegmentType::ReadWrite as u8;
            const RO: u8 = SegmentType::ReadOnly as u8;
            const ZP: u8 = SegmentType::Zp as u8;
            const CODE: u8 = SegmentType::Code as u8;

            match seg.seg_type {
                RW => {
                    for i in seg.start..(seg.start + seg.size) {
                        shadow[i as usize] |= ShadowFlags::READ | ShadowFlags::WRITE;
                    }
                }
                RO => {
                    for i in seg.start..(seg.start + seg.size) {
                        shadow[i as usize] |= ShadowFlags::READ;
                    }
                }
                ZP => {
                    for i in seg.start..(seg.start + seg.size) {
                        shadow[i as usize] |= ShadowFlags::READ | ShadowFlags::WRITE;
                    }
                }
                CODE => {
                    for i in seg.start..(seg.start + seg.size) {
                        shadow[i as usize] |= ShadowFlags::EXECUTE | ShadowFlags::READ;
                    }
                }
                _ => {
                    bail!("unknown segment type {}", seg.seg_type);
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
        self.dbgdb.clear()?;
        self.dbgdb.parse(&mut reader)?;

        self.dbgdb.load_seg_list(&mut self.seg_list)?;
        self.dbgdb.load_all_cfiles()?;
        self.source_info.clear();
        self.dbgdb.load_all_source_files(&mut self.source_info)?;

        self.load_intercepts()?;
        self.init_shadow()?;
        //   self.dbgdb.load_files(&mut self.file_table)?;

        let regbank = self.dbgdb.get_symbol("zeropage.regbank")?;
        if !regbank.is_empty() {
            self.regbank_addr = Some(regbank[0].1);
        }
        let regbanksize = self.dbgdb.get_symbol("regbanksize")?;
        if !regbanksize.is_empty() {
            self.regbank_size = Some(regbanksize[0].1);
        }
        self.dbg_file = Some(file.to_path_buf());
        self.enable_heap_check = true;
        self.enable_mem_check = true;
        self.enable_stack_check = true;
        Ok(())
    }

    fn reset(&mut self) {
        self.stack_frames.clear();
        self.heap_blocks.clear();
        self.run_done = false;
        self.next_bp = None;
        self.source_mode = SourceDebugMode::None;
        self.ticks = 0;
        Cpu::reset();
    }
    pub fn load_code(&mut self, file: &Path) -> Result<(u16, u16)> {
        self.reset();
        let (sp65_addr, run, _cpu, size) = loader::load_code(file)?;

        Cpu::sp65_addr(sp65_addr);
        let arg0 = file.file_name().unwrap().to_str().unwrap().to_string();
        self.load_name = arg0;
        self.loader_start = run;

        if let Some(prefix) = file.file_stem() {
            let prefix = prefix.to_str().unwrap();
            let mut path = file.to_path_buf();
            path.pop();
            path.push(format!("{}{}", prefix, self.dbg_suffix));

            if path.exists() {
                say!("Loading debug info from {:?}", path);
                self.load_dbg(&path)?;
            }
        }

        Ok((size, run))
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
            // set a temp bp on the following inst and run
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

        // a c symbol? Only meaningful when running
        // because the c symbols we support depend on the
        // stack frame we are in

        if self.run_done {
            if let Some(caddr) = self.find_csym_address(addr_str)? {
                return Ok((caddr, addr_str.to_string()));
            }
        }

        // a regular symbol?
        let syms = self.dbgdb.get_symbol(addr_str)?;
        match syms.len() {
            0 => bail!("Symbol '{}' not found", addr_str),
            1 => Ok((syms[0].1, addr_str.to_string())),
            _ => bail!("Symbol '{}' is ambiguous", addr_str),
        }
    }

    // reverse of convert_addr.
    // tried to find a symbol matching an address
    // if not found it returns a numeric string
    // preference is for a label
    pub fn symbol_lookup(&self, addr: u16) -> Result<String> {
        let syms = self.dbgdb.find_symbol_by_addr(addr)?;

        // look for lab first
        if let Some(sym) = syms.iter().find(|s| s.sym_type == SymbolType::Label) {
            return Ok(sym.name.clone());
        }

        // no - take first matching eq

        if !syms.is_empty() {
            return Ok(syms[0].name.clone());
        }
        Ok(format!("${:04x}", addr))
    }
    // same for zero page
    pub fn zp_symbol_lookup(&self, addr: u8) -> Result<String> {
        let syms = self.dbgdb.find_symbol_by_addr(addr as u16)?;
        if let Some(sym) = syms.iter().find(|s| s.sym_type == SymbolType::Label) {
            return Ok(sym.name.clone());
        }
        if !syms.is_empty() {
            return Ok(syms[0].name.clone());
        }

        Ok(format!("${:02x}", addr))
    }

    pub fn _find_module(&self, addr: u16) -> Option<&SegChunk> {
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
    pub fn _find_csym(&self, name: &str, scope: i64) -> Result<Option<HLSym>> {
        self.dbgdb.find_csym(name, scope)
    }

    pub fn where_are_we(&self, addr: u16) -> Result<CodeLocation> {
        // given an address find out where we are
        // finds seg, module, assembly line and c line
        // if no debug data is available it returns an empty location

        let mut location = CodeLocation {
            module: None,
            cfile: None,
            cline: 0,
            ctext: None,
            afile: None,
            aline: 0,
            atext: None,
            seg: 0,
            offset: 0,
            absaddr: addr,
            scope: None,
            parent: String::new(),
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

        // find scope
        location.scope = self.dbgdb.find_scope(seg.id as i64, rel_addr)?;

        // find parent symbol

        location.parent = if let Some(parent) = self.find_parent_symbol(addr)? {
            let off = parent.1;
            if off > 0 {
                format!("{}+0x{:x}:", parent.0, off)
            } else {
                parent.0.to_string()
            }
        } else {
            format!("0x{:04x}", addr)
        };

        // now find the assembly line
        if let Some(aline) = self.dbgdb.find_assembly_line(addr)? {
            location.aline = aline.line_no;
            location.afile = Some(aline.file_id);
            location.offset = aline.addr;
            location.absaddr = addr;
            location.seg = aline.seg;
            location.atext = Some(aline.line);
        }

        // now find the c line
        if let Some(cline) = self.dbgdb.find_c_line(addr)? {
            // this will always find something but it might not be the right one
            // so check in bounds of module
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

    fn find_parent_symbol(&self, addr: u16) -> Result<Option<(String, u16)>> {
        // tries to find the module + offset for a code address
        for seg in self.seg_list.iter() {
            if seg.start <= addr && seg.start + seg.size > addr {
                for module in seg.modules.iter() {
                    if module.offset + seg.start <= addr
                        && module.offset + seg.start + module.size > addr
                    {
                        return Ok(Some((
                            module.module_name.clone(),
                            addr - module.offset - seg.start,
                        )));
                    }
                }
            }
        }
        Ok(None)
    }
    pub fn find_csym_address(&self, name: &str) -> Result<Option<u16>> {
        let addr = self.read_pc();
        let waw = self.where_are_we(addr)?;
        if let Some(scope) = waw.scope {
            if let Some(csym) = self.dbgdb.find_csym(name, scope)? {
                match csym.type_.as_str() {
                    "auto" => {
                        //let stack = self.debugger.read_stack();
                        let mut sp65 = 0;
                        for i in (0..self.stack_frames.len()).rev() {
                            if let FrameType::Jsr(jsr) = &self.stack_frames[i].frame_type {
                                sp65 = jsr.sp65;
                                if i == 0 {
                                    // the call main stack frame is out by 4 (argc,argv)
                                    sp65 -= 4;
                                }
                                break;
                            }
                        }

                        return Ok(Some((sp65 as i64 + csym.value) as u16));
                    }
                    "reg" => {
                        if let Some(regbank) = self.regbank_addr {
                            return Ok(Some((regbank as i64 + csym.value) as u16));
                        }
                    }
                    "static" => {}
                    _ => {
                        return Ok(None);
                    }
                }
            };
        }
        Ok(None)
    }
}

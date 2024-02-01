#![allow(clippy::uninlined_format_args)]
use crate::about::About;
use crate::debugger::core::{CodeLocation, Debugger, FrameType::*, SymbolType, WatchType};
use crate::debugger::cpu::Status;
use crate::debugger::execute::{BugType, StopReason};

use crate::syntax;
use anyhow::{anyhow, bail, Result};
//use clap::error::ErrorKind;
use clap::ArgMatches;

use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::collections::VecDeque;
use std::fs::File;
use std::io::Read;

use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
/*

TODO list

trace command
log / trace in code
dbiginfo for module
load bin from command line
stack write check
write bugcheck for locals

*/
#[derive(Debug, Eq, PartialEq, PartialOrd)]
enum SourceMode {
    C = 3,
    Asm = 2,
    Raw = 1,
}
pub struct Shell {
    debugger: Debugger,
    current_dis_addr: u16,
    _current_mem_addr: u16,
    waw: CodeLocation,
    about: About,
    number_of_lines: u8,
    source_mode: SourceMode,
    always_reg_dis: bool,
    current_file: Option<i64>,
}
static VERBOSE: AtomicBool = AtomicBool::new(false);
static SHELL_HISTORY_FILE: &str = ".db65_history";
impl Shell {
    pub fn new() -> Self {
        Self {
            debugger: Debugger::new(),
            current_dis_addr: 0,
            _current_mem_addr: 0,
            waw: CodeLocation::default(),
            about: About::new(),
            number_of_lines: 10,
            source_mode: SourceMode::C,
            always_reg_dis: false,
            current_file: None,
        }
    }
    fn say(s: &str, v: bool) {
        if !v || v && VERBOSE.load(std::sync::atomic::Ordering::SeqCst) {
            println!("{s}")
        };
    }
    pub fn shell(&mut self, file: Option<PathBuf>, _args: &[String]) -> Result<u8> {
        let mut rl = DefaultEditor::new()?;
        crate::log::set_say_cb(Self::say);
        if let Err(e) = rl.load_history(SHELL_HISTORY_FILE) {
            if let ReadlineError::Io(ref re) = e {
                if re.kind() != std::io::ErrorKind::NotFound {
                    println!("cannot open history {:?}", e);
                }
            } else {
                println!("cannot open history {:?}", e);
            }
        }

        //do we have a command file to run?
        if let Some(f) = file {
            let mut fd = File::open(f)?;
            let mut commstr = String::new();
            fd.read_to_string(&mut commstr)?;
            let mut commands: VecDeque<String> = commstr
                .split('\n')
                .map(std::string::ToString::to_string)
                .collect();
            loop {
                let line = commands.pop_front();
                if line.is_none() {
                    break;
                }
                match self.dispatch(&line.unwrap()) {
                    Err(e) => println!("{}", e),
                    Ok(true) => return Ok(0),
                    Ok(false) => {}
                }
            }
        }
        // remeber the last line, replay it if user hits enter
        // maybe need more sophisticated mechanism. ie
        // 'dis x' followed by enter should do plain 'dis'
        let mut lastinput = String::new();

        // main shell loop
        // readline, dispatch it, repeat
        loop {
            let readline = rl.readline(">> ");

            match readline {
                Ok(mut line) => {
                    if line.is_empty() {
                        line = lastinput.clone();
                    } else {
                        rl.add_history_entry(line.as_str())?;
                        lastinput = line.clone();
                    };
                    match self.dispatch(&line) {
                        Err(e) => {
                            if let Some(original_error) = e.downcast_ref::<clap::error::Error>() {
                                println!("{}", original_error);
                            } else {
                                if e.backtrace().status()
                                    == std::backtrace::BacktraceStatus::Captured
                                {
                                    println!("{} {}", e, e.backtrace());
                                } else {
                                    println!("{}", e);
                                }
                            }
                        }

                        Ok(true) => break, // quit was typed
                        Ok(false) => {}    // continue
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    println!();
                }
                Err(ReadlineError::Eof) => {
                    println!("quit"); // treat eof as quit
                    break;
                }
                Err(err) => {
                    println!("Error: {:?}", err);
                    break;
                }
            }
        }

        let _ = rl.save_history(SHELL_HISTORY_FILE);
        Ok(0)
    }

    fn dispatch(&mut self, line: &str) -> Result<bool> {
        // split the line up into args
        let args = shlex::split(line).ok_or(anyhow!("error: Invalid quoting"))?;
        // parse with clap
        let matches = syntax::syntax().try_get_matches_from(args)?;
        // execute the command
        match matches.subcommand() {
            Some(("break", args)) => {
                let addr = args.get_one::<String>("address").unwrap();
                let addr = &self.expand_expr(addr)?;

                if addr.chars().next().unwrap() == ':' && self.current_file.is_some() {
                    if let Some(name) = self.debugger.lookup_file_by_id(self.current_file.unwrap())
                    {
                        let addr = format!("{}{}", name.short_name, addr);
                        self.debugger.set_break(&addr, false)?;
                    }
                } else {
                    self.debugger.set_break(addr, false)?;
                }
            }
            Some(("watch", args)) => {
                let addr = args.get_one::<String>("address").unwrap();
                let addr = &self.expand_expr(addr)?;
                let read = *args.get_one::<bool>("read").unwrap();
                let write = *args.get_one::<bool>("write").unwrap();
                let rw = if read && write {
                    WatchType::ReadWrite
                } else if read {
                    WatchType::Read
                } else if write {
                    WatchType::Write
                } else {
                    bail!("must specify -r or -w")
                };
                self.debugger.set_watch(addr, rw)?;
            }
            Some(("list_breakpoints", _)) => {
                let blist = self.debugger.get_breaks()?;

                for (i, bp) in blist.values().enumerate() {
                    println!("#{} 0x{:04X} ({})", i + 1, bp.addr, bp.symbol);
                }
            }
            Some(("list_watchpoints", _)) => {
                let wlist = self.debugger.get_watches()?;

                for (i, wp) in wlist.values().enumerate() {
                    println!(
                        "#{} 0x{:04X} ({}) {:?}",
                        i + 1,
                        wp.addr,
                        wp.symbol,
                        wp.watch
                    );
                }
            }
            Some(("load_dbginfo", args)) => {
                let file = args.get_one::<String>("file").unwrap();
                self.debugger.load_dbg(Path::new(file))?;
            }
            Some(("list_symbols", args)) => {
                let mtch = args.get_one::<String>("match");
                let symbols = self.debugger.get_dbg_symbols(mtch)?;
                for symbol in symbols {
                    let symt = match symbol.sym_type {
                        SymbolType::Equate => "equ",
                        SymbolType::Label => "lab",
                        SymbolType::CSymbol => "c",
                        _ => "???",
                    };
                    println!(
                        "0x{:04x} [{}.]{} ({})",
                        symbol.value, symbol.module, symbol.name, symt
                    );
                }
            }
            Some(("load_code", args)) => {
                let file = args.get_one::<String>("file").unwrap();
                self.debugger.load_code(Path::new(file))?;
            }

            Some(("quit", _)) => {
                println!("quit");
                return Ok(true);
            }

            Some(("display_memory", args)) => {
                let addr_str = args.get_one::<String>("address").unwrap();
                let addr_str = &self.expand_expr(addr_str)?;
                let (addr, _) = self.debugger.convert_addr(addr_str)?;
                let len = self.number_of_lines as u16 * 16;
                let chunk = self.debugger.get_chunk(addr, len)?;
                self.mem_dump(addr, &chunk);
            }

            Some(("run", args)) => {
                let cmd_args = args
                    .get_many::<String>("args")
                    .map(Iterator::collect)
                    .unwrap_or_default();

                let reason = self.debugger.run(cmd_args)?;
                self.stop(reason)?;
            }

            Some(("go", _)) => {
                if !self.debugger.run_done {
                    bail!("program not running");
                };
                let reason = self.debugger.go()?;
                self.stop(reason)?;
            }

            Some(("next_instruction", _)) => {
                if !self.debugger.run_done {
                    bail!("program not running");
                };
                let reason = self.debugger.next()?;
                self.stop(reason)?;
            }

            Some(("step_instruction", _)) => {
                if !self.debugger.run_done {
                    bail!("program not running");
                };
                let reason = self.debugger.step()?;
                self.stop(reason)?;
            }

            Some(("delete_breakpoint", args)) => {
                let id = args.get_one::<String>("id");
                self.debugger.delete_breakpoint(id)?;
            }
            Some(("delete_watchpoint", args)) => {
                let id = args.get_one::<String>("id");
                self.debugger.delete_watchpoint(id)?;
            }
            Some(("back_trace", _)) => {
                let stack = self.debugger.read_stack();
                print!("0x{:04x} ", self.waw.absaddr);
                if !self.print_code_line(&self.waw)? {
                    println!("{}", self.waw.parent);
                }
                for i in (0..stack.len()).rev() {
                    let frame = &stack[i];
                    match &frame.frame_type {
                        Jsr(jd) => {
                            let waw = self.debugger.where_are_we(jd.call_addr)?;
                            print!("0x{:04x} ", jd.call_addr);
                            match (&waw.cfile, &waw.afile) {
                                (Some(cf), _) if self.source_mode == SourceMode::C => {
                                    let file_name = self.debugger.lookup_file_by_id(*cf).unwrap();
                                    println!(
                                        "{}:{}\t\t{}",
                                        file_name.short_name,
                                        waw.cline,
                                        waw.ctext.as_ref().unwrap()
                                    );
                                }
                                (_, Some(af)) if self.source_mode > SourceMode::Raw => {
                                    let file_name = self.debugger.lookup_file_by_id(*af).unwrap();
                                    println!(
                                        "{}:{}\t\t{}",
                                        file_name.short_name,
                                        waw.aline,
                                        waw.atext.as_ref().unwrap_or(&"".to_string())
                                    );
                                }
                                _ => {
                                    println!(
                                        "{} \t\tjsr {} ",
                                        waw.parent,
                                        self.debugger.symbol_lookup(jd.dest_addr)?,
                                    );
                                }
                            }
                        }
                        Pha(pd) => println!(
                            "pha ${:02x} @{}",
                            pd.value,
                            self.debugger.symbol_lookup(pd.addr)?
                        ),
                        Php(pd) => println!(
                            "php ${:02x} @{}",
                            pd.value,
                            self.debugger.symbol_lookup(pd.addr)?
                        ),
                    }
                }
            }

            Some(("dis", args)) => {
                let mut addr = if let Some(addr_str) = args.get_one::<String>("address") {
                    let addr_str = self.expand_expr(addr_str)?;
                    self.debugger.convert_addr(&addr_str)?.0
                } else {
                    let mut a = self.current_dis_addr;
                    if a == 0 {
                        a = self.debugger.read_pc();
                    }

                    a
                };
                self.current_dis_addr = addr;
                for _i in 0..self.number_of_lines {
                    let chunk = self.debugger.get_chunk(addr, 3)?;
                    if chunk.len() < 3 {
                        // we ran off the end of memory
                        break;
                    }
                    let delta = self.debugger.dis(&chunk, addr);
                    let addr_str = self.debugger.symbol_lookup(addr)?;
                    if !addr_str.starts_with('$') {
                        println!("{}:", addr_str);
                    }
                    println!("{:04x}:       {}", addr, self.debugger.dis_line);
                    addr += delta as u16;
                    self.current_dis_addr = addr;
                }
            }
            Some(("print", args)) => {
                let addr_str = args.get_one::<String>("address").unwrap();

                let addr_str = self.expand_expr(addr_str)?;
                let (addr, _) = self.debugger.convert_addr(&addr_str)?;
                self.print(addr, args)?;
            }

            Some(("expr", args)) => {
                let expr = args.get_one::<String>("expression").unwrap();
                let ans = self.expand_expr(expr)?;
                println!("{:}", ans);
            }

            Some(("dbginfo", args)) => {
                if *args.get_one::<bool>("segments").unwrap() {
                    let segs = self.debugger.get_segments();
                    for seg in segs {
                        println!("{:15} 0x{:04x} size:{}", seg.name, seg.start, seg.size);
                    }
                } else if let Some(arg) = args.get_one::<String>("segment") {
                    let segs = self.debugger.get_segments();
                    if let Some(seg) = segs.iter().find(|s| s.name.as_str() == arg) {
                        for chunk in seg.modules.iter() {
                            println!(
                                "{:15} 0x{:04x} = 0x{:04x}",
                                chunk.module_name,
                                chunk.offset,
                                seg.start + chunk.offset
                            );
                        }
                    } else {
                        bail!("unknown segment {}", arg);
                    }
                } else if *args.get_one::<bool>("address_map").unwrap() {
                    let map = self.debugger.get_addr_map();
                    for (addr, info) in map {
                        //let waw = self.debugger.where_are_we(addr)?;
                        let file_name = self.debugger.lookup_file_by_id(info.file_id);
                        println!(
                            "0x{:04x} {}:{}",
                            addr,
                            file_name.map_or("default", |f| &f.short_name),
                            info.line_no
                        );
                    }
                }
            }
            Some(("finish", _)) => {
                if !self.debugger.run_done {
                    bail!("program not running");
                };
                let reason = self.debugger.finish()?;
                self.stop(reason)?;
            }
            Some(("reg", args)) => {
                let regname = args.get_one::<String>("register").unwrap();
                let value_str = args.get_one::<String>("value").unwrap();
                let value_str = self.expand_expr(value_str)?;
                let value = self.debugger.convert_addr(&value_str)?.0;
                match regname.as_str() {
                    "ac" => self.debugger.write_ac(value as u8),
                    "xr" => self.debugger.write_xr(value as u8),
                    "yr" => self.debugger.write_yr(value as u8),
                    "sp" => self.debugger.write_sp(value as u8),
                    "sr" => self.debugger.write_sr(value as u8),
                    "pc" => self.debugger.write_pc(value),
                    _ => bail!("unknown register {}", regname),
                }
            }
            Some(("write_memory", args)) => {
                let addr_str = args.get_one::<String>("address").unwrap();
                let addr_str = self.expand_expr(addr_str)?;
                let addr = self.debugger.convert_addr(&addr_str)?.0;

                let value_str = args.get_one::<String>("value").unwrap();
                let value_str = self.expand_expr(value_str)?;
                let value = self.debugger.convert_addr(&value_str)?.0;

                self.debugger.write_byte(addr, value as u8);
            }
            Some(("next_statement", _)) => {
                if !self.debugger.run_done {
                    bail!("program not running");
                };
                let reason = self.debugger.next_statement()?;
                self.stop(reason)?;
            }
            Some(("step_statement", _)) => {
                if !self.debugger.run_done {
                    bail!("program not running");
                };
                let reason = self.debugger.step_statement()?;
                self.stop(reason)?;
            }
            Some(("list_source", args)) => {
                let addr = args.get_one::<String>("address");
                if let Some(add) = addr {
                    let (filename, start) = if add.contains(':') {
                        let mut parts = add.split(':');
                        let file = parts.next().unwrap();
                        let line = parts.next().unwrap();
                        let start = line.parse::<i64>().unwrap();
                        (file, start)
                    } else {
                        (add.as_str(), 1)
                    };
                    let fileid = self
                        .debugger
                        .lookup_file_by_name(filename)
                        .ok_or(anyhow!("Unknown source file {}", filename))?;

                    let source = self.debugger.get_source(
                        fileid.file_id,
                        start,
                        start + self.number_of_lines as i64,
                    )?;
                    for (i, s) in source.iter().enumerate() {
                        println!("{}:{}\t\t{}", filename, i + start as usize, s);
                    }
                    self.current_file = Some(fileid.file_id);
                } else {
                    let (fileid, from) = if let Some(cf) = &self.waw.cfile {
                        (*cf, self.waw.cline)
                    } else if let Some(af) = &self.waw.afile {
                        (*af, self.waw.aline)
                    } else {
                        (-1, -1)
                    };
                    let source = self.debugger.get_source(
                        fileid,
                        from,
                        from + self.number_of_lines as i64,
                    )?;
                    let file_name = self
                        .debugger
                        .lookup_file_by_id(fileid)
                        .ok_or_else(|| anyhow!("no source"))?;
                    for (i, s) in source.iter().enumerate() {
                        println!("{}:{}\t\t{}", file_name.short_name, i + from as usize, s);
                    }
                    self.current_file = Some(fileid);
                }
            }
            Some(("about", args)) => {
                let topic = args.get_one::<String>("topic");
                if let Some(t) = topic {
                    println!("{}", self.about.get_topic(t.as_str()));
                } else {
                    println!("{}", self.about.get_topic("topics"));
                }
            }
            Some(("display_heap", _args)) => {
                let heap = self.debugger.get_heap_blocks();
                for (addr, hb) in heap {
                    let waw = self.debugger.where_are_we(hb.alloc_addr)?;
                    if let Some(cf) = waw.cfile {
                        let file_name = self.debugger.lookup_file_by_id(cf).unwrap();
                        println!(
                            "0x{:04x} size {} allocated at 0x{:04x} = {}:{}",
                            addr, hb.size, hb.alloc_addr, file_name.short_name, waw.cline
                        );
                    } else {
                        println!(
                            "0x{:04x} size {} allocated at 0x{:04x} ",
                            addr, hb.size, hb.alloc_addr
                        );
                    }
                }
            }
            Some(("status", _)) => {
                if !self.debugger.load_name.is_empty() {
                    println!("Loaded code: {}", self.debugger.load_name);
                }
                if let Some(dbgfile) = &self.debugger.dbg_file {
                    println!("Loaded dbginfo: {}", dbgfile.display());
                }

                println!("Settings:");
                println!("  lines: {}", self.number_of_lines);
                println!("  source_mode: {:?}", self.source_mode);
                println!("  source_tree: {}", self.debugger.get_cc65_dir().display());
                println!("  dbg suffix: {}", self.debugger.dbg_suffix);
                println!(
                    "  traps: {}",
                    if self.debugger.enable_heap_check {
                        "On"
                    } else {
                        "Off"
                    }
                );
                println!(
                    "  verbose: {}",
                    VERBOSE.load(std::sync::atomic::Ordering::SeqCst)
                );
                println!("  register: {}", self.always_reg_dis);
            }
            Some(("settings", args)) => {
                if let Some(cc65_dir) = args.get_one::<PathBuf>("source_tree") {
                    self.debugger.set_cc65_dir(cc65_dir)?;
                }
                if let Some(number) = args.get_one("lines") {
                    self.number_of_lines = *number;
                }
                if let Some(assm) = args.get_one::<String>("source_mode") {
                    match assm.as_str() {
                        "c" => self.source_mode = SourceMode::C,
                        "asm" => self.source_mode = SourceMode::Asm,
                        "raw" => self.source_mode = SourceMode::Raw,
                        _ => unreachable!(),
                    };
                    // forces the redisplay of state
                    self.stop(StopReason::None)?;
                }
                if let Some(dbgsuffix) = args.get_one::<String>("dbgfile") {
                    self.debugger.set_dbgfile_suffix(dbgsuffix.as_str());
                }
                if let Some(t) = args.get_one("traps") {
                    self.debugger.enable_heap_check(*t);
                    self.debugger.enable_stack_check(*t);
                    self.debugger.enable_mem_check(*t);
                }
                if let Some(t) = args.get_one::<bool>("verbose") {
                    VERBOSE.store(*t, std::sync::atomic::Ordering::SeqCst);
                }
                if let Some(t) = args.get_one::<bool>("regdis") {
                    self.always_reg_dis = *t;
                }
            }

            Some((name, _matches)) => unimplemented!("{name}"),
            None => unreachable!("subcommand required"),
        }

        Ok(false)
    }
    fn expand_expr(&mut self, exp: &str) -> Result<String> {
        if let Some(exp) = exp.strip_prefix('=') {
            let res = self.debugger.evaluate(exp)?;
            Ok(format!("${:x}", res))
        } else {
            Ok(exp.to_string())
        }
    }
    fn print(&self, addr: u16, args: &ArgMatches) -> Result<()> {
        if *args.get_one::<bool>("asstring").unwrap() {
            let mut addr = addr;
            loop {
                let chunk = self.debugger.get_chunk(addr, 1)?;
                if chunk[0] == 0 {
                    break;
                }
                print!("{}", chunk[0] as char);
                addr += 1;
            }
            println!();
        } else if *args.get_one::<bool>("aspointer").unwrap() {
            let chunk = self.debugger.get_chunk(addr, 2)?;
            println!("0x{:02x}{:02x} ", chunk[1], chunk[0]);
        } else {
            // asint

            let lo = self.debugger.get_chunk(addr, 1)?;
            let hi = self.debugger.get_chunk(addr + 1, 1)?;
            println!("{} ", lo[0] as u16 | ((hi[0] as u16) << 8));
        }

        Ok(())
    }
    fn mem_dump(&mut self, mut addr: u16, chunk: &[u8]) {
        // pretty memory dump
        let mut line = String::new();
        for (i, byte) in chunk.iter().enumerate() {
            if i % 16 == 0 {
                if i > 0 {
                    println!("{}", line);
                    line.clear();
                }
                print!("{:04x}: ", addr);
            }
            print!("{:02x} ", byte);
            if *byte >= 32 && *byte < 127 {
                line.push(*byte as char);
            } else {
                line.push('.');
            }
            addr += 1;
        }
        println!("{}", line);
    }
    fn stop(&mut self, reason: StopReason) -> Result<()> {
        // common handler for when execution is interrupted

        // first of all explain why we stopped

        match reason {
            StopReason::BreakPoint(bp_addr) => {
                let bp = self.debugger.get_bp(bp_addr).unwrap();
                let bnum = self
                    .debugger
                    .get_breaks()?
                    .iter()
                    .take_while(|b| b.1.addr != bp_addr)
                    .count()
                    + 1;
                println!("bp #{} {}", bnum, bp.symbol);
            }
            StopReason::Exit(_) => {
                println!("Exit");
                let heap = self.debugger.get_heap_blocks();
                for (addr, hb) in heap {
                    println!(
                        "Heap block 0x{:04x} size {} leaked at {:04x}",
                        addr, hb.size, hb.alloc_addr
                    );
                }
                return Ok(());
            }
            StopReason::Count | StopReason::Next => {}
            StopReason::Bug(bug) => match bug {
                BugType::SpMismatch => {
                    println!("Stack pointer mismatch");
                }
                BugType::Memcheck(addr) => {
                    println!("Unitialized memory read -> ${:04x}", addr);
                }
                BugType::HeapCheck => {
                    println!("Heap check failed");
                }
                BugType::SegCheck(addr) => {
                    println!("Seg read/write violation -> ${:04x}", addr);
                }
            },
            StopReason::WatchPoint(addr) => {
                let wp = self.debugger.get_watch(addr).unwrap();
                let wnum = self
                    .debugger
                    .get_watches()?
                    .iter()
                    .take_while(|w| w.1.addr != addr)
                    .count()
                    + 1;
                println!("Watch #{} 0x{:04x} ({}) ", wnum, wp.addr, wp.symbol);
            }
            StopReason::Finish => {
                println!("Finish");
            }
            StopReason::Ctrlc => {
                println!("Ctrl-c break");
            }
            StopReason::None => {}
        }

        // now display where we are
        let inst_addr = self.debugger.read_pc();
        self.waw = self.debugger.where_are_we(inst_addr)?;
        if let Some(cf) = self.waw.cfile {
            self.current_file = Some(cf);
        }
        match (self.waw.cfile, self.waw.afile) {
            (Some(cf), _) if self.source_mode == SourceMode::C => {
                let file_name = self.debugger.lookup_file_by_id(cf).unwrap();
                println!(
                    "{}:{}\t\t{}",
                    file_name.short_name,
                    self.waw.cline,
                    self.waw.ctext.as_ref().unwrap()
                );
                if self.always_reg_dis {
                    self.print_reg_dis(inst_addr);
                };
            }
            (_, Some(af)) if self.source_mode > SourceMode::Raw => {
                let file_name = self.debugger.lookup_file_by_id(af).unwrap();
                println!(
                    "{}:{}\t\t{}",
                    file_name.short_name,
                    self.waw.aline,
                    self.waw.atext.as_ref().unwrap_or(&"".to_string())
                );
                if self.always_reg_dis {
                    self.print_reg_dis(inst_addr);
                };
            }
            _ => {
                println!("{}", self.waw.parent);
                self.print_reg_dis(inst_addr);
            }
        };

        Ok(())
    }
    fn print_reg_dis(&mut self, inst_addr: u16) {
        let mem = self.debugger.get_chunk(inst_addr, 3).unwrap();
        self.debugger.dis(&mem, inst_addr);

        // print pc, dissasembled instruction and registers
        let stat = Status::from_bits_truncate(self.debugger.read_sr());
        println!(
        "{:04x}:       {:<15} ac=${:02x} xr=${:02x} yr=${:02x} sp=${:02x} sp65=${:04x} sr=${:02x} {:?}",
        self.debugger.read_pc(),
        self.debugger.dis_line,
        self.debugger.read_ac(),
        self.debugger.read_xr(),
        self.debugger.read_yr(),
        self.debugger.read_sp(),
        self.debugger.read_sp65(),
        stat,
        stat
    );
    }
    fn print_code_line(&self, waw: &CodeLocation) -> Result<bool> {
        if let Some(cf) = waw.cfile {
            let file_name = self.debugger.lookup_file_by_id(cf).unwrap();
            println!(
                "{}:{}\t\t{}",
                file_name.short_name,
                waw.cline,
                waw.ctext.as_ref().unwrap()
            );
            return Ok(true);
        };
        Ok(false)
    }
}

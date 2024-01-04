#![allow(clippy::uninlined_format_args)]
use crate::cpu::Status;
use crate::debugger::{Debugger, FrameType::*, WatchType};
use crate::execute::{BugType, StopReason};
use crate::syntax;
use anyhow::{anyhow, bail, Result};
use clap::ArgMatches;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::collections::VecDeque;
use std::fs::File;
use std::io::{ErrorKind, Read};
use std::path::{Path, PathBuf};

pub struct Shell {
    debugger: Debugger,
    current_dis_addr: u16,
    _current_mem_addr: u16,
}

impl Shell {
    pub fn new() -> Self {
        Self {
            debugger: Debugger::new(),
            current_dis_addr: 0,
            _current_mem_addr: 0,
        }
    }
    pub fn shell(&mut self, file: Option<PathBuf>, _args: &[String]) -> Result<u8> {
        let mut rl = DefaultEditor::new()?;

        if let Err(e) = rl.load_history("history.txt") {
            if let ReadlineError::Io(ref re) = e {
                if re.kind() != ErrorKind::NotFound {
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
                        Err(e) => println!("{}", e), // display error
                        Ok(true) => break,           // quit was typed
                        Ok(false) => {}              // continue
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    println!("CTRL-C"); // pass on to running program?
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

        let _ = rl.save_history("history.txt");
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
                self.debugger.set_break(addr, false)?;
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
            Some(("list_bp", _)) => {
                let blist = self.debugger.get_breaks();
                for bp_addr in &blist {
                    let bp = self.debugger.get_bp(*bp_addr).unwrap();
                    println!("#{} 0x{:04X} ({})", bp.number, bp.addr, bp.symbol);
                }
            }
            Some(("list_wp", _)) => {
                let wlist = self.debugger.get_watches();
                for wp_addr in &wlist {
                    let wp = self.debugger.get_watch(*wp_addr).unwrap();
                    println!("#{} 0x{:04X} ({})", wp.number, wp.addr, wp.symbol);
                }
            }
            Some(("symbols", args)) => {
                let file = args.get_one::<String>("file").unwrap();
                self.debugger.load_ll(Path::new(file))?;
            }
            Some(("list_symbols", args)) => {
                let mtch = args.get_one::<String>("match");
                let symbols = self.debugger.get_symbols(mtch)?;
                for (sym, addr) in symbols {
                    println!("0x{:04x} {}", addr, sym);
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

            Some(("memory", args)) => {
                let addr_str = args.get_one::<String>("address").unwrap();
                let addr_str = &self.expand_expr(addr_str)?;
                let (addr, _) = self.debugger.convert_addr(addr_str)?;
                let chunk = self.debugger.get_chunk(addr, 48)?;
                self.mem_dump(addr, &chunk);
            }

            Some(("run", args)) => {
                let cmd_args = args
                    .get_many::<String>("args")
                    .map(Iterator::collect)
                    .unwrap_or_default();

                let reason = self.debugger.run(cmd_args)?;
                self.stop(reason);
            }

            Some(("go", _)) => {
                if !self.debugger.run_done {
                    bail!("program not running");
                };
                let reason = self.debugger.go()?;
                self.stop(reason);
            }

            Some(("next", _)) => {
                if !self.debugger.run_done {
                    bail!("program not running");
                };
                let reason = self.debugger.next()?;
                self.stop(reason);
            }

            Some(("step", _)) => {
                if !self.debugger.run_done {
                    bail!("program not running");
                };
                let reason = self.debugger.step()?;
                self.stop(reason);
            }

            Some(("delete_breakpoint", args)) => {
                let id = args.get_one::<String>("id");
                self.debugger.delete_breakpoint(id)?;
            }

            Some(("back_trace", _)) => {
                let stack = self.debugger.read_stack();
                for i in (0..stack.len()).rev() {
                    let frame = &stack[i];
                    match frame.frame_type {
                        Jsr((addr, ret, _sp, _)) => {
                            println!("jsr {:<10} x{:04x}", self.debugger.symbol_lookup(addr), ret);
                        }
                        Pha(ac) => println!("pha x{:02x}", ac),
                        Php(sr) => println!("php x{:02x}", sr),
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
                for _i in 0..10 {
                    let chunk = self.debugger.get_chunk(addr, 3)?;
                    if chunk.len() < 3 {
                        // we ran off the end of memory
                        break;
                    }
                    let delta = self.debugger.dis(&chunk, addr);
                    let addr_str = self.debugger.symbol_lookup(addr);
                    if addr_str.starts_with('.') {
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
            Some(("enable", args)) => {
                self.debugger
                    .enable_mem_check(*args.get_one::<bool>("memcheck").unwrap());
                self.debugger
                    .enable_stack_check(*args.get_one::<bool>("stackcheck").unwrap());
            }
            Some(("expr", args)) => {
                let expr = args.get_one::<String>("expression").unwrap();
                let ans = self.expand_expr(expr)?;
                println!("{:}", ans);
            }
            Some(("finish", _)) => {
                if !self.debugger.run_done {
                    bail!("program not running");
                };
                let reason = self.debugger.finish()?;
                self.stop(reason);
            }
            Some((name, _matches)) => unimplemented!("{name}"),
            None => unreachable!("subcommand required"),
        }

        Ok(false)
    }
    fn expand_expr(&mut self, exp: &str) -> Result<String> {
        if let Some(exp) = exp.strip_prefix('=') {
            let res = self.debugger.evaluate(exp)?;
            Ok(format!("${:04x}", res))
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
            println!("{:02x}{:02x} ", chunk[1], chunk[0]);
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
    fn stop(&mut self, reason: StopReason) {
        // common handler for when execution is interrupted
        match reason {
            StopReason::BreakPoint(bp_addr) => {
                let bp = self.debugger.get_bp(bp_addr).unwrap();
                println!("bp #{} {}", bp.number, bp.symbol);
            }
            StopReason::Exit(_) => {
                println!("exit");
                return;
            }
            StopReason::Count | StopReason::Next => {}
            StopReason::Bug(bug) => match bug {
                BugType::SpMismatch => {
                    println!("stack pointer mismatch");
                }
                BugType::Memcheck(addr) => {
                    println!("memory read uninit {:04x}", addr);
                }
            },
            StopReason::WatchPoint(addr) => {
                let wp = self.debugger.get_watch(addr).unwrap();
                println!("watch #{} 0x{:04x} ({}) ", wp.number, wp.addr, wp.symbol);
            }
            StopReason::Finish => {}
        }
        // disassemble the current instruction
        let inst_addr = self.debugger.read_pc();
        let mem = self.debugger.get_chunk(self.debugger.read_pc(), 3).unwrap();
        self.debugger.dis(&mem, inst_addr);

        // print pc, dissasembled instruction and registers
        let stat = Status::from_bits_truncate(self.debugger.read_sr());
        println!(
            "{:04x}:       {:<15} A={:02x} X={:02x} Y={:02x} SP={:02x} SR={:?}",
            self.debugger.read_pc(),
            self.debugger.dis_line,
            self.debugger.read_ac(),
            self.debugger.read_xr(),
            self.debugger.read_yr(),
            self.debugger.read_sp(),
            stat
        );
    }
}

use crate::cpu::Status;
use crate::debugger::{Debugger, FrameType::*};
use crate::execute::StopReason;
use anyhow::{anyhow, Result};
use clap::Arg;
use clap::Command;

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
    pub fn shell(&mut self, file: Option<PathBuf>, _args: Vec<String>) -> Result<u8> {
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

        // if let Some(args) = args {
        //self.debugger.cmd_args = args;
        //}

        //do we have a command file to run?
        if let Some(f) = file {
            let mut fd = File::open(f)?;
            let mut commstr = String::new();
            fd.read_to_string(&mut commstr)?;
            let mut commands: VecDeque<String> =
                commstr.split("\n").map(|s| s.to_string()).collect();
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

        let mut lastinput = String::new();
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
                        Err(e) => println!("{}", e),
                        Ok(true) => break,
                        Ok(false) => {}
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    println!("CTRL-C");
                    //  break;
                }
                Err(ReadlineError::Eof) => {
                    println!("CTRL-D");
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
        let args = shlex::split(line).ok_or(anyhow!("error: Invalid quoting"))?;
        let matches = self.syntax().try_get_matches_from(args)?;
        //.map_err(|e| e.to_string())?;
        match matches.subcommand() {
            Some(("break", args)) => {
                let addr = args.get_one::<String>("address").unwrap();
                self.debugger.set_break(&addr, false)?;
            }

            Some(("list_bp", args)) => {
                let blist = self.debugger.get_breaks();
                for i in 0..blist.len() {
                    let bp = self.debugger.get_bp(blist[i]).unwrap();
                    println!("#{} 0x{:04X} ({})", bp.number, bp.addr, bp.symbol);
                }
            }

            Some(("symbols", args)) => {
                let file = args.get_one::<String>("file").unwrap();
                self.debugger.load_ll(Path::new(file))?;
            }
            Some(("load_code", args)) => {
                let file = args.get_one::<String>("file").unwrap();
                self.debugger.load_code(Path::new(file))?;
            }
            Some(("quit", _matches)) => {
                println!("quit");
                return Ok(true);
            }
            Some(("memory", args)) => {
                let addr_str = args.get_one::<String>("address").unwrap();
                let addr = self.debugger.convert_addr(&addr_str)?;
                let chunk = self.debugger.get_chunk(addr, 48)?;
                self.mem_dump(addr, &chunk);
            }
            Some(("run", args)) => {
                let cmd_args = args
                    .get_many::<String>("args")
                    .map(|vals| vals.collect::<Vec<_>>())
                    .unwrap_or_default();

                let reason = self.debugger.run(cmd_args)?;
                self.stop(reason);
            }
            Some(("go", _args)) => {
                let reason = self.debugger.go()?;
                self.stop(reason);
            }
            Some(("next", _args)) => {
                let reason = self.debugger.next()?;
                self.stop(reason);
            }
            Some(("step", _args)) => {
                let reason = self.debugger.step()?;
                self.stop(reason);
            }
            Some(("delete_breakpoint", args)) => {
                let id = args.get_one::<String>("id");
                self.debugger.delete_breakpoint(id)?;
            }
            Some(("back_trace", _args)) => {
                let stack = self.debugger.read_stack();
                for i in (0..stack.len()).rev() {
                    let frame = &stack[i];
                    match frame.frame_type {
                        Jsr((addr, ret, _sp, _)) => {
                            println!("jsr {:<10} x{:04x}", self.debugger.symbol_lookup(addr), ret)
                        }
                        Pha(ac) => println!("pha x{:02x}", ac),
                        Php(sr) => println!("php x{:02x}", sr),
                    }
                }
            }
            Some(("dis", args)) => {
                let mut addr = if let Some(addr_str) = args.get_one::<String>("address") {
                    self.debugger.convert_addr(&addr_str)?
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
                    let delta = self.debugger.dis(&chunk, addr);
                    let addr_str = self.debugger.symbol_lookup(addr);
                    if addr_str.chars().next().unwrap() == '.' {
                        println!("{}:", addr_str);
                    }
                    println!("{:04x}:       {}", addr, self.debugger.dis_line);
                    addr += delta as u16;
                    self.current_dis_addr = addr;
                }
            }

            Some((name, _matches)) => unimplemented!("{name}"),
            None => unreachable!("subcommand required"),
        }

        Ok(false)
    }
    fn mem_dump(&mut self, mut addr: u16, chunk: &[u8]) {
        //let mut addr = 0;
        let mut line = String::new();
        for i in 0..chunk.len() {
            if i % 16 == 0 {
                if i > 0 {
                    println!("{}", line);
                    line.clear();
                }
                print!("{:04x}: ", addr);
            }
            print!("{:02x} ", chunk[i]);
            if chunk[i] >= 32 && chunk[i] < 127 {
                line.push(chunk[i] as char);
            } else {
                line.push('.');
            }
            addr += 1;
        }
        println!("{}", line);
    }
    fn stop(&mut self, reason: StopReason) {
        match reason {
            StopReason::BreakPoint(bp_addr) => {
                //println!("breakpoint");
                let bp = self.debugger.get_bp(bp_addr).unwrap();
                println!("bp #{} {}", bp.number, bp.symbol);
            }
            StopReason::Exit(_) => {
                println!("exit");
                return;
            }
            StopReason::Count | StopReason::Next => {
                // println!("count");
            }
            StopReason::Bug(_) => {
                println!("bug {:?}", reason);
            }
        }
        let inst_addr = self.debugger.read_pc();
        let mem = self.debugger.get_chunk(self.debugger.read_pc(), 3).unwrap();
        self.debugger.dis(&mem, inst_addr);
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
    fn syntax(&self) -> Command {
        // strip out usage
        const PARSER_TEMPLATE: &str = "\
        {all-args}
    ";
        // strip out name/version
        const APPLET_TEMPLATE: &str = "\
        {about-with-newline}\n\
        {usage-heading}\n    {usage}\n\
        \n\
        {all-args}{after-help}\
    ";

        Command::new("db65")
            .multicall(true)
            .arg_required_else_help(true)
            .subcommand_required(true)
            .subcommand_value_name("Command")
            .subcommand_help_heading("Commands")
            .help_template(PARSER_TEMPLATE)
            .subcommand(
                Command::new("break")
                    .about("set break points")
                    .alias("b")
                    .arg(Arg::new("address").required(true))
                    .help_template(APPLET_TEMPLATE),
            )
            .subcommand(
                Command::new("list_bp")
                    .about("set break points")
                    .alias("bl")
                    .help_template(APPLET_TEMPLATE),
            )
            .subcommand(
                Command::new("symbols")
                    .alias("ll")
                    .about("load symbol file")
                    .arg(Arg::new("file").required(true))
                    .arg_required_else_help(true)
                    .help_template(APPLET_TEMPLATE),
            )
            .subcommand(
                Command::new("load_code")
                    .alias("load")
                    .about("load binary file")
                    .arg(Arg::new("file").required(true))
                    .arg_required_else_help(true)
                    .help_template(APPLET_TEMPLATE),
            )
            .subcommand(
                Command::new("run")
                    .about("run code")
                    .arg(Arg::new("address"))
                    .arg(Arg::new("args").last(true).num_args(0..))
                    .help_template(APPLET_TEMPLATE),
            )
            .subcommand(
                Command::new("dis")
                    .about("disassemble")
                    .arg(Arg::new("address"))
                    .help_template(APPLET_TEMPLATE),
            )
            .subcommand(
                Command::new("quit")
                    .aliases(["exit", "q"])
                    .about("Quit db65")
                    .help_template(APPLET_TEMPLATE),
            )
            .subcommand(
                Command::new("next")
                    .alias("n")
                    .about("next instruction (step over)")
                    .help_template(APPLET_TEMPLATE),
            )
            .subcommand(
                Command::new("go")
                    .alias("g")
                    .about("resume execution")
                    .help_template(APPLET_TEMPLATE),
            )
            .subcommand(
                Command::new("step")
                    .alias("s")
                    .about("next instruction (step into)")
                    .help_template(APPLET_TEMPLATE),
            )
            .subcommand(
                Command::new("memory")
                    .aliases(["mem", "m"])
                    .about("display memory")
                    .arg(Arg::new("address").required(true))
                    .help_template(APPLET_TEMPLATE),
            )
            .subcommand(
                Command::new("back_trace")
                    .alias("bt")
                    .about("display call stack")
                    .help_template(APPLET_TEMPLATE),
            )
            .subcommand(
                Command::new("delete_breakpoint")
                    .alias("bd")
                    .arg(Arg::new("id").required(false))
                    .about("delete breakpoint")
                    .help_template(APPLET_TEMPLATE),
            )
    }
}

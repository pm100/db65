use crate::debugger::Debugger;
use anyhow::{anyhow, bail, Result};
use clap::Command;
use clap::{Arg, Parser};

use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::path::Path;

pub struct Shell {
    debugger: Debugger,
}

#[derive(Debug, Parser, Default)]
#[command(about, version, no_binary_name(true))]
struct BreakPoint {
    //#[arg()]
    //#[arg(long, short, default_value_t = String::from("Default endpoint"))]
    /// RPC endpoint of the node that this wallet will connect to
    addr: Option<String>,

    // #[arg(long, short)]
    refresh_rate: Option<u32>,
}
impl Shell {
    pub fn new() -> Self {
        Self {
            debugger: Debugger::new(),
        }
    }
    pub fn shell(&mut self) -> Result<()> {
        // `()` can be used when no completer is required
        let mut rl = DefaultEditor::new()?;
        //#[cfg(feature = "with-file-history")]
        if rl.load_history("history.txt").is_err() {
            println!("No previous history.");
        }

        loop {
            let readline = rl.readline(">> ");

            match readline {
                Ok(line) => {
                    rl.add_history_entry(line.as_str())?;

                    match self.dispatch(&line) {
                        Err(e) => println!("{}", e),
                        Ok(true) => break,
                        Ok(false) => {}
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    println!("CTRL-C");
                    break;
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
        //ß #[cfg(feature = "with-file-history")]
        let _ = rl.save_history("history.txt");
        Ok(())
    }

    fn load_code(&mut self, args: Vec<&str>) {
        //let input = vec!["--endpoint", "localhost:8000", "--refresh-rate", "15"];

        //let c = Cli::parse_from(args);
    }

    fn load_symbols(&mut self, args: Vec<&str>) {
        //let mut d = Debugger::new();
        let p = Path::new("ll");
        self.debugger.load_ll(&p).unwrap();
    }
    fn set_break(&mut self, args: Vec<&str>) -> Result<()> {
        let c = BreakPoint::parse_from(args);
        // c.addr;
        println!("{:?}", c);
        if let Some(ad) = c.addr {
            self.debugger.set_break(&ad)?;
        } else {
            let blist = self.debugger.get_breaks();

            for i in 0..blist.len() {
                println!("{:04x}", blist[i]);
            }
            //  blist.iter().map(|a| );
        }
        Ok(())
    }

    fn dispatch(&mut self, line: &str) -> Result<bool> {
        let args = shlex::split(line).ok_or(anyhow!("error: Invalid quoting"))?;
        let matches = self.syntax().try_get_matches_from(args)?;
        //.map_err(|e| e.to_string())?;
        match matches.subcommand() {
            Some(("break", args)) => {
                if let Some(addr) = args.get_one::<String>("address") {
                    self.debugger.set_break(&addr);
                } else {
                    let blist = self.debugger.get_breaks();
                    for i in 0..blist.len() {
                        println!("{:04x}", blist[i]);
                    }
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
                //println!("memory {} {}", addr, string);
            }
            Some(("run", args)) => {
                // let addr = args.get_one::<String>("address").unwrap();
                self.debugger.run()?;
                // println!("run {}", addr);
            }
            Some(("go", args)) => {
                // let addr = args.get_one::<String>("address").unwrap();
                self.debugger.go()?;
                // println!("run {}", addr);
            }
            Some(("next", args)) => {
                // let addr = args.get_one::<String>("address").unwrap();
                self.debugger.next()?;
                // println!("run {}", addr);
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
                    .about("set break points, no argument means list all")
                    .arg(Arg::new("address"))
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
    }
}

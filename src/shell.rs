use crate::debugger::Debugger;
use anyhow::Result;
use clap::Parser;
use regex::Regex;
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
        #[cfg(feature = "with-file-history")]
        if rl.load_history("history.txt").is_err() {
            println!("No previous history.");
        }
        let reg = Regex::new(r" ")?;
        loop {
            let readline = rl.readline(">> ");

            match readline {
                Ok(line) => {
                    rl.add_history_entry(line.as_str())?;
                    println!("Line: {}", line);
                    let mut spl = reg.split(&line);
                    let mut cmd = spl.next().unwrap();
                    //   println!("cmd={}", cmd);
                    if cmd.trim().is_empty() {
                        cmd = spl.next().unwrap();
                        //  println!("cmd={}", cmd);
                    }
                    println!("cmd={}", cmd);
                    let args = spl.collect();
                    match cmd {
                        "load" => self.load_code(args),
                        "quit" => break,
                        "ll" => self.load_symbols(args),
                        "break" => self.set_break(args),
                        _ => println!("what"),
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
        #[cfg(feature = "with-file-history")]
        rl.save_history("history.txt");
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
    fn set_break(&mut self, args: Vec<&str>) {
        let c = BreakPoint::parse_from(args);
        // c.addr;
        println!("{:?}", c);
        if let Some(ad) = c.addr {
            self.debugger.set_break(&ad);
        } else {
            let blist = self.debugger.get_breaks();

            for i in 0..blist.len() {
                println!("{:04x}", blist[i]);
            }
            //  blist.iter().map(|a| );
        }
    }
}
use clap::Command;
fn respond(line: &str) -> Result<bool, String> {
    let args = shlex::split(line).ok_or("error: Invalid quoting")?;
    let matches = cli()
        .try_get_matches_from(args)
        .map_err(|e| e.to_string())?;
    match matches.subcommand() {
        Some(("ping", _matches)) => {
            // write!(std::io::stdout(), "Pong").map_err(|e| e.to_string())?;
            // std::io::stdout().flush().map_err(|e| e.to_string())?;
            println!("ping");
        }
        Some(("quit", _matches)) => {
            //  write!(std::io::stdout(), "Exiting ...").map_err(|e| e.to_string())?;
            //  std::io::stdout().flush().map_err(|e| e.to_string())?;
            return Ok(true);
        }
        Some((name, _matches)) => unimplemented!("{name}"),
        None => unreachable!("subcommand required"),
    }

    Ok(false)
}

fn cli() -> Command {
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

    Command::new("repl")
        .multicall(true)
        .arg_required_else_help(true)
        .subcommand_required(true)
        .subcommand_value_name("APPLET")
        .subcommand_help_heading("APPLETS")
        .help_template(PARSER_TEMPLATE)
        .subcommand(
            Command::new("ping")
                .about("Get a response")
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("quit")
                .alias("exit")
                .about("Quit the REPL")
                .help_template(APPLET_TEMPLATE),
        )
}

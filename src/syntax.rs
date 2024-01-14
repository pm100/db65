use clap::{arg, Arg};
use clap::{ArgGroup, Command};

// Clap sub command syntax defintions
pub fn syntax() -> Command {
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
                .about("Set break points")
                .alias("b")
                .arg(Arg::new("address").required(true))
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("watch")
                .about("Set watch points")
                .alias("w")
                .arg(Arg::new("address").required(true))
                .arg(arg!(-r --read  "watch for read"))
                .arg(arg!(-w --write  "watch for write"))
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("list_bp")
                .about("List break points")
                .alias("bl")
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("list_wp")
                .about("List watch points")
                .alias("wl")
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("symbols")
                .alias("ll")
                .about("Load symbol file")
                .arg(Arg::new("file").required(true))
                .arg_required_else_help(true)
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("load_code")
                .alias("load")
                .about("Load binary file")
                .arg(Arg::new("file").required(true))
                .arg_required_else_help(true)
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("load_source")
                .alias("xx")
                .about("Load binary file")
                .arg(Arg::new("file").required(true))
                .arg_required_else_help(true)
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("run")
                .about("Run code")
                .arg(Arg::new("address"))
                .arg(Arg::new("args").last(true).num_args(0..))
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("dis")
                .about("Disassemble")
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
                .about("Next instruction (step over)")
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("go")
                .alias("g")
                .about("Resume execution")
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("step")
                .alias("s")
                .about("Next instruction (step into)")
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("memory")
                .aliases(["mem", "m"])
                .about("Display memory")
                .arg(Arg::new("address").required(true))
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("back_trace")
                .alias("bt")
                .about("Display call stack")
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("delete_breakpoint")
                .alias("bd")
                .arg(Arg::new("id").required(false))
                .about("Delete breakpoint")
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("print")
                .alias("p")
                .arg(arg!(<address>  "address of value to print"))
                .arg(arg!(asint:     -i   "integer"))
                .arg(arg!(aspointer: -p   "pointer"))
                .arg(arg!(asstring:  -s   "string"))
                .group(
                    ArgGroup::new("format").args(["asint", "aspointer", "asstring"]), //.required(true), // default to int
                )
                .about("Formatted display of memory")
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("list_symbols")
                .alias("ls")
                .arg(Arg::new("match").required(false))
                .about("List symbols")
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("enable")
                .alias("en")
                .arg(arg!( -m --memcheck  "enable memory check"))
                //.arg(arg!(  -t --memtrace  "enable memory trace"))
                .arg(arg!(  -s --stackcheck  "enable stack check"))
                .about("Enable features")
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("finish")
                .alias("fin")
                .about("Run until current function returns")
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("reg")
                .about("Set register value")
                .arg(arg!(<register> "register to set (ac,zr,yr,sp,pc,sr"))
                .arg(arg!(<value> "value, either integer or expression"))
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("write_memory")
                .alias("wm")
                .about("Write to memory")
                .arg(arg!(<address> "address to write to"))
                .arg(arg!(<value> "value, either integer or expression"))
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("dbginfo")
                .about("display various debug data")
                .arg(arg!(-s --segments  "display segments"))
                .arg(arg!([arg] "arg"))
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("next_statement")
                .alias("ns")
                .about("next statement")
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("list_source")
                .alias("lc")
                .about("list source")
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("expr")
                .arg(arg!(<expression>  "expression to evaluate"))
                .about("Evaluate address expression")
                .help_template(APPLET_TEMPLATE)
                .after_help(
                    r#"
Expressions can be used anywhere an address is required. Expr command
can be used to test an expression and also to inspect values.

Examples:
expr =0x20          evaluates to 0x20 (redundant)
expr =.xr           the xr register
expr =.xr+1         the xr register plus 1
dis =.pc-6          disassemble from pc-6
m =.xr+0x20         display memory at xr+0x20
m ptr               display memory at pointer (raw symbols just work anyway)
m =@(ptr)           dereference a pointer
m =@(ptr+0x20)      do math on a pointer
m =@(p1+(2*.yr))    more math
p -s =@(sreg)       print a string pointed to by sreg, sreg+1 

@ is the dereference operator for a word 
@b is the dereference operator for a byte 

Note if there are spaces in the expression, you must quote it:
mem '=@(ptr + 0x20)'
                    
                    "#,
                ),
        )
}

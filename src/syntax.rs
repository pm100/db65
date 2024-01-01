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
                .about("set break points")
                .alias("b")
                .arg(Arg::new("address").required(true))
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("watch")
                .about("set watch points")
                .alias("w")
                .arg(Arg::new("address").required(true))
                .arg(arg!(-r --read  "watch for read"))
                .arg(arg!(-w --write  "watch for write"))
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("list_bp")
                .about("list break points")
                .alias("bl")
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("list_wp")
                .about("list watch points")
                .alias("wl")
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
                .about("pretty print of memory")
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("list_symbols")
                .alias("ls")
                .arg(Arg::new("match").required(false))
                .about("list symbols")
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("enable")
                .alias("en")
                .arg(arg!( -m --memcheck  "enable memory check"))
                //.arg(arg!(  -t --memtrace  "enable memory trace"))
                .arg(arg!(  -s --stackcheck  "enable stack check"))
                .about("enable features")
                .help_template(APPLET_TEMPLATE),
        )
}

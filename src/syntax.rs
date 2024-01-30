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
            Command::new("load_code")
                .visible_alias("load")
                .about("Load binary file")
                .arg(Arg::new("file").required(true))
                .arg_required_else_help(true)
                .help_template(APPLET_TEMPLATE)
                .after_help(
                    "loads binary file into memory. 
 If there is a dbginfo file (.dbg), it will also load that,
 removing the need for load_dbginfo command",
                ),
        )
        .subcommand(
            Command::new("load_dbginfo")
                .visible_alias("dbg")
                .about("Load dbginfo file")
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
            Command::new("list_source")
                .visible_alias("lsc")
                .arg(arg!([address] "address to list from"))
                .about("list source")
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("quit")
                .visible_aliases(["exit", "q"])
                .about("Quit db65")
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("next_instruction")
                .visible_alias("ni")
                .about("Next instruction (step over)")
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("step_instruction")
                .visible_alias("si")
                .about("Next instruction (step into)")
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("next_statement")
                .visible_alias("ns")
                .about("next statement")
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("step_statement")
                .visible_alias("ss")
                .about("step statement")
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("go")
                .visible_alias("g")
                .about("Resume execution")
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("finish")
                .visible_alias("fin")
                .about("Run until current function returns")
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("break")
                .about("Set break point")
                .visible_alias("b")
                .arg(Arg::new("address").required(true))
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("watch")
                .about("Set watch points")
                .visible_alias("w")
                .arg(Arg::new("address").required(true))
                .arg(arg!(-r --read  "watch for read"))
                .arg(arg!(-w --write  "watch for write"))
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("list_breakpoints")
                .about("List break points")
                .alias("lbp")
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("list_watchpoints")
                .about("List watch points")
                .alias("lwp")
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("delete_breakpoint")
                .visible_alias("dbp")
                .arg(Arg::new("id").required(false))
                .about("Delete breakpoint")
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("delete_watchpoint")
                .visible_alias("dwp")
                .arg(Arg::new("id").required(false))
                .about("Delete watchpoint")
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("dis")
                .about("Disassemble")
                .arg(Arg::new("address"))
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("display_memory")
                .visible_aliases(["mem", "m"])
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
            Command::new("print")
                .visible_alias("p")
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
                .visible_alias("lsy")
                .arg(Arg::new("match").required(false))
                .about("List symbols")
                .help_template(APPLET_TEMPLATE)
                .long_about(
                    "List symbols matching match. If match is omitted, all symbols are listed.
Match is a substring, eg 'lsy main' will list all symbols containing 'main'",
                ),
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
                .visible_alias("wm")
                .about("Write to memory")
                .arg(arg!(<address> "address to write to"))
                .arg(arg!(<value> "value, either integer or expression"))
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("dbginfo")
                .about("display various debug data")
                .arg(arg!(-s --segments  "display segments"))
                .arg(arg!(-a --address_map  "display c source address map"))
                .arg(arg!([arg] "arg"))
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("expr")
                .arg(arg!(<expression>  "expression to evaluate"))
                .about("Evaluate address expression")
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("about")
                .about("explanation of commands")
                .arg(arg!([topic]))
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("settings")
                .about("change various settings")
                .alias("set")
                .arg(arg!(source_tree: -s --source_tree <path> "cc65 source tree")
                    .value_parser(clap::builder::PathBufValueParser::new()))
                .arg(
                    arg!(lines: -l --lines <number> "number of lines to list (dis, lsc)")
                        .value_parser(clap::value_parser!(u8).range(1..)),
                )
                .arg(
                    arg!(source_mode: -m --source_mode <switch> "Source mode ")
                    .value_parser(clap::builder::PossibleValuesParser::new(["c", "asm", "raw"])),
                )
                .arg(
                    arg!(regdis: -r --registers <switch> "display registers always")
                    .value_parser(clap::builder::BoolishValueParser::new()),
                )
                .arg(
                    arg!(dbgfile: -g --dbgfile_suffix <suffix> "File suffix for auto load of dbginfo files")
                    .value_parser(clap::builder::StringValueParser::new()),
                )
                .arg(
                    arg!(traps: -t --traps <switch> "Turn traps on or off ")
                        .value_parser(clap::builder::BoolishValueParser::new()),
                )
                .arg(
                    arg!(verbose: -v --verbose <switch> "Turn verbose messages on or off ")
                        .value_parser(clap::builder::BoolishValueParser::new()),
                )
                .after_help("'switch' means, 'on'/'true'/'yes' or 'off'/'false'/'no'")
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("status")
                .about("display db65 state")
                .help_template(APPLET_TEMPLATE),
        )
}

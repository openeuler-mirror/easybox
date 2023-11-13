//! This file is part of the easybox package.
//
// (c) Allen Xu <xubo3006@163.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use clap::{crate_version, Arg, Command};
use nix::unistd::isatty;
use std::cell::RefCell;
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::fs::MetadataExt;
use std::{env, fs, io, path::Path, process, rc::Rc};
use users::{get_user_by_name, get_user_by_uid};
use uucore::error::{USimpleError, UUsageError};
use uucore::libc::{c_ushort, sysconf, _SC_CLK_TCK};
use uucore::{error::UResult, format_usage, libc::ioctl};

const PROC_BASE: &str = "/proc";
const DEFAULT_ROOT_PID: i32 = 1;
const PFLAG_HILIGHT: u8 = 0x01;
const PFLAG_THREAD: u8 = 0x02;
const NUM_NS: usize = 8;
const BOLD: &str = "\x1b[1m";
const BOLD_END: &str = "\x1b[0m";
const NS_NAME: [&str; 8] = ["cgroup", "ipc", "mnt", "net", "pid", "user", "uts", "time"];
const AGE_TO_COLOR: [AgeToColor; 3] = [
    AgeToColor {
        age_seconds: 60,
        color: "\x1b[32m",
    },
    AgeToColor {
        age_seconds: 3600,
        color: "\x1b[33m",
    },
    AgeToColor {
        age_seconds: 0,
        color: "\x1b[31m",
    },
];
const SYM_ASCII: Symbols = Symbols {
    empty_2: "  ",
    branch_2: "|-",
    vert_2: "| ",
    last_2: "`-",
    single_3: "---",
    first_3: "-+-",
};
const SYM_UTF: Symbols = Symbols {
    empty_2: "  ",
    branch_2: "├─",
    vert_2: "│ ",
    last_2: "└─",
    single_3: "───",
    first_3: "─┬─",
};
const SYM_VT100: Symbols = Symbols {
    empty_2: "  ",
    branch_2: "\x1B(0\x0Ftq\x1B(B",
    vert_2: "\x1B(0\x0Fx\x1B(B ",
    last_2: "\x1B(0\x0Fmq\x1B(B",
    single_3: "\x1B(0\x0Fqqq\x1B(B",
    first_3: "\x1B(0\x0Fqwq\x1B(B",
};

///
#[derive(Clone)]
pub struct Config {
    ///
    pub arguments: bool,
    ///
    pub ascii: bool,
    ///
    pub compact_not: bool,
    ///
    pub color_age: Option<String>,
    ///
    pub show_pgids: bool,
    ///
    pub vt100: bool,
    ///
    pub highlight_all: bool,
    ///
    pub highlight_pid: Option<i32>,
    ///
    pub long: bool,
    ///
    pub numeric_sort: bool,
    ///
    pub ns_sort: Option<String>,
    ///
    pub show_pids: bool,
    ///
    pub show_parents: bool,
    ///
    pub ns_changes: bool,
    ///
    pub thread_names: bool,
    ///
    pub hide_threads: bool,
    ///
    pub uid_changes: bool,
    ///
    pub unicode: bool,
    ///
    pub security_context: bool,
    ///
    pub usage: Vec<u8>,
    ///
    pub pid_or_user: Option<String>,
    ///
    pub list_head: Rc<RefCell<HeadProc>>,
}

///
pub mod options {
    ///
    pub static ARGUMENTS: &str = "arguments";
    ///
    pub static ASCII: &str = "ascii";
    ///
    pub static COMPACTNOT: &str = "compact-not";
    ///
    pub static COLOR: &str = "color";
    ///
    pub static SHOWPGIDS: &str = "show-pgids";
    ///
    pub static VT100: &str = "vt100";
    ///
    pub static HIGHLIGHTALL: &str = "highlight-all";
    ///
    pub static HIGHLIGHTPID: &str = "highlight-pid";
    ///
    pub static LONG: &str = "long";
    ///
    pub static NUMERISORT: &str = "numeric-sort";
    ///
    pub static NSSORT: &str = "ns-sort";
    ///
    pub static SHOWPIDS: &str = "show-pids";
    ///
    pub static SHOWPARENTS: &str = "show-parents";
    ///
    pub static NSCHANGES: &str = "ns-changes";
    ///
    pub static THREADNAMES: &str = "thread-names";
    ///
    pub static HIDETHREADS: &str = "hide-threads";
    ///
    pub static UIDCHANGES: &str = "uid-changes";
    ///
    pub static UNICODE: &str = "unicode";
    ///
    pub static SECURITY: &str = "security-context";
    ///
    pub static PIDORUSER: &str = "pid-or-user";
}

impl Config {
    ///
    pub fn from(args_matches: &clap::ArgMatches, usage: Vec<u8>) -> UResult<Self> {
        let color_age: Option<String> = args_matches
            .get_one::<String>(options::COLOR)
            .map(|v| v.to_owned());

        let highlight_pid: Option<i32> = args_matches
            .get_one::<String>(options::HIGHLIGHTPID)
            .map(|sec: &String| {
                sec.parse::<i32>()
                    .map_err(|_| USimpleError::new(1, String::from_utf8_lossy(&usage)))
            })
            .transpose()?;

        let ns_sort: Option<String> = args_matches
            .get_one::<String>(options::NSSORT)
            .map(|v| v.to_owned());

        let pid_or_user: Option<String> = args_matches
            .get_one::<String>(options::PIDORUSER)
            .map(|v| v.to_owned());

        Ok(Self {
            arguments: args_matches.contains_id(options::ARGUMENTS),
            ascii: args_matches.contains_id(options::ASCII),
            compact_not: args_matches.contains_id(options::COMPACTNOT),
            color_age,
            show_pgids: args_matches.contains_id(options::SHOWPGIDS),
            vt100: args_matches.contains_id(options::VT100),
            highlight_all: args_matches.contains_id(options::HIGHLIGHTALL),
            highlight_pid,
            long: !args_matches.contains_id(options::LONG),
            numeric_sort: args_matches.contains_id(options::NUMERISORT),
            ns_sort,
            show_pids: args_matches.contains_id(options::SHOWPIDS),
            show_parents: args_matches.contains_id(options::SHOWPARENTS),
            ns_changes: args_matches.contains_id(options::NSCHANGES),
            thread_names: args_matches.contains_id(options::THREADNAMES),
            hide_threads: args_matches.contains_id(options::HIDETHREADS),
            uid_changes: args_matches.contains_id(options::UIDCHANGES),
            unicode: args_matches.contains_id(options::UNICODE),
            security_context: args_matches.contains_id(options::SECURITY),
            pid_or_user,
            usage,
            list_head: Rc::new(RefCell::new(HeadProc::new())),
        })
    }
}

///
pub fn parse_pstree_cmd_args(
    args: impl uucore::Args,
    about: &str,
    usage: &str,
    after_help: &str,
) -> UResult<Config> {
    let mut command = pstree_app(about, usage, after_help);
    let mut usage_doc = Vec::new();
    command.write_help(&mut usage_doc).unwrap();
    let arg_list = args.collect_lossy();
    Config::from(&command.try_get_matches_from(arg_list)?, usage_doc)
}

///
pub fn pstree_app<'a>(about: &'a str, usage: &'a str, after_help: &'a str) -> Command<'a> {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about(about)
        .after_help(after_help)
        .override_usage(format_usage(usage))
        .infer_long_args(true)
        .arg( Arg::new(options::ARGUMENTS)
                .short('a')
                .long(options::ARGUMENTS)
                .help("show command line arguments"),
        )
        .arg( Arg::new(options::ASCII)
                .short('A')
                .long(options::ASCII)
                .help("use ASCII line drawing characters"),
        )
        .arg( Arg::new(options::COMPACTNOT)
                .short('c')
                .long(options::COMPACTNOT)
                .help("don't compact identical subtrees"),
        )
        .arg( Arg::new(options::COLOR)
                .short('C')
                .long(options::COLOR)
                .value_name("TYPE")
                .takes_value(true)
                .help("color process by attribute\n (age)\n"),
        )
        .arg( Arg::new(options::SHOWPGIDS)
                .short('g')
                .long(options::SHOWPGIDS)
                .help("show process group ids; implies -c"),
        )
        .arg( Arg::new(options::VT100)
                .short('G')
                .long(options::VT100)
                .help("use VT100 line drawing characters"),
        )
        .arg( Arg::new(options::HIGHLIGHTALL)
                .short('h')
                .long(options::HIGHLIGHTALL)
                .help("highlight current process and its ancestors"),
        )
        .arg( Arg::new(options::HIGHLIGHTPID)
                .short('H')
                .long(options::HIGHLIGHTPID)
                .value_name("PID")
                .takes_value(true)
                .help("highlight this process and its ancestors"),
        )
        .arg( Arg::new(options::LONG)
                .short('l')
                .long(options::LONG)
                .help("don't truncate long lines"),
        )
        .arg( Arg::new(options::NUMERISORT)
                .short('n')
                .long(options::NUMERISORT)
                .help("sort output by PID"),
        )
        .arg( Arg::new(options::NSSORT)
                .short('N')
                .long(options::NSSORT)
                .value_name("TYPE")
                .takes_value(true)
                .help("sort output by this namespace type\n  (cgroup, ipc, mnt, net, pid, time, user, uts)\n"),
        )
        .arg( Arg::new(options::SHOWPIDS)
                .short('p')
                .long(options::SHOWPIDS)
                .help("show PIDs; implies -c"),
        )
        .arg( Arg::new(options::SHOWPARENTS)
                .short('s')
                .long(options::SHOWPARENTS)
                .help("show parents of the selected process"),
        )
        .arg( Arg::new(options::NSCHANGES)
                .short('S')
                .long(options::NSCHANGES)
                .help("show namespace transition"),
        )
        .arg(
            Arg::new(options::THREADNAMES)
                .short('t')
                .long(options::THREADNAMES)
                .help("show full thread names"),
        )
        .arg(
            Arg::new(options::HIDETHREADS)
                .short('T')
                .long(options::HIDETHREADS)
                .help("hide threads, show only process"),
        )
        .arg(
            Arg::new(options::UIDCHANGES)
                .short('u')
                .long(options::UIDCHANGES)
                .help("show uid transitions"),
        )
        .arg(
            Arg::new(options::UNICODE)
                .short('U')
                .long(options::UNICODE)
                .help("use UTF-8 (Unicode) line drawing characters"),
        )
        .arg(
            Arg::new(options::SECURITY)
                .short('Z')
                .long(options::SECURITY)
                .help("show security attributes"),
        )
        .arg(
            Arg::new(options::PIDORUSER)
                .action(clap::ArgAction::Append)
                .index(1)
                .multiple_values(true)
        )
        .trailing_var_arg(true)
}

///
#[derive(Clone)]
pub struct Symbols {
    empty_2: &'static str,
    branch_2: &'static str,
    vert_2: &'static str,
    last_2: &'static str,
    single_3: &'static str,
    first_3: &'static str,
}

///
#[derive(Clone)]
pub struct Proc {
    comm: String,
    argv: Vec<String>,
    argc: i32,
    pid: i32,
    pgid: i32,
    uid: i32,
    ns: Vec<u64>,
    flags: u8,
    age: f64,
    children: Option<Rc<RefCell<Child>>>,
    parent: Option<Rc<RefCell<Proc>>>,
    next: Option<Rc<RefCell<Proc>>>,
}

///
#[derive(Clone)]
pub struct HeadProc {
    head: Option<Rc<RefCell<Proc>>>,
}

impl HeadProc {
    ///
    pub fn new() -> Self {
        Self { head: None }
    }
}

impl Default for HeadProc {
    fn default() -> Self {
        Self::new()
    }
}

///
#[derive(Clone)]
pub struct Child {
    child: Rc<RefCell<Proc>>,
    next: Option<Rc<RefCell<Child>>>,
}

///
#[derive(Clone)]
pub struct NsEntry {
    number: u64,
    children: Option<Rc<RefCell<Child>>>,
    next: Option<Rc<RefCell<NsEntry>>>,
}

///
#[derive(Clone)]
pub struct HeadNsEntry {
    head: Option<Rc<RefCell<NsEntry>>>,
}

impl Proc {
    ///
    pub fn new(comm: String, pid: i32, uid: i32, list: Option<Rc<RefCell<Proc>>>) -> Self {
        Self {
            comm,
            argv: Vec::new(),
            argc: 0,
            pid,
            pgid: 0,
            uid,
            flags: 0,
            age: 0.0,
            children: None,
            next: list,
            parent: None,
            ns: vec![0; NUM_NS],
        }
    }
}

impl Child {
    ///
    pub fn new(child: Rc<RefCell<Proc>>) -> Self {
        Self { child, next: None }
    }
}

impl NsEntry {
    ///
    pub fn new(number: u64) -> Self {
        Self {
            number,
            children: None,
            next: None,
        }
    }
}

impl HeadNsEntry {
    ///
    pub fn new() -> Self {
        Self { head: None }
    }
}

impl Default for HeadNsEntry {
    fn default() -> Self {
        Self::new()
    }
}

///
#[derive(PartialEq, Eq, Clone)]
pub enum ColorType {
    ///
    ColorNone,
    ///
    ColorAge,
}

///
#[derive(Clone)]
struct AgeToColor {
    age_seconds: i32,
    color: &'static str,
}

///
#[derive(Clone)]
pub struct DumpTreeArgs {
    level: i32,
    rep: i32,
    leaf: bool,
    last: bool,
    prev_uid: i32,
    closing: i32,
}

impl DumpTreeArgs {
    ///
    pub fn new(level: i32, rep: i32, leaf: bool, last: bool, prev_uid: i32, closing: i32) -> Self {
        Self {
            level,
            rep,
            leaf,
            last,
            prev_uid,
            closing,
        }
    }
}

///
pub struct OutputArgs {
    width: Vec<i32>,
    more: Vec<i32>,
    cur_x: usize,
}

impl OutputArgs {
    ///
    pub fn new() -> Self {
        Self {
            width: Vec::new(),
            more: Vec::new(),
            cur_x: 1,
        }
    }
}

impl Default for OutputArgs {
    fn default() -> Self {
        Self::new()
    }
}

///
pub fn handle_input(config: Config) -> UResult<i32> {
    let mut pid: i32;
    let mut wait_end: bool = false;
    let _print_args: i32 = 0;
    let _compact: i32 = 1;
    let mut color_highlight = ColorType::ColorNone;
    let mut highlight: i32 = 0;
    let mut pid_set: i32 = 0;
    let mut pw_uid: i32 = -1;
    let mut nsid: usize = NUM_NS;
    let mut nsroot: HeadNsEntry = HeadNsEntry::new();
    let mut outputargs = OutputArgs::new();
    let mut dumped: bool = false;
    let mut sym: Symbols = SYM_ASCII;

    let output_width = get_output_width();
    let root_pid = find_root_pid();
    pid = root_pid;

    let args: Vec<String> = env::args().collect();
    let prog_name = &args[0];
    if prog_name == "pstree.x11" {
        wait_end = true;
    }

    // Attempt to figure out a good default symbol set.  Will be overridden by
    // command-line options, if given.
    if isatty(1).unwrap() {
        if let Ok(ctype) = env::var("LC_CTYPE") {
            if ctype.contains("UTF-8") {
                sym = SYM_UTF
            }
        } else if let Ok(lang) = env::var("LANG") {
            if lang.contains("UTF-8") {
                sym = SYM_UTF
            }
        }
    };

    if config.ascii {
        sym = SYM_ASCII;
    }
    if config.vt100 {
        sym = SYM_VT100;
    }

    if config.color_age.is_some() {
        let color_type = config.color_age.clone().unwrap();
        if color_type.eq_ignore_ascii_case("age") {
            color_highlight = ColorType::ColorAge;
        } else {
            return Err(UUsageError::new(1, String::from_utf8_lossy(&config.usage)));
        }
    }

    if config.highlight_all && config.highlight_pid.is_some() {
        return Err(UUsageError::new(1, String::from_utf8_lossy(&config.usage)));
    }

    if config.highlight_all {
        highlight = process::id() as i32;
    }

    if config.highlight_pid.is_some() {
        highlight = config.highlight_pid.unwrap();
    }

    if config.ns_sort.clone().is_some() {
        nsid = get_ns_id(config.ns_sort.clone().unwrap());
        if nsid == 8 {
            return Err(UUsageError::new(1, String::from_utf8_lossy(&config.usage)));
        }
        if !verify_ns(nsid) {
            return Err(UUsageError::new(
                1,
                format!(
                    "procfs file for {} namespace not available",
                    config.ns_sort.unwrap()
                ),
            ));
        }
    }

    if config.pid_or_user.is_some() {
        match config.pid_or_user.clone().unwrap().parse::<i32>() {
            Ok(pid_tmp) => {
                pid = pid_tmp;
                pid_set = 1;
            }
            Err(_) => {
                pw_uid = match get_user_by_name(&config.pid_or_user.clone().unwrap()) {
                    Some(user) => user,
                    None => {
                        return Err(USimpleError::new(
                            1,
                            format!("No such user name: {}", &config.pid_or_user.unwrap()),
                        ));
                    }
                }
                .uid() as i32;
            }
        }
    }

    read_proc(root_pid, &config)?;
    let mut current = find_proc(highlight, config.list_head.borrow().head.clone());
    while current.is_some() {
        let now_node = current.unwrap();
        now_node.borrow_mut().flags |= PFLAG_HILIGHT;
        current = now_node.borrow().parent.clone();
    }

    if config.show_parents && pid_set == 1 {
        let child_proc = find_proc(pid, config.list_head.borrow().head.clone());
        if child_proc.is_some() {
            trim_tree_by_parent(child_proc.unwrap(), config.numeric_sort);
            pid = root_pid;
        } else {
            return Err(USimpleError::new(1, format!("Process {} not found.", pid)));
        }
    }

    if nsid != 8 {
        sort_by_namespace(None, nsid, &mut nsroot, config.list_head.clone());
        dump_by_namespace(
            &nsroot,
            &mut outputargs,
            output_width,
            color_highlight,
            &config,
            sym,
        );
    } else if pw_uid == -1 {
        dump_tree(
            find_proc(pid, config.list_head.borrow().head.clone()),
            DumpTreeArgs::new(0, 1, true, true, 0, 0),
            &mut outputargs,
            output_width,
            color_highlight,
            &config,
            sym,
        );
    } else {
        dump_by_user(
            find_proc(root_pid, config.list_head.borrow().head.clone()),
            (pw_uid, output_width),
            &mut dumped,
            &mut outputargs,
            color_highlight,
            &config,
            sym,
        );
        if !dumped {
            return Err(USimpleError::new(1, "No processes found."));
        }
    }

    if wait_end {
        eprintln!("Press return to close");
        let _ = io::stdout().flush();
        let _ = io::stdin().read_line(&mut String::new());
    }

    Ok(0)
}

struct Winsz {
    #[allow(dead_code)]
    ws_row: c_ushort,
    ws_col: c_ushort,
    #[allow(dead_code)]
    ws_xpixel: c_ushort,
    #[allow(dead_code)]
    ws_ypixel: c_ushort,
}

/// Determine the correct output width
pub fn get_output_width() -> i32 {
    if let Ok(env_columns) = env::var("COLUMNS") {
        if let Ok(t) = env_columns.parse::<i32>() {
            if t > 0 && t < 0x7fffffff {
                return t;
            }
        }
    }

    let mut winsz: Winsz = Winsz {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };

    // TIOCGWINSZ 0x5413 Terminal ioctl get weindow size
    if unsafe { ioctl(1, 0x5413, &mut winsz) == 0 } && winsz.ws_col != 0 {
        return winsz.ws_col as i32;
    }
    132
}

/// Find the root PID.
/// Check to see if PID 0 exists, such as in LXC
/// Otherwise return 0 for BSD, 1 for others
pub fn find_root_pid() -> i32 {
    let proc_path = Path::new(PROC_BASE).join("0");
    if fs::metadata(proc_path).is_ok() {
        0
    } else {
        DEFAULT_ROOT_PID
    }
}

///
pub fn get_ns_id(name: String) -> usize {
    for (index, ns) in NS_NAME.iter().enumerate() {
        if name.eq(ns) {
            return index;
        }
    }

    NUM_NS
}

///
pub fn get_ns_name(id: usize) -> String {
    if id >= 8 {
        return "".to_string();
    }
    NS_NAME[id].to_string()
}

/// Verify namespace
pub fn verify_ns(id: usize) -> bool {
    let ns_name: String = get_ns_name(id);
    let filename = format!("/proc/{}/ns/{}", process::id(), ns_name);

    if fs::metadata(filename).is_err() {
        return false;
    }

    true
}

/// Find namespace and add node
pub fn find_ns_and_add(root: &mut HeadNsEntry, r: Rc<RefCell<Proc>>, id: usize) {
    let mut last: Option<Rc<RefCell<NsEntry>>> = None;
    let mut ptr = root.head.clone();
    while ptr.is_some() {
        if ptr.clone().unwrap().borrow().number == r.borrow().ns[id] {
            break;
        }
        last = ptr.clone();
        ptr = ptr.unwrap().borrow().next.clone();
    }

    if ptr.is_none() {
        ptr = Some(Rc::new(RefCell::new(NsEntry::new(r.borrow().ns[id]))));
        if root.head.is_none() {
            root.head = ptr.clone();
        } else {
            last.unwrap().borrow_mut().next = ptr.clone();
        }
    }

    // move the child to under the namespace's umbrella
    let new = Rc::new(RefCell::new(Child::new(r.clone())));
    if ptr.clone().unwrap().borrow().children.is_some() {
        let mut c_before = ptr.unwrap().borrow().children.clone();
        while c_before.clone().unwrap().borrow().next.is_some() {
            c_before = c_before.unwrap().borrow().next.clone();
        }
        c_before.unwrap().borrow_mut().next = Some(new);
    } else {
        ptr.unwrap().borrow_mut().children = Some(new);
    }

    // detaching from parent
    if r.borrow().parent.is_some() {
        let mut c = r.borrow().parent.clone().unwrap().borrow().children.clone();
        let mut last: Option<Rc<RefCell<Child>>> = None;
        while c.is_some() {
            let c_now = c.clone().unwrap();
            if Rc::ptr_eq(&c_now.borrow().child, &r) {
                let tmp_child = c_now.borrow().next.clone();
                if let Some(last_tmp) = last {
                    last_tmp.borrow_mut().next = tmp_child;
                }
                break;
            }
            last = c.clone();
            c = c_now.borrow().next.clone();
        }
        r.borrow_mut().parent = None;
    }
}

/// Sort by namespace
pub fn sort_by_namespace(
    mut r: Option<Rc<RefCell<Proc>>>,
    id: usize,
    root: &mut HeadNsEntry,
    list_head: Rc<RefCell<HeadProc>>,
) {
    // first run, find the first process
    if r.is_none() {
        r = find_proc(1, list_head.borrow().head.clone());
        if r.is_none() {
            return;
        }
    }

    let now_r = r.clone().unwrap();

    if now_r.borrow().parent.is_none()
        || now_r.borrow().parent.clone().unwrap().borrow().ns[id] != now_r.borrow().ns[id]
    {
        find_ns_and_add(root, r.unwrap(), id);
    }

    let mut walk = now_r.borrow().children.clone();
    while walk.is_some() {
        let next = walk.clone().unwrap().borrow().next.clone();
        sort_by_namespace(
            Some(walk.unwrap().borrow().child.clone()),
            id,
            root,
            list_head.clone(),
        );
        walk = next;
    }
}

/// Output information
pub fn dump_tree(
    current: Option<Rc<RefCell<Proc>>>,
    mut dumpargs: DumpTreeArgs,
    outputargs: &mut OutputArgs,
    output_width: i32,
    color_highlight: ColorType,
    config: &Config,
    sym: Symbols,
) {
    let add: i32;
    let mut info: bool;
    let trunc = config.long;

    assert!(dumpargs.closing >= 0);
    if current.is_none() {
        return;
    }
    let current = current.unwrap();

    if !dumpargs.leaf {
        for lvl in 0..dumpargs.level as usize {
            for _ in 0..outputargs.width[lvl] + 1 {
                out_char(' ', trunc, outputargs, output_width);
            }
            let output = if lvl == (dumpargs.level - 1) as usize {
                if dumpargs.last {
                    sym.last_2
                } else {
                    sym.branch_2
                }
            } else if outputargs.more[lvl + 1] != 0 {
                sym.vert_2
            } else {
                sym.empty_2
            };
            out_string(output.to_string(), trunc, outputargs, output_width);
        }
    }

    if dumpargs.rep < 2 {
        add = 0;
    } else {
        add = out_int(dumpargs.rep, trunc, outputargs, output_width) + 2;
        out_string("*[".to_string(), trunc, outputargs, output_width);
    }
    print_proc_color(current.borrow().age as i32, color_highlight.clone());
    if current.borrow().flags & PFLAG_HILIGHT != 0 {
        print!("{}", BOLD);
    }

    let swapped = i32::from(config.arguments);
    info = config.arguments;
    if swapped == 1 && current.borrow().argc < 0 {
        out_char('(', trunc, outputargs, output_width);
    }
    let comm_len = out_args(
        current.borrow().comm.clone(),
        trunc,
        outputargs,
        output_width,
    );
    let offset = outputargs.cur_x;
    if config.show_pids {
        out_char(
            if info {
                ','
            } else {
                info = true;
                '('
            },
            trunc,
            outputargs,
            output_width,
        );
        out_int(current.borrow().pid, trunc, outputargs, output_width);
    }
    if config.show_pgids {
        out_char(
            if info {
                ','
            } else {
                info = true;
                '('
            },
            trunc,
            outputargs,
            output_width,
        );
        out_int(current.borrow().pgid, trunc, outputargs, output_width);
    }
    if config.uid_changes && dumpargs.prev_uid != current.borrow().uid {
        out_char(
            if info {
                ','
            } else {
                info = true;
                '('
            },
            trunc,
            outputargs,
            output_width,
        );
        if let Some(pw) = get_user_by_uid(current.borrow().uid as u32) {
            let name = pw.name().to_string_lossy().into_owned();
            out_string(name, trunc, outputargs, output_width)
        } else {
            out_int(current.borrow().uid, trunc, outputargs, output_width);
        }
    }
    if config.ns_changes && current.borrow().parent.is_some() {
        for i in 0..NUM_NS {
            if current.borrow().ns[i] == 0
                || current.borrow().parent.clone().unwrap().borrow().ns[i] == 0
            {
                continue;
            }
            if current.borrow().ns[i] != current.borrow().parent.clone().unwrap().borrow().ns[i] {
                out_char(
                    if info {
                        ','
                    } else {
                        info = true;
                        '('
                    },
                    trunc,
                    outputargs,
                    output_width,
                );
                out_string(get_ns_name(i), trunc, outputargs, output_width)
            }
        }
    }
    if config.security_context {
        out_char(
            if info {
                ','
            } else {
                info = true;
                '('
            },
            trunc,
            outputargs,
            output_width,
        );
        out_scontext(current.clone(), trunc, outputargs, output_width)
    }
    if (swapped != 0 && config.arguments && current.borrow().argc < 0) || (swapped == 0 && info) {
        out_char(')', trunc, outputargs, output_width)
    }
    if current.borrow().flags & PFLAG_HILIGHT != 0 {
        print!("{}", BOLD_END);
    }
    if config.arguments {
        for i in 0..current.borrow().argc {
            out_char(' ', trunc, outputargs, output_width);
            let mut len = 0;
            let here = current.borrow().argv[i as usize].clone();
            for c in here.chars() {
                len += if (' '..='~').contains(&c) { 1 } else { 4 };
            }
            if ((outputargs.cur_x as i32 + len)
                <= (output_width - (if i == current.borrow().argc - 1 { 0 } else { 4 })))
                || !trunc
            {
                out_args(here, trunc, outputargs, output_width);
            } else {
                out_string("...".to_string(), trunc, outputargs, output_width);
                break;
            }
        }
    }
    reset_color(color_highlight.clone());
    if config.security_context || config.arguments || current.borrow().children.is_none() {
        while dumpargs.closing > 0 {
            out_char(']', trunc, outputargs, output_width);
            dumpargs.closing -= 1;
        }
        out_newline(outputargs);
    }
    if dumpargs.level > outputargs.more.len() as i32 - 1 {
        outputargs.more.push(if dumpargs.last { 0 } else { 1 });
    } else {
        outputargs.more[dumpargs.level as usize] = if dumpargs.last { 0 } else { 1 };
    }

    if config.security_context || config.arguments {
        if dumpargs.level > outputargs.width.len() as i32 - 1 {
            outputargs
                .width
                .push(swapped + if comm_len > 1 { 0 } else { -1 });
        } else {
            outputargs.width[dumpargs.level as usize] = swapped + if comm_len > 1 { 0 } else { -1 }
        }
        let mut count;
        let mut walk = current.borrow().children.clone();
        while walk.is_some() {
            let walk_now = walk.clone().unwrap();
            let mut next = walk_now.borrow().next.clone();
            count = 0;

            if !config.show_pids
                && !config.compact_not
                && (walk_now.borrow().child.borrow().flags & PFLAG_THREAD) > 0
            {
                let mut scan = walk_now.borrow().next.clone();
                let mut last: Option<Rc<RefCell<Child>>> = None;
                while scan.is_some() {
                    let scan_now = scan.clone().unwrap();
                    if !tree_equal(
                        walk_now.borrow().child.clone(),
                        scan_now.borrow().child.clone(),
                        config.uid_changes,
                        config.ns_changes,
                    ) {
                        last = scan.clone();
                        scan = scan_now.borrow().next.clone();
                    } else {
                        if match (next.clone(), scan.clone()) {
                            (Some(rc_a), Some(rc_b)) => Rc::ptr_eq(&rc_a, &rc_b),
                            (None, None) => true,
                            _ => false,
                        } {
                            next = scan_now.borrow().next.clone();
                        }
                        count += 1;
                        if let Some(l) = last.clone() {
                            let tmp = scan_now.borrow().next.clone();
                            l.borrow_mut().next = tmp.clone();
                            scan = tmp.clone();
                        } else {
                            let tmp = scan_now.borrow().next.clone();
                            scan = tmp.clone();
                        }
                        last = scan.clone();
                    }
                }
                dump_tree(
                    Some(walk_now.borrow().child.clone()),
                    DumpTreeArgs::new(
                        dumpargs.level + 1,
                        count + 1,
                        false,
                        next.is_none(),
                        current.borrow().uid,
                        dumpargs.closing + i32::from(count > 0),
                    ),
                    outputargs,
                    output_width,
                    color_highlight.clone(),
                    config,
                    sym.clone(),
                );
            } else {
                dump_tree(
                    Some(walk_now.borrow().child.clone()),
                    DumpTreeArgs::new(
                        dumpargs.level + 1,
                        1,
                        false,
                        walk.unwrap().borrow().next.is_none(),
                        current.borrow().uid,
                        0,
                    ),
                    outputargs,
                    output_width,
                    color_highlight.clone(),
                    config,
                    sym.clone(),
                )
            }
            walk = next;
        }
        return;
    }
    if dumpargs.level > outputargs.width.len() as i32 - 1 {
        outputargs
            .width
            .push(comm_len + outputargs.cur_x as i32 - offset as i32 + add);
    } else {
        outputargs.width[dumpargs.level as usize] =
            comm_len + outputargs.cur_x as i32 - offset as i32 + add;
    }

    if outputargs.cur_x as i32 >= output_width && trunc {
        out_string(sym.first_3.to_string(), trunc, outputargs, output_width);
        out_string("+".to_string(), trunc, outputargs, output_width);
        out_newline(outputargs);
        return;
    }
    let mut first = true;
    let mut walk = current.borrow().children.clone();
    while walk.is_some() {
        let walk_now = walk.clone().unwrap();
        let mut next = walk_now.borrow().next.clone();
        let mut count = 0;

        if !config.show_pids && !config.compact_not {
            let mut scan = walk_now.borrow().next.clone();
            let mut last: Option<Rc<RefCell<Child>>> = None;
            while scan.is_some() {
                let scan_now = scan.clone().unwrap();
                if !tree_equal(
                    walk_now.borrow().child.clone(),
                    scan_now.borrow().child.clone(),
                    config.uid_changes,
                    config.ns_changes,
                ) {
                    last = scan.clone();
                    scan = scan_now.borrow().next.clone();
                } else {
                    if match (next.clone(), scan.clone()) {
                        (Some(rc_a), Some(rc_b)) => Rc::ptr_eq(&rc_a, &rc_b),
                        (None, None) => true,
                        _ => false,
                    } {
                        next = scan_now.borrow().next.clone();
                    }
                    count += 1;
                    if let Some(l) = last.clone() {
                        let tmp = scan_now.borrow().next.clone();
                        l.borrow_mut().next = tmp.clone();
                        scan = tmp.clone();
                    } else {
                        let tmp = scan_now.borrow().next.clone();
                        scan = tmp.clone();
                    }
                    last = scan.clone();
                }
            }
        }
        if first {
            let out_str = if next.is_some() {
                sym.first_3.to_string()
            } else {
                sym.single_3.to_string()
            };
            out_string(out_str, trunc, outputargs, output_width);
            first = false;
        }
        dump_tree(
            Some(walk_now.borrow().child.clone()),
            DumpTreeArgs::new(
                dumpargs.level + 1,
                count + 1,
                Rc::ptr_eq(&walk_now, &current.borrow().children.clone().unwrap()),
                next.is_none(),
                current.borrow().uid,
                dumpargs.closing + i32::from(count > 0),
            ),
            outputargs,
            output_width,
            color_highlight.clone(),
            config,
            sym.clone(),
        );

        walk = next;
    }
}

/// Output according to user
pub fn dump_by_user(
    current: Option<Rc<RefCell<Proc>>>,
    uid_outwidth: (i32, i32),
    dumped: &mut bool,
    outputargs: &mut OutputArgs,
    color_highlight: ColorType,
    config: &Config,
    sym: Symbols,
) {
    if current.is_none() {
        return;
    }
    let (uid, output_width) = uid_outwidth;
    let current = current.unwrap();
    if current.borrow().uid == uid {
        if *dumped {
            println!();
        }
        dump_tree(
            Some(current),
            DumpTreeArgs::new(0, 1, true, true, uid, 0),
            outputargs,
            output_width,
            color_highlight,
            config,
            sym,
        );
        *dumped = true;
        return;
    }
    let mut walk = current.borrow().children.clone();
    while walk.is_some() {
        let walk_now = walk.clone().unwrap();
        dump_by_user(
            Some(walk_now.borrow().child.clone()),
            uid_outwidth,
            dumped,
            outputargs,
            color_highlight.clone(),
            config,
            sym.clone(),
        );
        walk = walk_now.borrow().next.clone();
    }
}

/// Output according to namespace
pub fn dump_by_namespace(
    root: &HeadNsEntry,
    outputargs: &mut OutputArgs,
    output_width: i32,
    color_highlight: ColorType,
    config: &Config,
    sym: Symbols,
) {
    let mut ns_head = root.head.clone();
    while ns_head.is_some() {
        let ns_now = ns_head.unwrap();
        let buff = format!("[{}]\n", ns_now.borrow().number);
        out_string(buff, config.long, outputargs, output_width);
        let mut c = ns_now.borrow().children.clone();
        while c.is_some() {
            let c_now = c.unwrap();
            dump_tree(
                Some(c_now.borrow().child.clone()),
                DumpTreeArgs::new(0, 1, true, true, 0, 0),
                outputargs,
                output_width,
                color_highlight.clone(),
                config,
                sym.clone(),
            );
            c = c_now.borrow().next.clone();
        }
        ns_head = ns_now.borrow().next.clone();
    }
}

/// Trim process tree by parent
pub fn trim_tree_by_parent(current: Rc<RefCell<Proc>>, by_pid: bool) {
    let parent = current.borrow().parent.clone();
    if parent.is_none() {
        return;
    }
    let parent = parent.unwrap();
    parent.borrow_mut().children = None;
    add_child(parent.clone(), current, by_pid);
    trim_tree_by_parent(parent, by_pid);
}

/// Obtain system runtime
pub fn uptime() -> f64 {
    let path = Path::new("/proc/uptime");

    let mut file = match fs::File::open(path) {
        Ok(file) => file,
        Err(_) => {
            eprintln!("pstree: error opening uptime file");
            process::exit(1);
        }
    };

    let mut contents = String::new();
    if file.read_to_string(&mut contents).is_err() {
        eprintln!("uptime");
        return 0.0;
    }

    let uptime: f64 = match contents.split_whitespace().next() {
        Some(num_str) => num_str.parse().unwrap_or(0.0),
        None => 0.0,
    };

    uptime
}

/// process age from jiffies to seconds via uptime
pub fn process_age(jf: u64) -> f64 {
    let sc_clk_tck = unsafe { sysconf(_SC_CLK_TCK) };
    assert!(sc_clk_tck > 0, "Failed to get _SC_CLK_TCK");
    let age: f64 = uptime() - jf as f64 / sc_clk_tck as f64;
    if age < 0.0 {
        return 0.0;
    }
    age
}

/// Get threadname
pub fn get_threadname(pid: i32, tid: i32, comm: String, thread_names: bool) -> String {
    let threadname: String;
    let path = format!("{}/{}/task/{}/stat", PROC_BASE, pid, tid);

    if !thread_names {
        threadname = format!("{{{}}}", comm);
        return threadname;
    }

    if let Ok(mut file) = fs::File::open(path) {
        let mut readbuf: String = String::new();
        if file.read_to_string(&mut readbuf).is_ok() {
            if let (Some(start), Some(end)) = (readbuf.find('('), readbuf.find(')')) {
                let thread_common = &readbuf[start + 1..end];
                threadname = format!("{{{}}}", thread_common);
                return threadname;
            }
        }
    }

    // Fall back to old method
    threadname = format!("{{{}}}", comm);
    threadname
}

/// Add new process node
pub fn new_proc(
    comm: String,
    pid: i32,
    uid: i32,
    list_head: Rc<RefCell<HeadProc>>,
) -> Option<Rc<RefCell<Proc>>> {
    let new = Rc::new(RefCell::new(Proc::new(
        comm,
        pid,
        uid,
        list_head.borrow().head.clone(),
    )));
    new_proc_ns(new.clone());
    list_head.borrow_mut().head = Some(new.clone());
    Some(new)
}

/// Add child process nodesadd
pub fn add_child(parent: Rc<RefCell<Proc>>, child: Rc<RefCell<Proc>>, by_pid: bool) {
    let new = Rc::new(RefCell::new(Child::new(child.clone())));

    if parent.borrow_mut().children.is_none() {
        parent.borrow_mut().children = Some(new);
        return;
    }

    let mut walk = parent.borrow().children.clone();
    let mut last: Option<Rc<RefCell<Child>>> = None;
    while let Some(now_node) = walk.clone() {
        if by_pid {
            if now_node.borrow().child.borrow().pid > child.borrow().pid
                || (now_node.borrow().child.borrow().pid == child.borrow().pid
                    && now_node.borrow().child.borrow().comm > child.borrow().comm)
                || (now_node.borrow().child.borrow().pid == child.borrow().pid
                    && now_node.borrow().child.borrow().comm == child.borrow().comm
                    && now_node.borrow().child.borrow().uid > child.borrow().uid)
            {
                break;
            }
        } else if now_node.borrow().child.borrow().comm > child.borrow().comm
            || (now_node.borrow().child.borrow().comm == child.borrow().comm
                && now_node.borrow().child.borrow().pid > child.borrow().pid)
            || (now_node.borrow().child.borrow().comm == child.borrow().comm
                && now_node.borrow().child.borrow().pid == child.borrow().pid
                && now_node.borrow().child.borrow().uid > child.borrow().uid)
        {
            break;
        }

        last = walk.clone();
        walk = now_node.borrow().next.clone();
    }

    new.borrow_mut().next = walk;
    if let Some(tmp_last) = last {
        tmp_last.borrow_mut().next = Some(new);
    } else {
        parent.borrow_mut().children = Some(new);
    }
}

/// Set args of process
pub fn set_args(this: Rc<RefCell<Proc>>, args: String) {
    if args.is_empty() {
        this.borrow_mut().argc = -1;
        return;
    }
    this.borrow_mut().argc = 0;

    let parts: Vec<String> = args
        .split_once('\0')
        .map(|x| x.1)
        .unwrap_or("")
        .split('\0')
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect();

    this.borrow_mut().argc = parts.len() as i32;
    this.borrow_mut().argv = parts;
}

/// Reset color output
pub fn reset_color(color_highlight: ColorType) {
    if color_highlight != ColorType::ColorNone {
        print!("\x1b[0m");
    }
}

/// Add color to node output
pub fn print_proc_color(process_age: i32, color_highlight: ColorType) {
    if color_highlight == ColorType::ColorAge {
        let mut p: AgeToColor = AGE_TO_COLOR[2].clone();
        for p_tmp in AGE_TO_COLOR {
            if process_age < p_tmp.age_seconds {
                p = p_tmp;
                break;
            }
        }
        print!("{}", p.color);
    }
}

/// Find nodes by pid
pub fn find_proc(pid: i32, list: Option<Rc<RefCell<Proc>>>) -> Option<Rc<RefCell<Proc>>> {
    let mut walk: Option<Rc<RefCell<Proc>>> = list;

    while walk.is_some() {
        if walk.clone().unwrap().borrow().pid == pid {
            return walk;
        }
        walk = walk.unwrap().borrow().next.clone();
    }

    None
}

/// Modify process node name information
pub fn rename_proc(this: Rc<RefCell<Proc>>, comm: String, uid: i32, by_pid: bool) {
    this.borrow_mut().comm = comm;
    this.borrow_mut().uid = uid;

    // Re-sort children in parent, now we have a name
    if !by_pid && this.borrow().parent.is_some() {
        let parent = this.borrow().parent.clone().unwrap();
        let mut walk = parent.borrow().children.clone();
        while walk.is_some() {
            let now_node = walk.unwrap();
            if now_node.borrow().next.is_some()
                && now_node.clone().borrow().child.borrow().comm
                    > now_node
                        .clone()
                        .borrow()
                        .next
                        .clone()
                        .unwrap()
                        .borrow()
                        .child
                        .borrow()
                        .comm
            {
                let tmp = now_node.borrow().child.clone();
                now_node.borrow_mut().child = now_node
                    .borrow()
                    .next
                    .clone()
                    .unwrap()
                    .borrow()
                    .child
                    .clone();
                now_node.borrow().next.clone().unwrap().borrow_mut().child = tmp.clone();
            }
            walk = now_node.borrow().next.clone();
        }
    }
}

/// Add process node
pub fn add_proc(
    comm: String,
    pid: i32,
    ppid_pgid_uid: (i32, i32, i32),
    args: Option<String>,
    isthread: bool,
    process_age_sec: f64,
    config: &Config,
) {
    let mut this: Option<Rc<RefCell<Proc>>>;
    let (mut ppid, pgid, uid) = ppid_pgid_uid;
    this = find_proc(pid, config.list_head.borrow().head.clone());
    if this.is_none() {
        this = new_proc(comm, pid, uid, config.list_head.clone());
    } else {
        rename_proc(this.clone().unwrap(), comm, uid, config.numeric_sort);
    }
    let this = this.unwrap();

    if let Some(tmp_arg) = args {
        set_args(this.clone(), tmp_arg);
    }
    if pid == ppid {
        ppid = 0;
    }
    this.borrow_mut().pgid = pgid;
    this.borrow_mut().age = process_age_sec;
    if isthread {
        this.borrow_mut().flags |= PFLAG_THREAD;
    }

    let mut parent = find_proc(ppid, config.list_head.borrow().head.clone());
    if parent.is_none() {
        parent = new_proc("?".to_string(), ppid, 0, config.list_head.clone());
    }

    if pid != 0 {
        add_child(parent.clone().unwrap(), this.clone(), config.numeric_sort);
        this.borrow_mut().parent = parent;
    }
}

/// read_proc now uses a similar method as procps for finding the process
/// name in the /proc filesystem.
pub fn read_proc(root_pid: i32, config: &Config) -> UResult<()> {
    let mut empty: bool = true;
    let dir = match fs::read_dir(PROC_BASE) {
        Ok(dir) => dir,
        Err(e) => {
            return Err(USimpleError::new(1, format!("{}", e)));
        }
    };

    for de in dir {
        let entry = de?;
        let path = entry.path();
        let file_name = path.file_name().unwrap().to_str().unwrap();
        if path.is_dir() {
            if let Ok(pid) = file_name.parse::<i32>() {
                let stat_path = path.join("stat");
                let stat_file = fs::File::open(stat_path);
                if stat_file.is_err() {
                    continue;
                }
                empty = false;

                let st: fs::Metadata;
                if let Ok(st_tmp) = fs::metadata(&path) {
                    st = st_tmp;
                } else {
                    continue;
                }

                let mut file = stat_file.unwrap();
                let mut readbuf = String::new();
                let _size = match file.read_to_string(&mut readbuf) {
                    Ok(size) => size,
                    Err(_) => continue,
                };
                //find commands between()
                if let (Some(start), Some(end)) = (readbuf.find('('), readbuf.find(')')) {
                    let command = readbuf[start + 1..end].to_string();
                    let rest = &readbuf[end + 1..];
                    let parts: Vec<&str> = rest.split_whitespace().collect();
                    if let (Some(ppid_str), Some(pgid_str), Some(proc_stt_jf_str)) =
                        (parts.get(1), parts.get(2), parts.get(19))
                    {
                        if let (Ok(ppid), Ok(pgid), Ok(proc_stt_jf)) = (
                            ppid_str.parse::<i32>(),
                            pgid_str.parse::<i32>(),
                            proc_stt_jf_str.parse::<u64>(),
                        ) {
                            let process_age_sec = process_age(proc_stt_jf);

                            // handle process threads
                            if !config.hide_threads {
                                let task_path = path.join("task");
                                let task_dir = fs::read_dir(task_path);
                                if let Ok(task_dir) = task_dir {
                                    // if we have this dir, we're on 2.6
                                    for task_de in task_dir {
                                        let task_entry = task_de?;
                                        if let Ok(thread) = task_entry
                                            .file_name()
                                            .into_string()
                                            .unwrap()
                                            .parse::<i32>()
                                        {
                                            if thread != pid {
                                                let threadname = get_threadname(
                                                    pid,
                                                    thread,
                                                    command.clone(),
                                                    config.thread_names,
                                                );
                                                if config.arguments {
                                                    add_proc(
                                                        threadname.clone(),
                                                        thread,
                                                        (pid, pgid, st.uid() as i32),
                                                        Some(threadname.clone()),
                                                        true,
                                                        process_age_sec,
                                                        config,
                                                    );
                                                } else {
                                                    add_proc(
                                                        threadname,
                                                        thread,
                                                        (pid, pgid, st.uid() as i32),
                                                        None,
                                                        true,
                                                        process_age_sec,
                                                        config,
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            // handle process
                            if !config.arguments {
                                add_proc(
                                    command.clone(),
                                    pid,
                                    (ppid, pgid, st.uid() as i32),
                                    None,
                                    false,
                                    process_age_sec,
                                    config,
                                );
                            } else {
                                let path_cmdline = path.join("cmdline");
                                let cmd_file = fs::File::open(path_cmdline);
                                if cmd_file.is_err() {
                                    // If this fails then the process is gone.  If a PID
                                    // was specified on the command-line then we might
                                    // not even be interested in the current process.
                                    // There's no sensible way of dealing with this race
                                    // so we might as well behave as if the current
                                    // process did not exist.
                                    continue;
                                }
                                let mut cmd_file = cmd_file.unwrap();
                                let mut buffer = String::new();
                                let _ = match cmd_file.read_to_string(&mut buffer) {
                                    Ok(size) => size,
                                    Err(_) => continue,
                                };

                                add_proc(
                                    command.clone(),
                                    pid,
                                    (ppid, pgid, st.uid() as i32),
                                    Some(buffer),
                                    false,
                                    process_age_sec,
                                    config,
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    fix_orphans(root_pid, config.list_head.clone(), config.numeric_sort);
    if empty {
        return Err(USimpleError::new(1, "/proc is empty (not mounted ?)\n"));
    }
    Ok(())
}

/// When using kernel 3.3 with hidepid feature enabled on /proc
/// then we need fake root pid and gather all the orphan processes
/// that is, processes with no known parent
/// As we cannot be sure if it is just the root pid or others missing
/// we gather the lot
pub fn fix_orphans(pid: i32, list_head: Rc<RefCell<HeadProc>>, by_pid: bool) {
    let root: Rc<RefCell<Proc>>;
    if let Some(root_tmp) = find_proc(pid, list_head.borrow().head.clone()) {
        root = root_tmp;
    } else {
        root = new_proc("?".to_string(), pid, 0, list_head.clone()).unwrap();
    }

    let mut walk = list_head.borrow().head.clone();
    while walk.is_some() {
        let now_node = walk.clone().unwrap();
        if now_node.borrow().pid == 1 || now_node.borrow().pid == 0 {
            walk = now_node.borrow().next.clone();
            continue;
        }
        if now_node.borrow().parent.is_none() {
            add_child(root.clone(), now_node.clone(), by_pid);
            walk.unwrap().borrow_mut().parent = Some(root.clone());
        }
        walk = now_node.borrow().next.clone();
    }
}

/// Add process node namespace information
pub fn new_proc_ns(ns_task: Rc<RefCell<Proc>>) {
    for i in 0..NUM_NS {
        let path = format!("/proc/{}/ns/{}", ns_task.borrow().pid, get_ns_name(i));
        if let Ok(st) = fs::metadata(&path) {
            ns_task.borrow_mut().ns[i] = st.ino();
        }
    }
}

/// Determine if the tree structure is equal
pub fn tree_equal(
    a: Rc<RefCell<Proc>>,
    b: Rc<RefCell<Proc>>,
    user_change: bool,
    ns_change: bool,
) -> bool {
    if a.borrow().comm != b.borrow().comm {
        return false;
    }
    if user_change && a.borrow().uid != b.borrow().uid {
        return false;
    }
    if ns_change {
        for i in 0..NUM_NS {
            if a.borrow().ns[i] != b.borrow().ns[i] {
                return false;
            }
        }
    }

    let mut walk_a = a.borrow().children.clone();
    let mut walk_b = b.borrow().children.clone();
    while walk_a.is_some() && walk_b.is_some() {
        if !tree_equal(
            walk_a.clone().unwrap().borrow().child.clone(),
            walk_b.clone().unwrap().borrow().child.clone(),
            user_change,
            ns_change,
        ) {
            return false;
        }

        walk_a = walk_a.unwrap().borrow().next.clone();
        walk_b = walk_b.unwrap().borrow().next.clone();
    }

    !(walk_a.is_some() || walk_b.is_some())
}

/// Output single character
pub fn out_char(c: char, trunc: bool, outputargs: &mut OutputArgs, output_width: i32) {
    outputargs.cur_x += 1;

    if !trunc || outputargs.cur_x <= output_width as usize {
        print!("{}", c);
    } else if trunc && outputargs.cur_x == (output_width + 1) as usize {
        print!("+");
    }

    io::stdout().flush().unwrap();
}

/// Output string
pub fn out_string(str: String, trunc: bool, outputargs: &mut OutputArgs, output_width: i32) {
    for c in str.chars() {
        if c == '\0' {
            break;
        }
        out_char(c, trunc, outputargs, output_width);
    }
}

/// Output int
pub fn out_int(x: i32, trunc: bool, outputargs: &mut OutputArgs, output_width: i32) -> i32 {
    let mut digits = 0;
    let mut div = 1;

    while x / div != 0 {
        digits += 1;
        div *= 10;
    }

    if digits == 0 {
        digits = 1;
    }

    div /= 10;
    while div != 0 {
        out_char(
            char::from_digit(((x / div) % 10) as u32, 10).unwrap(),
            trunc,
            outputargs,
            output_width,
        );
        div /= 10;
    }

    digits
}

/// Output parameters
pub fn out_args(mystr: String, trunc: bool, outputargs: &mut OutputArgs, output_width: i32) -> i32 {
    let mut strcount: i32 = 0;

    for c in mystr.chars() {
        if c == '\\' {
            out_string("\\\\".to_string(), trunc, outputargs, output_width);
            strcount += 2;
        } else if (' '..='~').contains(&c) {
            out_char(c, trunc, outputargs, output_width);
            strcount += 1;
        } else {
            let tmpstr = format!("\\{:03o}", c as u32);
            out_string(tmpstr, trunc, outputargs, output_width);
            strcount += 4;
        }
    }

    strcount
}

/// Output newline
pub fn out_newline(outputargs: &mut OutputArgs) {
    println!();
    outputargs.cur_x = 1;
}

/// Print the security context of the current process. This is largely lifted
/// from pr_context from procps ps/output.c
pub fn out_scontext(
    current: Rc<RefCell<Proc>>,
    trunc: bool,
    outputargs: &mut OutputArgs,
    output_width: i32,
) {
    out_string("`".to_string(), trunc, outputargs, output_width);
    let path = format!("/proc/{}/attr/current", current.borrow().pid);
    if let Ok(file) = fs::File::open(path) {
        let mut buf_reader = BufReader::new(file);
        let mut readbuf = String::new();
        if buf_reader.read_line(&mut readbuf).is_ok() {
            out_string(readbuf, trunc, outputargs, output_width)
        }
    }
    out_string("'".to_string(), trunc, outputargs, output_width);
}

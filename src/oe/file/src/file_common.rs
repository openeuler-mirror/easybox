//! This file is part of the easybox package.
//
// (c) Zhihua Zhao <YuukaC@outlook.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use std::fs::File;
use std::io::{stdout, BufRead, BufReader, Read, Write};

use uucore::error::{UResult, USimpleError, UUsageError};
use uucore::msg_log::{err, errx, warn, warnx};
use uucore::{format_usage, util_name};

use clap::{crate_version, Arg, Command};

use crate::file_utils::*;
use crate::{file_magic::*, oe_app};

/// file cmd parse error code.
pub static FILE_CMD_PARSE_ERROR: i32 = 1;

struct Param {
    name: String,
    value: Option<usize>,
    tag: i32,
}

impl Param {
    pub fn new(name: &str, tag: i32) -> Self {
        Self {
            name: name.to_string(),
            value: None,
            tag,
        }
    }
}

/// Config
#[derive(Default)]
pub struct Config {
    ///
    pub files: Vec<String>,

    ///
    pub magic_file: Option<String>,
    ///
    pub uncompress: bool,
    ///
    pub uncompress_noreport: bool,
    ///
    pub brief: u8,
    ///
    pub checking_printout: bool,
    ///
    pub stop_on_error: bool,
    ///
    pub exclude: Vec<String>,
    ///
    pub exclude_quiet: Vec<String>,
    ///
    pub files_from: String,
    ///
    pub separator: String,
    ///
    pub mime: bool,
    ///
    pub apple: bool,
    ///
    pub extension: bool,
    ///
    pub mime_type: bool,
    ///
    pub mime_encoding: bool,
    ///
    pub keep_going: bool,
    ///
    pub list: bool,
    ///
    pub dereference: bool,
    ///
    pub no_dereference: bool,
    ///
    pub no_buffer: bool,
    ///
    pub no_pad: bool,
    ///
    pub print0: u8,
    ///
    pub preserve_date: bool,
    ///
    pub parameter: Vec<String>,
    ///
    pub raw: bool,
    ///
    pub special_files: bool,
    ///
    #[cfg(feature = "sandbox")]
    pub no_sandbox: bool,
    ///
    pub compile: bool,
    ///
    pub debug: bool,
}

pub mod options {
    ///
    pub static MAGIC_FILE: &str = "magic-file";
    ///
    pub static UNCOMPRESS: &str = "uncompress";
    ///
    pub static UNCOMPRESS_NOREPORT: &str = "uncompress-noreport";
    ///
    pub static BRIEF: &str = "brief";
    ///
    pub static CHECKING_PRINTOUT: &str = "checking-printout";
    ///
    pub static STOP_ON_ERROR: &str = "stop-on-error";
    ///
    pub static EXCLUDE: &str = "exclude";
    ///
    pub static EXCLUDE_QUIET: &str = "exclude-quiet";
    ///
    pub static FILES_FROM: &str = "files-from";
    ///
    pub static SEPARATOR: &str = "separator";
    ///
    pub static MIME: &str = "mime";
    ///
    pub static APPLE: &str = "apple";
    ///
    pub static EXTENSION: &str = "extension";
    ///
    pub static MIME_TYPE: &str = "mime-type";
    ///
    pub static MIME_ENCODING: &str = "mime-encoding";
    ///
    pub static KEEP_GOING: &str = "keep-going";
    ///
    pub static LIST: &str = "list";
    ///
    pub static DEREFERENCE: &str = "dereference";
    ///
    pub static NO_DEREFERENCE: &str = "no-dereference";
    ///
    pub static NO_BUFFER: &str = "no-buffer";
    ///
    pub static NO_PAD: &str = "no-pad";
    ///
    pub static PRINT0: &str = "print0";
    ///
    pub static PRESERVE_DATE: &str = "preserve-date";
    ///
    pub static PARAMETER: &str = "parameter";
    ///
    pub static RAW: &str = "raw";
    ///
    pub static SPECIAL_FILES: &str = "special-files";
    ///
    pub static NO_SANDBOX: &str = "no-sandbox";
    ///
    pub static COMPILE: &str = "compile";
    ///
    pub static DEBUG: &str = "debug";
}

impl Config {
    ///
    pub fn from(options: &clap::ArgMatches) -> UResult<Self> {
        if let Some(index) = options.index_of(options::FILES_FROM) {
            for arg in [options::CHECKING_PRINTOUT, options::COMPILE, options::LIST] {
                if let Some(p) = options.index_of(arg) {
                    if p < index {
                        println!("{}", oe_app().render_usage());
                        return Err(UUsageError::new(
                            FILE_CMD_PARSE_ERROR,
                            "Cannot specify -f after -c, -C, or -l",
                        ));
                    }
                }
            }
        }

        Ok(Self {
            files: options
                .get_many::<String>("files")
                .unwrap_or_default()
                .cloned()
                .collect::<Vec<_>>(),

            magic_file: options.get_one::<String>(options::MAGIC_FILE).cloned(),
            uncompress: options.contains_id(options::UNCOMPRESS),
            uncompress_noreport: options.contains_id(options::UNCOMPRESS_NOREPORT),
            brief: options.get_count(options::BRIEF),
            checking_printout: options.contains_id(options::CHECKING_PRINTOUT),
            stop_on_error: options.contains_id(options::STOP_ON_ERROR),
            exclude: options
                .get_many::<String>(options::EXCLUDE)
                .unwrap_or_default()
                .cloned()
                .collect::<Vec<_>>(),
            exclude_quiet: options
                .get_many::<String>(options::EXCLUDE_QUIET)
                .unwrap_or_default()
                .cloned()
                .collect::<Vec<_>>(),
            files_from: options
                .get_one::<String>(options::FILES_FROM)
                .unwrap_or(&String::new())
                .clone(),
            separator: options
                .get_one::<String>(options::SEPARATOR)
                .unwrap_or(&String::from(":"))
                .clone(),
            mime: options.contains_id(options::MIME),
            apple: options.contains_id(options::APPLE),
            extension: options.contains_id(options::EXTENSION),
            mime_type: options.contains_id(options::MIME_TYPE),
            mime_encoding: options.contains_id(options::MIME_ENCODING),
            keep_going: options.contains_id(options::KEEP_GOING),
            list: options.contains_id(options::LIST),
            dereference: options.contains_id(options::DEREFERENCE),
            no_dereference: options.contains_id(options::NO_DEREFERENCE),
            no_buffer: options.contains_id(options::NO_BUFFER),
            no_pad: options.contains_id(options::NO_PAD),
            print0: options.get_count(options::PRINT0),
            preserve_date: options.contains_id(options::PRESERVE_DATE),
            parameter: options
                .get_many::<String>(options::PARAMETER)
                .unwrap_or_default()
                .cloned()
                .collect::<Vec<_>>(),
            raw: options.contains_id(options::RAW),
            special_files: options.contains_id(options::SPECIAL_FILES),
            #[cfg(feature = "sandbox")]
            no_sandbox: options.contains_id(options::NO_SANDBOX),
            compile: options.contains_id(options::COMPILE),
            debug: options.contains_id(options::DEBUG),
        })
    }
}

pub fn parse_file_cmd_args(args: impl uucore::Args, about: &str, usage: &str) -> UResult<Config> {
    let command = base_app(about, usage);
    let arg_list = args.collect_lossy();
    Config::from(&command.try_get_matches_from(arg_list)?)
}

pub fn base_app<'a>(about: &'a str, usage: &'a str) -> Command<'a> {
    Command::new(util_name())
        .version(crate_version!())
        .about(about)
        .override_usage(format_usage(usage))
        .infer_long_args(true)
        // Format arguments.
        .arg(
            Arg::new(options::MAGIC_FILE)
                .short('m')
                .long(options::MAGIC_FILE)
                .takes_value(true)
                .value_name("LIST")
                .help("use LIST as a colon-separated list of magic number files")
        )
        .arg(
            Arg::new(options::UNCOMPRESS)
                .short('z')
                .long(options::UNCOMPRESS)
                .help("try to look inside compressed files"),
        )
        .arg(
            Arg::new(options::UNCOMPRESS_NOREPORT)
                .short('Z')
                .long(options::UNCOMPRESS_NOREPORT)
                .help("only print the contents of compressed files"),
        )
        .arg(
            Arg::new(options::BRIEF)
                .short('b')
                .long(options::BRIEF)
                .action(clap::ArgAction::Count)
                .help("do not prepend filenames to output lines"),
        )
        .arg(
            Arg::new(options::CHECKING_PRINTOUT)
                .short('c')
                .long(options::CHECKING_PRINTOUT)
                .help("print the parsed form of the magic file, use in conjunction with -m to debug a new magic file before installing it"),
        )
        .arg(
            Arg::new(options::STOP_ON_ERROR)
                .short('E')
                .help("issue an error message and exit on filesystem errors")
        )
        .arg(
            Arg::new(options::EXCLUDE)
                .short('e')
                .long(options::EXCLUDE)
                .action(clap::ArgAction::Append)
                .takes_value(true)
                .value_name("TEST")
                .help("exclude TEST from the list of test to be performed for file. Valid tests are: apptype, ascii, cdf, compress, csv, elf, encoding, soft, tar, json, simh, text, tokens"),
        )
        .arg(
            Arg::new(options::EXCLUDE_QUIET)
                .long(options::EXCLUDE_QUIET)
                .action(clap::ArgAction::Append)
                .takes_value(true)
                .value_name("TEST")
                .help("like exclude, but ignore unknown tests"),
        )
        .arg(
            Arg::new(options::FILES_FROM)
                .short('f')
                .long(options::FILES_FROM)
                .takes_value(true)
                .value_name("FILE")
                .help("read the filenames to be examined from FILE"),
        )
        .arg(
            Arg::new(options::SEPARATOR)
                .short('F')
                .long(options::SEPARATOR)
                .takes_value(true)
                .value_name("STRING")
                .help("use string as separator instead of `:'"),
        )
        .arg(
            Arg::new(options::MIME)
                .short('i')
                .long(options::MIME)
                .help("output MIME type strings (--mime-type and --mime-encoding)"),
        )
        .arg(
            Arg::new(options::APPLE)
                .long(options::APPLE)
                .help("output the Apple CREATOR/TYPE"),
        )
        .arg(
            Arg::new(options::EXTENSION)
                .long(options::EXTENSION)
                .help("output a slash-separated list of extensions"),
        )
        .arg(
            Arg::new(options::MIME_TYPE)
                .long(options::MIME_TYPE)
                .help("output the MIME type"),
        )
        .arg(
            Arg::new(options::MIME_ENCODING)
                .long(options::MIME_ENCODING)
                .help("output the MIME encoding"),
        )
        .arg(
            Arg::new(options::KEEP_GOING)
                .short('k')
                .long(options::KEEP_GOING)
                .help("don't stop at the first match"),
        )
        .arg(
            Arg::new(options::LIST)
                .short('l')
                .long(options::LIST)
                .help("list magic strength"),
        )
        .arg(
            Arg::new(options::DEREFERENCE)
                .short('L')
                .long(options::DEREFERENCE)
                .help("follow symlinks (default if POSIXLY_CORRECT is set)"),
        )
        .arg(
            Arg::new(options::NO_DEREFERENCE)
                .short('h')
                .long(options::NO_DEREFERENCE)
                .help("don't follow symlinks (default if POSIXLY_CORRECT is not set) (default)"),
        )
        .arg(
            Arg::new(options::NO_BUFFER)
                .short('n')
                .long(options::NO_BUFFER)
                .help("do not buffer output"),
        )
        .arg(
            Arg::new(options::NO_PAD)
                .short('N')
                .long(options::NO_PAD)
                .help("do not pad output"),
        )
        .arg(
            Arg::new(options::PRINT0)
                .short('0')
                .long(options::PRINT0)
                .action(clap::ArgAction::Count)
                .help("terminate filenames with ASCII NUL"),
        )
        .arg(
            Arg::new(options::PRESERVE_DATE)
                .short('p')
                .long(options::PRESERVE_DATE)
                .help("preserve access times on files"),
        )
        .arg(
            Arg::new(options::PARAMETER)
                .short('P')
                .long(options::PARAMETER)
                .action(clap::ArgAction::Append)
                .takes_value(true)
                .value_name("PARAMETER")
                .help("set file engine parameter limits"),
        )
        .arg(
            Arg::new(options::RAW)
                .short('r')
                .long(options::RAW)
                .help("don't translate unprintable chars to \\ooo"),
        )
        .arg(
            Arg::new(options::SPECIAL_FILES)
                .short('s')
                .long(options::SPECIAL_FILES)
                .help("treat special (block/char devices) files as ordinary ones"),
        )
        .arg(
            Arg::new(options::NO_SANDBOX)
                .short('S')
                .long(options::NO_SANDBOX)
                .help("disable system call sandboxing"),
        )
        .arg(
            Arg::new(options::COMPILE)
                .short('C')
                .long(options::COMPILE)
                .help("compile file specified by -m"),
        )
        .arg(
            Arg::new(options::DEBUG)
                .short('d')
                .long(options::DEBUG)
                .help("print debugging messages"),
        )
        .arg(
            Arg::new("files")
                .takes_value(true)
                .multiple(true)
        )
}

pub fn handle_input(config: &Config) -> UResult<()> {
    // Set the locale, wcwidth() needs it
    #[cfg(feature = "wide")]
    crate::file_unsafe::setlocale();

    let mut flags = MAGIC_NONE;
    let mut action = MAGIC_NONE;
    let mut magic: magic_t = std::ptr::null_mut();
    let mut didsomefiles = false;
    let mut params = vec![
        Param::new("bytes", MAGIC_PARAM_BYTES_MAX),
        Param::new("elf_notes", MAGIC_PARAM_ELF_NOTES_MAX),
        Param::new("elf_phnum", MAGIC_PARAM_ELF_PHNUM_MAX),
        Param::new("elf_shnum", MAGIC_PARAM_ELF_SHNUM_MAX),
        Param::new("elf_shsize", MAGIC_PARAM_ELF_SHSIZE_MAX),
        Param::new("encoding", MAGIC_PARAM_ENCODING_MAX),
        Param::new("indir", MAGIC_PARAM_INDIR_MAX),
        Param::new("name", MAGIC_PARAM_NAME_MAX),
        Param::new("regex", MAGIC_PARAM_REGEX_MAX),
        Param::new("magwarn", MAGIC_PARAM_MAGWARN_MAX),
    ];

    if std::env::var("POSIXLY_CORRECT").is_ok() {
        flags |= MAGIC_SYMLINK;
    }

    if config.apple {
        flags |= MAGIC_APPLE;
    }
    if config.extension {
        flags |= MAGIC_EXTENSION;
    }
    if config.mime_type {
        flags |= MAGIC_MIME_TYPE;
    }
    if config.mime_encoding {
        flags |= MAGIC_MIME_ENCODING;
    }
    if config.debug {
        flags |= MAGIC_DEBUG | MAGIC_CHECK;
    }
    if config.stop_on_error {
        flags |= MAGIC_ERROR;
    }
    if config.mime {
        flags |= MAGIC_MIME;
    }
    if config.keep_going {
        flags |= MAGIC_CONTINUE;
    }
    if config.raw {
        flags |= MAGIC_RAW;
    }
    if config.special_files {
        flags |= MAGIC_DEVICES;
    }
    if config.uncompress {
        flags |= MAGIC_COMPRESS;
    }
    if config.uncompress_noreport {
        flags |= MAGIC_COMPRESS | MAGIC_COMPRESS_TRANSP;
    }

    if config.preserve_date {
        flags |= MAGIC_PRESERVE_ATIME;
    }

    if config.checking_printout {
        action = FILE_CHECK;
    }
    if config.compile {
        action = FILE_COMPILE;
    }
    if config.list {
        action = FILE_LIST;
    }

    if config.dereference {
        flags |= MAGIC_SYMLINK;
    }
    if config.no_dereference {
        flags &= !MAGIC_SYMLINK;
    }

    for test in &config.exclude {
        match test.as_str() {
            "apptype" => flags |= MAGIC_NO_CHECK_APPTYPE,
            "ascii" => flags |= MAGIC_NO_CHECK_TEXT,
            "cdf" => flags |= MAGIC_NO_CHECK_CDF,
            "compress" => flags |= MAGIC_NO_CHECK_COMPRESS,
            "csv" => flags |= MAGIC_NO_CHECK_CSV,
            "elf" => flags |= MAGIC_NO_CHECK_ELF,
            "encoding" => flags |= MAGIC_NO_CHECK_ENCODING,
            "soft" => flags |= MAGIC_NO_CHECK_SOFT,
            "tar" => flags |= MAGIC_NO_CHECK_TAR,
            "json" => flags |= MAGIC_NO_CHECK_JSON,
            "simh" => flags |= MAGIC_NO_CHECK_SIMH,
            "text" => flags |= MAGIC_NO_CHECK_TEXT,
            "tokens" => flags |= MAGIC_NO_CHECK_TOKENS,
            _ => {
                println!("{}", oe_app().render_usage());
                return Err(USimpleError::new(
                    FILE_CMD_PARSE_ERROR,
                    format!("Invalid test: {}", test),
                ));
            }
        }
    }

    for test in &config.exclude_quiet {
        match test.as_str() {
            "apptype" => flags |= MAGIC_NO_CHECK_APPTYPE,
            "ascii" => flags |= MAGIC_NO_CHECK_TEXT,
            "cdf" => flags |= MAGIC_NO_CHECK_CDF,
            "compress" => flags |= MAGIC_NO_CHECK_COMPRESS,
            "csv" => flags |= MAGIC_NO_CHECK_CSV,
            "elf" => flags |= MAGIC_NO_CHECK_ELF,
            "encoding" => flags |= MAGIC_NO_CHECK_ENCODING,
            "soft" => flags |= MAGIC_NO_CHECK_SOFT,
            "tar" => flags |= MAGIC_NO_CHECK_TAR,
            "json" => flags |= MAGIC_NO_CHECK_JSON,
            "simh" => flags |= MAGIC_NO_CHECK_SIMH,
            "text" => flags |= MAGIC_NO_CHECK_TEXT,
            "tokens" => flags |= MAGIC_NO_CHECK_TOKENS,
            _ => (),
        }
    }

    if !config.files_from.is_empty() {
        if magic.is_null() {
            magic = load(flags, config);
            if magic.is_null() {
                return Err(USimpleError::new(
                    FILE_CMD_PARSE_ERROR,
                    "Cannot load magic file",
                ));
            }
        }
        apply_param(magic, &params);
        unwrap(magic, &config.files_from, config)?;
        didsomefiles = true;
    }

    'param: for expression in &config.parameter {
        let vec = expression.split('=').collect::<Vec<_>>();
        if vec.len() == 2 {
            if let Ok(v) = vec[1].parse::<usize>() {
                for p in &mut params {
                    if p.name == vec[0] {
                        p.value = Some(v);
                        continue 'param;
                    }
                }
            }
        }
        errx(
            FILE_CMD_PARSE_ERROR,
            &format!("Unknown param {}", expression),
        );
    }

    #[cfg(feature = "sandbox")]
    if !config.no_sandbox {
        if !enable_sandbox_full() {
            err(FILE_CMD_PARSE_ERROR, "SECCOMP initialisation failed");
        }
        flags |= MAGIC_NO_COMPRESS_FORK;
    }

    match action {
        FILE_CHECK | FILE_COMPILE | FILE_LIST => {
            magic = magic_open(flags | MAGIC_CHECK);
            if magic.is_null() {
                warn("Can't create magic");
                return Err(USimpleError::new(
                    FILE_CMD_PARSE_ERROR,
                    "Can't create magic",
                ));
            }

            if match action {
                FILE_CHECK => magic_check(magic, config.magic_file.as_deref()),
                FILE_COMPILE => magic_compile(magic, config.magic_file.as_deref()),
                FILE_LIST => magic_list(magic, config.magic_file.as_deref()),
                _ => panic!("Invalid action"),
            } == -1
            {
                warnx(&magic_error(magic).unwrap_or_default());
                if !config.no_buffer {
                    stdout().flush()?;
                }
                if !magic.is_null() {
                    magic_close(magic);
                }
                return Err(USimpleError::new(FILE_CMD_PARSE_ERROR, "action failed"));
            }
            didsomefiles = true;
        }
        _ => {
            if magic.is_null() {
                magic = load(flags, config);
                if magic.is_null() {
                    return Err(USimpleError::new(
                        FILE_CMD_PARSE_ERROR,
                        "Cannot load magic file",
                    ));
                }
            }
            apply_param(magic, &params);
        }
    }

    if config.files.is_empty() && !didsomefiles {
        println!("{}", oe_app().render_usage());

        if !config.no_buffer {
            stdout().flush()?;
        }
        if !magic.is_null() {
            magic_close(magic);
        }

        return Ok(());
    }

    let mut width = 0;
    for f in &config.files {
        width = width.max(file_mbswidth(f, config.raw));
    }

    for f in &config.files {
        process(magic, f, config, width);
    }

    if !config.no_buffer {
        stdout().flush()?;
    }
    if !magic.is_null() {
        magic_close(magic);
    }

    Ok(())
}

fn load(flags: i32, config: &Config) -> magic_t {
    let magic = magic_open(flags);

    if magic.is_null() {
        warn("Can't create magic");
        return std::ptr::null_mut();
    }
    if magic_load(magic, config.magic_file.as_deref()) == -1 {
        warn(&magic_error(magic).unwrap_or_default());
        magic_close(magic);
        return std::ptr::null_mut();
    }
    if let Some(e) = magic_error(magic) {
        warn(&e);
    }

    magic
}

fn apply_param(magic: magic_t, params: &Vec<Param>) {
    for p in params {
        if let Some(value) = p.value {
            if magic_setparam(magic, p.tag, value) == -1 {
                err(FILE_CMD_PARSE_ERROR, &format!("Can't set {}", p.name));
            }
        }
    }
}

fn unwrap(magic: magic_t, namefile: &str, config: &Config) -> UResult<()> {
    let mut reader = BufReader::new(if namefile == "-" {
        Box::new(std::io::stdin()) as Box<dyn Read>
    } else {
        Box::new(File::open(namefile)?) as Box<dyn Read>
    });
    let mut files = Vec::new();
    let mut max_width = 0;

    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 {
            break;
        }
        line = line.trim().to_string();
        let width = file_mbswidth(&line, config.raw);
        if config.no_buffer {
            if process(magic, &line, config, width) {
                return Err(USimpleError::new(FILE_CMD_PARSE_ERROR, "process failed"));
            }
            continue;
        }
        max_width = max_width.max(width);
        files.push(line);
    }

    if !config.no_buffer {
        for f in files {
            if process(magic, &f, config, max_width) {
                return Err(USimpleError::new(FILE_CMD_PARSE_ERROR, "process failed"));
            }
        }
    }

    Ok(())
}

fn process(magic: magic_t, name: &str, config: &Config, width: usize) -> bool {
    let c = if config.print0 > 1 { '\0' } else { '\n' };
    let std_in = name == "-";
    let mut haderror = false;
    let bflag = if config.brief == 2 {
        !config.files.is_empty()
    } else {
        config.brief != 0
    };

    if width > 0 && !bflag {
        let pname = if std_in { "/dev/stdin" } else { name };
        if !config.raw {
            fname_print(pname);
        } else {
            print!("{}", pname);
        }
        if config.print0 != 0 {
            print!("\0");
        }
        if config.print0 < 2 {
            print!(
                "{}{:width$} ",
                config.separator,
                "",
                width = if config.no_pad {
                    0
                } else {
                    width - file_mbswidth(name, config.raw)
                }
            );
        }
    }

    let t = magic_file(magic, if std_in { None } else { Some(name) });

    if t.is_none() {
        print!("ERROR: {}{}", magic_error(magic).unwrap_or_default(), c);
        haderror = true;
    } else {
        haderror |= write!(stdout(), "{}{}", t.unwrap(), c).is_err();
    }
    if config.no_buffer {
        haderror |= std::io::stdout().flush().is_err();
    }
    haderror
}

//! This file is part of the easybox package.
//
// (c) Wentong Yang <ywt0821@163.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use clap::{crate_version, Arg, ArgGroup, ArgMatches, Command};
use libc::{setlocale, EXIT_FAILURE, LC_ALL};
use serde::Serialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self};
use std::path::Path;
use uucore::{
    error::{UResult, USimpleError},
    format_usage,
};

use crate::lib_column::{
    fillcols_main, fillrows_main, get_terminal_width, parse_columns, read_input, simple_main,
    table_main, validate_args,
};

/// TableRow for json
#[derive(Debug, Serialize)]
pub struct TableRow {
    #[serde(flatten)]
    /// column
    pub columns: HashMap<String, String>,
}

/// ColumnMode
#[derive(PartialEq)]
pub enum ColumnMode {
    /// FillCols mode
    FillCols,
    /// FillRows mode
    FillRows,
    /// Table mode
    Table,
    /// Simple mode
    Simple,
}

/// Config
pub struct Config {
    ///
    pub mode: ColumnMode,
    ///
    pub table_columns: Option<Vec<String>>,
    ///
    pub input_files: Option<Vec<String>>,
    ///
    pub termwidth: Option<usize>,
    ///
    pub ents: Vec<Vec<String>>,
    ///
    pub input_separator: Option<String>,
    ///
    pub output_separator: Option<String>,
    ///
    pub table_order: Option<Vec<String>>,
    ///
    pub table_columns_limit: Option<usize>,
    ///
    pub table_noheadings: bool,
    ///
    pub table_hide: Option<Vec<usize>>,
    ///
    pub table_right: Option<Vec<usize>>,
    ///
    pub keep_empty_lines: bool,
    ///
    pub maxlength: usize,
    ///
    pub tree: Option<String>,
    ///
    pub tree_id: Option<String>,
    ///
    pub tree_parent: Option<String>,
    ///
    pub table_name: Option<String>,
    ///
    pub json: bool,
    ///
    pub table_truncate: Option<Vec<usize>>,
    ///
    pub table_wrap: Option<Vec<usize>>,
    ///
    pub table_noextreme: Option<Vec<usize>>,
    ///
    pub table_header_repeat: bool,
}

/// Command Options
pub mod options {
    /// --table
    pub static TABLE: &str = "table";
    /// --table-name <name>
    pub static TABLE_NAME: &str = "table-name";
    /// --table-order <columns>
    pub static TABLE_ORDER: &str = "table-order";
    /// --table-columns <names>
    pub static TABLE_COLUMNS: &str = "table-columns";
    /// --table-columns-limit <num>
    pub static TABLE_COLUMNS_LIMIT: &str = "table-columns-limit";
    /// --table-noextreme <columns>
    pub static TABLE_NOEXTREME: &str = "table-noextreme";
    /// --table-noheadings
    pub static TABLE_NOHEADINGS: &str = "table-noheadings";
    /// --table-header-repeat
    pub static TABLE_HEADER_REPEAT: &str = "table-header-repeat";
    /// --table-hide <columns>
    pub static TABLE_HIDE: &str = "table-hide";
    /// --table-right <columns>
    pub static TABLE_RIGHT: &str = "table-right";
    /// --table-truncate <columns>
    pub static TABLE_TRUNCATE: &str = "table-truncate";
    /// --table-wrap <columns>
    pub static TABLE_WRAP: &str = "table-wrap";
    /// --keep-empty-lines
    pub static KEEP_EMPTY_LINES: &str = "keep-empty-lines";
    /// --json
    pub static JSON: &str = "json";
    /// --tree <column>
    pub static TREE: &str = "tree";
    /// --tree-id <column>
    pub static TREE_ID: &str = "tree-id";
    /// --tree-parent <column>
    pub static TREE_PARENT: &str = "tree-parent";
    /// --output-width <width>
    pub static OUTPUT_WIDTH: &str = "output-width";
    /// --output-separator <string>
    pub static OUTPUT_SEPARATOR: &str = "output-separator";
    /// --separator <string>
    pub static SEPARATOR: &str = "separator";
    /// --fillrows
    pub static FILLROWS: &str = "fillrows";
    /// input_files
    pub static INPUT_FILES: &str = "input_files";
}

impl Config {
    /// Generate column general Config
    pub fn from(args_matches: &ArgMatches) -> UResult<Self> {
        let mode = if args_matches.contains_id(options::TABLE)
            || args_matches.contains_id(options::JSON)
        {
            ColumnMode::Table
        } else if args_matches.contains_id(options::FILLROWS) {
            ColumnMode::FillRows
        } else {
            ColumnMode::FillCols
        };

        let table_columns = args_matches
            .get_one::<String>(options::TABLE_COLUMNS)
            .map(|value| {
                value
                    .split(',')
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>()
            });

        let input_files = args_matches
            .get_many::<String>(options::INPUT_FILES)
            .map(|files| files.map(|s| s.to_string()).collect::<Vec<String>>());
        let input_separator = args_matches
            .get_one::<String>(options::SEPARATOR)
            .map(|s| s.to_string());

        let output_separator = args_matches
            .get_one::<String>(options::OUTPUT_SEPARATOR)
            .map(|s| s.to_string());

        let table_order = args_matches
            .get_one::<String>(options::TABLE_ORDER)
            .map(|s| s.split(',').map(|s| s.to_string()).collect::<Vec<String>>());

        let table_columns_limit = match args_matches.get_one::<String>(options::TABLE_COLUMNS_LIMIT)
        {
            Some(s) => match s.parse::<usize>() {
                Ok(t) => {
                    if t == 0 {
                        return Err(USimpleError::new(
                            EXIT_FAILURE,
                            format!("columns limit must be greater than zero"),
                        ));
                    }
                    Some(t)
                }
                Err(_) => {
                    return Err(USimpleError::new(
                        EXIT_FAILURE,
                        format!("invalid columns limit argument: '{}'", s),
                    ))
                }
            },
            None => None,
        };

        let table_noheadings = args_matches.contains_id(options::TABLE_NOHEADINGS);

        let table_hide = match parse_columns(
            args_matches.get_one::<String>(options::TABLE_HIDE),
            &table_columns,
        ) {
            Ok(columns) => columns,
            Err(e) => return Err(e),
        };

        let table_right = match parse_columns(
            args_matches.get_one::<String>(options::TABLE_RIGHT),
            &table_columns,
        ) {
            Ok(columns) => columns,
            Err(e) => return Err(e),
        };

        let keep_empty_lines = args_matches.contains_id(options::KEEP_EMPTY_LINES);

        let termwidth = match args_matches.get_one::<String>(options::OUTPUT_WIDTH) {
            Some(s) => match s.parse::<usize>() {
                Ok(t) => Some(t),
                Err(_) => {
                    return Err(USimpleError::new(
                        EXIT_FAILURE,
                        format!("invalid columns argument: '{}'", s),
                    ))
                }
            },
            None => None,
        };

        let tree = args_matches
            .get_one::<String>(options::TREE)
            .map(|s| s.to_string());

        let tree_id = args_matches
            .get_one::<String>(options::TREE_ID)
            .map(|s| s.to_string());

        let tree_parent = args_matches
            .get_one::<String>(options::TREE_PARENT)
            .map(|s| s.to_string());

        let table_name = args_matches
            .get_one::<String>(options::TABLE_NAME)
            .map(|s| s.to_string());

        let json = args_matches.contains_id(options::JSON);

        let table_truncate = match parse_columns(
            args_matches.get_one::<String>(options::TABLE_TRUNCATE),
            &table_columns,
        ) {
            Ok(columns) => columns,
            Err(e) => return Err(e),
        };

        let table_wrap = match parse_columns(
            args_matches.get_one::<String>(options::TABLE_WRAP),
            &table_columns,
        ) {
            Ok(columns) => columns,
            Err(e) => return Err(e),
        };

        let table_noextreme = match parse_columns(
            args_matches.get_one::<String>(options::TABLE_NOEXTREME),
            &table_columns,
        ) {
            Ok(columns) => columns,
            Err(e) => return Err(e),
        };

        let table_header_repeat = args_matches.contains_id(options::TABLE_HEADER_REPEAT);

        Ok(Self {
            mode,
            table_columns,
            input_files,
            termwidth,
            ents: Vec::new(),
            input_separator,
            output_separator,
            table_order,
            table_columns_limit,
            table_noheadings,
            table_hide,
            table_right,
            keep_empty_lines,
            maxlength: 0,
            tree,
            tree_id,
            tree_parent,
            table_name,
            json,
            table_truncate,
            table_wrap,
            table_noextreme,
            table_header_repeat,
        })
    }

    /// Width of a string
    pub fn width(&self, s: &str) -> usize {
        s.chars().count()
    }
}

impl TableRow {
    /// Create a new TableRow.
    pub fn new(columns: HashMap<String, String>) -> Self {
        Self { columns }
    }
}

/// Parse command line arguments.
pub fn parse_column_cmd_args(args: impl uucore::Args, about: &str, usage: &str) -> UResult<Config> {
    let command = column_app(about, usage);
    let arg_list = args.collect_lossy();
    // Config::from(&command.try_get_matches_from(arg_list)?)
    command
        .try_get_matches_from(arg_list)
        .map_err(Into::into)
        .and_then(|matches| Config::from(&matches))
}

/// Create command.
pub fn column_app<'a>(about: &'a str, usage: &'a str) -> Command<'a> {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about(about)
        .override_usage(format_usage(usage))
        .arg(
            Arg::new(options::TABLE)
                .short('t')
                .long(options::TABLE)
                .help("create a table")
                .takes_value(false)
                .required(false)
                .display_order(10),
        )
        .arg(
            Arg::with_name(options::TABLE_NAME)
                .short('n')
                .long(options::TABLE_NAME)
                .help("table name for JSON output")
                .value_name("name")
                .takes_value(true)
                .display_order(20),
        )
        .arg(
            Arg::with_name(options::TABLE_ORDER)
                .short('O')
                .long(options::TABLE_ORDER)
                .help("specify order of output columns")
                .value_name("columns")
                .takes_value(true)
                .display_order(30),
        )
        .arg(
            Arg::with_name(options::TABLE_COLUMNS)
                .short('N')
                .long(options::TABLE_COLUMNS)
                .value_name("names")
                .help("comma separated columns names")
                .takes_value(true)
                .display_order(40),
        )
        .arg(
            Arg::with_name(options::TABLE_COLUMNS_LIMIT)
                .short('l')
                .long(options::TABLE_COLUMNS_LIMIT)
                .help("maximal number of input columns")
                .value_name("num")
                .takes_value(true)
                .display_order(50),
        )
        .arg(
            Arg::with_name(options::TABLE_NOEXTREME)
                .short('E')
                .long(options::TABLE_NOEXTREME)
                .help("don't count long text from the columns to column width")
                .value_name("columns")
                .takes_value(true)
                .display_order(60),
        )
        .arg(
            Arg::with_name(options::TABLE_NOHEADINGS)
                .short('d')
                .long(options::TABLE_NOHEADINGS)
                .help("don't print header")
                .takes_value(false)
                .display_order(70),
        )
        .arg(
            Arg::with_name(options::TABLE_HEADER_REPEAT)
                .short('e')
                .long(options::TABLE_HEADER_REPEAT)
                .help("repeat header for each page")
                .display_order(80),
        )
        .arg(
            Arg::with_name(options::TABLE_HIDE)
                .short('H')
                .long(options::TABLE_HIDE)
                .value_name("columns")
                .allow_hyphen_values(true)
                .takes_value(true)
                .help("don't print the columns")
                .display_order(90),
        )
        .arg(
            Arg::with_name(options::TABLE_RIGHT)
                .short('R')
                .long(options::TABLE_RIGHT)
                .value_name("columns")
                .allow_hyphen_values(true)
                .takes_value(true)
                .help("right align text in these columns")
                .display_order(100),
        )
        .arg(
            Arg::with_name(options::TABLE_TRUNCATE)
                .short('T')
                .long(options::TABLE_TRUNCATE)
                .help("truncate text in the columns when necessary")
                .value_name("columns")
                .takes_value(true)
                .display_order(110),
        )
        .arg(
            Arg::with_name(options::TABLE_WRAP)
                .short('W')
                .long(options::TABLE_WRAP)
                .help("wrap text in the columns when necessary")
                .value_name("columns")
                .takes_value(true)
                .display_order(120),
        )
        .arg(
            Arg::with_name(options::KEEP_EMPTY_LINES)
                .short('L')
                .long(options::KEEP_EMPTY_LINES)
                .help("don't ignore empty lines")
                .takes_value(false)
                // deprecated alias
                .alias("table-empty-lines")
                .display_order(130),
        )
        .arg(
            Arg::with_name(options::JSON)
                .short('J')
                .long(options::JSON)
                .help("use JSON output format for table")
                .takes_value(false)
                .display_order(140),
        )
        .arg(
            Arg::with_name(options::TREE)
                .short('r')
                .long(options::TREE)
                .help("column to use tree-like output for the table")
                .value_name("column")
                .takes_value(true)
                .display_order(150),
        )
        .arg(
            Arg::with_name(options::TREE_ID)
                .short('i')
                .long(options::TREE_ID)
                .help("line ID to specify child-parent relation")
                .value_name("column")
                .takes_value(true)
                .display_order(160),
        )
        .arg(
            Arg::with_name(options::TREE_PARENT)
                .short('p')
                .long(options::TREE_PARENT)
                .help("parent to specify child-parent relation")
                .value_name("column")
                .takes_value(true)
                .display_order(170),
        )
        .arg(
            Arg::with_name(options::OUTPUT_WIDTH)
                .short('c')
                .long(options::OUTPUT_WIDTH)
                .help("width of output in number of characters")
                .value_name("width")
                .takes_value(true)
                .display_order(180),
        )
        .arg(
            Arg::with_name(options::OUTPUT_SEPARATOR)
                .short('o')
                .long(options::OUTPUT_SEPARATOR)
                .value_name("string")
                .help("columns separator for table output (default is two spaces)")
                .takes_value(true)
                .display_order(190),
        )
        .arg(
            Arg::new(options::SEPARATOR)
                .short('s')
                .long(options::SEPARATOR)
                .value_name("string")
                .help("possible table delimiters")
                .takes_value(true)
                .required(false)
                .display_order(200),
        )
        .arg(
            Arg::with_name(options::FILLROWS)
                .short('x')
                .long(options::FILLROWS)
                .help("fill rows before columns")
                .takes_value(false)
                .display_order(210),
        )
        .arg(
            Arg::new(options::INPUT_FILES)
                .help("Specifies the input file")
                .multiple_values(true)
                .required(false),
        )
        .groups(&[
            ArgGroup::new("tx").args(&[options::TABLE, options::FILLROWS]),
            ArgGroup::new("Jx").args(&[options::JSON, options::FILLROWS]),
        ])
}

/// This the main of column
pub fn column_main(mut config: Config) -> UResult<()> {
    // Set terminal width
    if config.termwidth.is_none() {
        config.termwidth = Some(get_terminal_width(80 as usize));
    }

    // Set locale to get correct symbols
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    unsafe {
        setlocale(LC_ALL, b"\0".as_ptr() as *const i8)
    };
    #[cfg(target_arch = "aarch64")]
    unsafe {
        setlocale(LC_ALL, b"\0".as_ptr())
    };

    // Validation of attribute values
    validate_args(&mut config)?;

    let input_files: Option<Vec<String>> = config.input_files.clone();
    if let Some(input_files) = input_files {
        for file_path in input_files {
            // println!("file_path = {}", file_path);
            let absolute_path = match Path::new(&file_path).canonicalize() {
                Ok(p) => p,
                Err(e) => {
                    return Err(USimpleError::new(
                        EXIT_FAILURE,
                        format!("{}: {}", file_path, e),
                    ))
                }
            };
            match File::open(&absolute_path) {
                Ok(file) => {
                    if let Err(err) = read_input(file, &mut config) {
                        return Err(USimpleError::new(
                            EXIT_FAILURE,
                            format!("Error reading file {}: {}", file_path, err),
                        ));
                    }
                }
                Err(err) => {
                    return Err(USimpleError::new(
                        EXIT_FAILURE,
                        format!("Failed to open file {}: {}", file_path, err),
                    ));
                }
            }
        }
    } else {
        if let Err(err) = read_input(io::stdin(), &mut config) {
            return Err(USimpleError::new(
                EXIT_FAILURE,
                format!("Error reading from stdin: {}", err),
            ));
        }
    }

    match config.mode {
        ColumnMode::Table => {
            table_main(&mut config)?;
        }
        ColumnMode::FillCols => {
            fillcols_main(&mut config);
        }
        ColumnMode::FillRows => {
            fillrows_main(&mut config);
        }
        ColumnMode::Simple => {
            simple_main(&config);
        }
    }

    Ok(())
}

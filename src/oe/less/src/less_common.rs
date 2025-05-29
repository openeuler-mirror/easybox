// This file is part of the uutils coreutils package.
//
// (c) Yuyichen2025 <vyu112@foxmail.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use clap::{crate_version, value_parser, Arg, ArgAction, Command};
use crossterm::event::{DisableMouseCapture, KeyEventKind, MouseEvent, MouseEventKind};
use crossterm::{
    cursor::{MoveTo, MoveUp},
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, queue,
    style::Attribute,
    terminal::{self, Clear, ClearType},
};
use regex::Regex;
use std::env;
use std::fs::File;
use std::io::{stdout, Stdout, Write};
use std::time::Duration;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;
use uucore::error::{UResult, USimpleError};
use uucore::{format_usage, help_section, help_usage};

const ABOUT: &str = help_section!("about", "less.md");
const USAGE: &str = help_usage!("less.md");
const BELL: &str = "\x07";
/// Multi file top prompt
const TOP_PROMPT: &str = "\rMultiple file mode\n::::::::::::::::::\n\r{}\n\r::::::::::::::::::\n";

/// options.
pub mod configs {
    /// silent
    pub const SILENT: &str = "silent";
    /// logical
    pub const LOGICAL: &str = "logical";
    /// no pause
    pub const NO_PAUSE: &str = "no-pause";
    /// print over
    pub const PRINT_OVER: &str = "print-over";
    /// clean print
    pub const CLEAN_PRINT: &str = "clean-print";
    /// squeeze
    pub const SQUEEZE: &str = "squeeze";
    /// lines
    pub const LINES: &str = "lines";
    /// number
    pub const NUMBER: &str = "number";
    /// pattern
    pub const PATTERN: &str = "pattern";
    /// from line
    pub const FROM_LINE: &str = "from-line";
    /// files
    pub const FILES: &str = "files";
    /// show percentage
    pub const SHOW_PERCENTAGE: &str = "show_percentage";
    /// exit at eof
    pub const EXIT_AT_EOF: &str = "exit_at_eof";
    /// buffer size
    pub const BUFFER_SIZE: &str = "buffer-size";
    /// force open
    pub const FORCE_OPEN: &str = "force";
    /// show line numbers
    pub const SHOW_LINE_NUMBERS: &str = "show-line-numbers";
    /// output
    pub const OUTPUT: &str = "log-file";
    /// none interactive mode
    pub const NON_INTERACTIVE: &str = "non-interactive";
}

/// Config
pub struct Configs {
    ///
    pub clean_print: bool,
    ///
    pub from_line: usize,
    ///
    pub lines: Option<u16>,
    ///
    pub pattern: Option<String>,
    ///
    pub print_over: bool,
    ///
    pub silent: bool,
    ///
    pub squeeze: bool,
    ///
    pub show_percentage: bool,
    ///
    pub exit_at_eof: bool,
    ///
    pub buffer_size: Option<usize>,
    ///
    pub force_open: bool,
    ///
    pub show_line_numbers: bool,
    ///
    pub output: Option<String>,
    ///
    pub test_config: TestConfig,
    ///
    pub non_interactive: bool,
}

/// Test Config
pub struct TestConfig {
    ///
    pub lines: bool,
    ///
    pub buffer: bool,
    ///
    pub force_open: bool,
}

/// Configs from clap matches
impl Configs {
    /// Configs from clap matches
    pub fn from(matches: &clap::ArgMatches) -> Self {
        let lines = matches
            .get_one::<u16>(configs::LINES)
            .or_else(|| matches.get_one::<u16>(configs::NUMBER))
            .map(|n| n + 1)
            .filter(|&n| n > 0);

        let from_line = matches
            .get_one::<usize>(configs::FROM_LINE)
            .and_then(|n| if *n > 1 { Some(n - 1) } else { None })
            .unwrap_or(0);

        Configs {
            clean_print: matches.get_flag(configs::CLEAN_PRINT),
            from_line,
            lines,
            pattern: matches.get_one::<String>(configs::PATTERN).cloned(),
            print_over: matches.get_flag(configs::PRINT_OVER),
            silent: matches.get_flag(configs::SILENT),
            squeeze: matches.get_flag(configs::SQUEEZE),
            show_percentage: matches.get_flag(configs::SHOW_PERCENTAGE),
            exit_at_eof: matches.get_flag(configs::EXIT_AT_EOF),
            buffer_size: matches.get_one::<usize>(configs::BUFFER_SIZE).copied(),
            force_open: matches.get_flag(configs::FORCE_OPEN),
            show_line_numbers: matches.get_flag(configs::SHOW_LINE_NUMBERS),
            output: matches.get_one::<String>(configs::OUTPUT).cloned(),
            test_config: TestConfig {
                lines: false,
                buffer: false,
                force_open: false,
            },
            non_interactive: matches.get_flag(configs::NON_INTERACTIVE),
        }
    }
}

/// Command
pub fn less_app<'a>() -> Command<'a> {
    let base_command = Command::new(uucore::util_name())
        .about(ABOUT)
        .override_usage(format_usage(USAGE))
        .version(crate_version!())
        .infer_long_args(true);

    let args = [
        Arg::new(configs::BUFFER_SIZE)
            .short('b')
            .long(configs::BUFFER_SIZE)
            .value_name("size")
            .value_parser(value_parser!(usize))
            .help("set the Buffer size"),
        Arg::new(configs::CLEAN_PRINT)
            .short('c')
            .long(configs::CLEAN_PRINT)
            .help("Do not scroll, clean screen and display text")
            .action(ArgAction::SetTrue),
        Arg::new(configs::EXIT_AT_EOF)
            .short('e')
            .long(configs::EXIT_AT_EOF)
            .help("Exit automatically at the end of the file")
            .action(ArgAction::SetTrue),
        Arg::new(configs::FORCE_OPEN)
            .short('f')
            .long(configs::FORCE_OPEN)
            .help("Force open special files such as device files, directories, and binary files")
            .action(ArgAction::SetTrue),
        Arg::new(configs::FROM_LINE)
            .short('F')
            .long(configs::FROM_LINE)
            .value_name("number")
            .value_parser(value_parser!(usize))
            .help("Display file beginning from line number"),
        Arg::new(configs::SHOW_PERCENTAGE)
            .short('m')
            .long(configs::SHOW_PERCENTAGE)
            .help("Percentage of files to be displayed")
            .action(ArgAction::SetTrue),
        Arg::new(configs::LINES)
            .short('n')
            .long(configs::LINES)
            .value_name("number")
            .value_parser(value_parser!(u16).range(0..))
            .help("The number of lines per screen full"),
        Arg::new(configs::SHOW_LINE_NUMBERS)
            .short('N')
            .long(configs::SHOW_LINE_NUMBERS)
            .help("Show line numbers")
            .action(ArgAction::SetTrue),
        Arg::new(configs::NUMBER)
            .long(configs::NUMBER)
            .value_name("number")
            .value_parser(value_parser!(u16).range(0..))
            .help("Same as --lines"),
        Arg::new(configs::OUTPUT)
            .short('o')
            .long(configs::OUTPUT)
            .value_name("filename")
            .help("Save the output of less to the specified file")
            .required(false),
        Arg::new(configs::PRINT_OVER)
            .short('p')
            .long(configs::PRINT_OVER)
            .help("Do not scroll, display text and clean line ends")
            .action(ArgAction::SetTrue),
        Arg::new(configs::PATTERN)
            .short('P')
            .long(configs::PATTERN)
            .allow_hyphen_values(true)
            .required(false)
            .value_name("pattern")
            .help("Display file beginning from pattern match"),
        Arg::new(configs::SILENT)
            .short('q')
            .long(configs::SILENT)
            .help("Quiet the terminal bell")
            .action(ArgAction::SetTrue),
        Arg::new(configs::SQUEEZE)
            .short('s')
            .long(configs::SQUEEZE)
            .help("Squeeze multiple blank lines into one")
            .action(ArgAction::SetTrue),
        Arg::new(configs::FILES)
            .required(false)
            .action(ArgAction::Append)
            .takes_value(true)
            .multiple_values(true)
            .help("Path to the files to be read.")
            .value_hint(clap::ValueHint::FilePath),
        Arg::new(configs::NON_INTERACTIVE)
            .long(configs::NON_INTERACTIVE)
            .help("Non-interactive mode, do not wait for user input")
            .action(ArgAction::SetTrue),
    ];

    args.iter().fold(base_command, |cmd, arg| cmd.arg(arg))
}

/// setup terminal
pub fn setup_term() -> Stdout {
    let stdout = stdout();
    terminal::enable_raw_mode().unwrap();
    stdout
}

/// reset terminal
pub fn reset_term(stdout: &mut std::io::Stdout) {
    terminal::disable_raw_mode().unwrap();
    queue!(stdout, terminal::Clear(ClearType::CurrentLine)).unwrap();
    print!("\r");
    stdout.flush().unwrap();
}

///
pub fn less(
    buff: &str,
    stdout: &mut Stdout,
    multiple_file: bool,
    file: Option<&str>,
    next_file: Option<&str>,
    configs: &mut Configs,
) -> UResult<String> {
    if configs.non_interactive {
        let mut output = String::new();
        let mut lines: Vec<&str> = buff.lines().collect();

        if configs.squeeze {
            let mut new_lines = Vec::new();
            let mut last_blank = false;

            for line in &lines {
                let is_blank = line.trim().is_empty();
                if is_blank && last_blank {
                    continue;
                }
                new_lines.push(*line);
                last_blank = is_blank;
            }
            lines = new_lines;
        }

        let start_line = configs.from_line;
        if start_line < lines.len() {
            lines = lines[start_line..].to_vec();
        } else {
            lines.clear();
        }

        if let Some(pattern) = &configs.pattern {
            let regex = Regex::new(pattern)
                .unwrap_or_else(|_| Regex::new(&regex::escape(pattern)).unwrap());

            lines = lines
                .into_iter()
                .filter(|line| regex.is_match(line))
                .collect();
        }

        if configs.show_line_numbers {
            for (i, line) in lines.iter().enumerate() {
                output.push_str(&format!("{:6}  {}\n", i + 1 + start_line, line));
            }
        } else {
            for line in &lines {
                output.push_str(line);
                output.push('\n');
            }
        }

        if let Some(output_file) = &configs.output {
            match File::create(output_file) {
                Ok(mut file) => {
                    file.write_all(output.as_bytes()).map_err(|e| {
                        USimpleError::new(0, format!("Failed to write to file: {}", e))
                    })?;
                }
                Err(e) => {
                    return Err(USimpleError::new(0, format!("Failed to open file: {}", e)).into());
                }
            }
        } else {
            print!("{}", output);
        }
        return Ok(output);
    }

    let (cols, mut rows) = terminal::size()?;
    if let Some(number) = configs.lines {
        rows = number;
        configs.test_config.lines = true;
    }
    let mut output = String::new();
    let lines = break_buff(buff, cols as usize);
    let mut numbered_lines = Vec::new();
    let mut pager = Pager::new(rows, lines.clone(), next_file, configs);

    initialize_pager(
        stdout,
        configs,
        &lines,
        file,
        multiple_file,
        &mut pager,
        &mut numbered_lines,
        &mut output,
    )?;

    if should_exit_early(&mut pager, next_file, configs) {
        return Ok(output);
    }

    run_main_loop(stdout, configs, &mut pager, &mut output)
}

fn initialize_pager<'a>(
    stdout: &mut Stdout,
    configs: &mut Configs,
    lines: &[&str],
    file: Option<&str>,
    multiple_file: bool,
    pager: &mut Pager<'a>,
    numbered_lines: &'a mut Vec<String>,
    output: &mut String,
) -> std::io::Result<()> {
    handle_pattern_search(stdout, configs, pager)?;
    handle_multiple_file(stdout, configs, file, pager, multiple_file)?;
    handle_line_numbers(configs, lines, numbered_lines, pager);
    draw_pager(stdout, configs, pager, output)?;
    set_test_flags(configs, pager);
    Ok(())
}

fn should_exit_early(pager: &mut Pager, next_file: Option<&str>, configs: &Configs) -> bool {
    pager.should_close() && next_file.is_none() && configs.exit_at_eof
}

enum EventResult {
    Continue,
    Exit,
    Process,
    PageDown,
    PageUp,
    Resize(u16, u16),
}

fn run_main_loop(
    stdout: &mut Stdout,
    configs: &Configs,
    pager: &mut Pager,
    output: &mut String,
) -> UResult<String> {
    loop {
        if cfg!(test) || env::var("CARGO_TEST").is_ok() {
            if pager.has_any_test_flag() {
                reset_term(stdout);
                execute!(std::io::stdout(), DisableMouseCapture)?;
                terminal::disable_raw_mode()?;
                return Ok(std::mem::take(output));
            }
        }

        if event::poll(Duration::from_millis(10))? {
            match handle_event(event::read()?, pager, configs)? {
                EventResult::Continue => continue,
                EventResult::Exit => {
                    reset_term(stdout);
                    execute!(std::io::stdout(), DisableMouseCapture)?;
                    terminal::disable_raw_mode()?;
                    return Ok(std::mem::take(output));
                }
                EventResult::Process => {
                    handle_print_and_draw(stdout, configs, pager, None, output)?;
                }
                EventResult::PageDown => {
                    if pager.should_close() {
                        if configs.exit_at_eof {
                            return Ok(std::mem::take(output));
                        }
                    } else {
                        pager.page_down();
                        handle_print_and_draw(stdout, configs, pager, None, output)?;
                    }
                }
                EventResult::PageUp => {
                    pager.page_up();
                    paging_add_back_message(configs, stdout)?;
                    handle_print_and_draw(stdout, configs, pager, None, output)?;
                }
                EventResult::Resize(col, row) => {
                    pager.page_resize(col, row, configs.lines);
                    handle_print_and_draw(stdout, configs, pager, None, output)?;
                }
            }
        }
    }
}

fn handle_event(
    event: Event,
    pager: &mut Pager,
    configs: &Configs,
) -> std::io::Result<EventResult> {
    match event {
        Event::Key(KeyEvent {
            kind: KeyEventKind::Release,
            ..
        }) => Ok(EventResult::Continue),

        Event::Mouse(MouseEvent {
            kind: MouseEventKind::ScrollDown,
            ..
        })
        | Event::Key(KeyEvent {
            code: KeyCode::Char('e' | 'j'),
            modifiers: KeyModifiers::NONE,
            ..
        })
        | Event::Key(KeyEvent {
            code: KeyCode::Char('e' | 'n'),
            modifiers: KeyModifiers::CONTROL,
            ..
        })
        | Event::Key(KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::NONE,
            ..
        }) => handle_scroll_down(pager, configs),

        Event::Key(KeyEvent {
            code: KeyCode::Char('y' | 'k'),
            modifiers: KeyModifiers::NONE,
            ..
        })
        | Event::Key(KeyEvent {
            code: KeyCode::Char('y' | 'k' | 'p'),
            modifiers: KeyModifiers::CONTROL,
            ..
        })
        | Event::Mouse(MouseEvent {
            kind: MouseEventKind::ScrollUp,
            ..
        }) => {
            pager.prev_line();
            Ok(EventResult::Process)
        }

        Event::Key(KeyEvent {
            code: KeyCode::Char('f'),
            modifiers: KeyModifiers::NONE,
            ..
        })
        | Event::Key(KeyEvent {
            code: KeyCode::Char('f' | 'v'),
            modifiers: KeyModifiers::CONTROL,
            ..
        })
        | Event::Key(KeyEvent {
            code: KeyCode::Down,
            modifiers: KeyModifiers::NONE,
            ..
        })
        | Event::Key(KeyEvent {
            code: KeyCode::PageDown,
            modifiers: KeyModifiers::NONE,
            ..
        })
        | Event::Key(KeyEvent {
            code: KeyCode::Char(' '),
            modifiers: KeyModifiers::NONE,
            ..
        }) => Ok(EventResult::PageDown),

        Event::Key(KeyEvent {
            code: KeyCode::Char('b'),
            modifiers: KeyModifiers::NONE,
            ..
        })
        | Event::Key(KeyEvent {
            code: KeyCode::Char('b'),
            modifiers: KeyModifiers::CONTROL,
            ..
        })
        | Event::Key(KeyEvent {
            code: KeyCode::Up,
            modifiers: KeyModifiers::NONE,
            ..
        })
        | Event::Key(KeyEvent {
            code: KeyCode::PageUp,
            modifiers: KeyModifiers::NONE,
            ..
        }) => Ok(EventResult::PageUp),

        Event::Resize(col, row) => Ok(EventResult::Resize(col, row)),

        Event::Key(KeyEvent {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            ..
        }) => Ok(EventResult::Exit),

        Event::Key(KeyEvent {
            code: KeyCode::Char(_),
            ..
        }) => {
            pager.test_flags.unknown_key = true;

            Ok(EventResult::Process)
        }

        _ => Ok(EventResult::Process),
    }
}

fn handle_scroll_down(pager: &mut Pager, configs: &Configs) -> std::io::Result<EventResult> {
    if pager.should_close() {
        if configs.exit_at_eof || pager.next_file.is_some() {
            return Ok(EventResult::Exit);
        }
    } else {
        pager.next_line();
    }
    Ok(EventResult::Process)
}

fn handle_pattern_search(
    stdout: &mut Stdout,
    configs: &Configs,
    pager: &mut Pager,
) -> std::io::Result<()> {
    if let Some(pat) = configs.pattern.as_ref() {
        pager.test_flags.pattern = true;
        match search_pattern_in_file(&pager.lines, pat) {
            Some(number) => pager.upper_mark = number,
            None => {
                execute!(stdout, Clear(ClearType::CurrentLine))?;
                stdout.write_all("\rPattern not found\n".as_bytes())?;
                pager.content_rows -= 1;
            }
        }
    }
    Ok(())
}

fn handle_multiple_file(
    stdout: &mut Stdout,
    configs: &mut Configs,
    file: Option<&str>,
    pager: &mut Pager,
    multiple_file: bool,
) -> std::io::Result<()> {
    if multiple_file {
        execute!(stdout, Clear(ClearType::CurrentLine))?;
        stdout.write_all(
            TOP_PROMPT
                .replace("{}", file.unwrap_or_default())
                .as_bytes(),
        )?;
        pager.test_flags.multiple = true;
        configs.from_line = 0;
    }
    Ok(())
}

fn handle_line_numbers<'a>(
    configs: &Configs,
    lines: &[&str],
    numbered_lines: &'a mut Vec<String>,
    pager: &mut Pager<'a>,
) {
    if configs.show_line_numbers {
        for (i, line) in lines.iter().enumerate() {
            let numbered_line = format!("\x1b[1m{:5}\x1b[0m {}", i + 1, line);
            numbered_lines.push(numbered_line);
        }
        let numbered_line_refs: Vec<&str> = numbered_lines.iter().map(|s| s.as_str()).collect();
        pager.bind_lines_numbered(numbered_line_refs);
        pager.test_flags.line_numbers = true;
    }
}

fn draw_pager(
    stdout: &mut Stdout,
    configs: &Configs,
    pager: &mut Pager,
    output: &mut String,
) -> std::io::Result<()> {
    if configs.output.is_some() {
        pager.draw(stdout, None, &mut Some(output));
    } else {
        pager.draw(stdout, None, &mut None);
    }
    Ok(())
}

fn handle_print_and_draw(
    stdout: &mut Stdout,
    configs: &Configs,
    pager: &mut Pager,
    wrong_key: Option<char>,
    output: &mut String,
) -> std::io::Result<()> {
    if configs.print_over {
        execute!(
            std::io::stdout(),
            MoveTo(0, 0),
            Clear(ClearType::FromCursorDown)
        )?;
    } else if configs.clean_print {
        execute!(std::io::stdout(), Clear(ClearType::All), MoveTo(0, 0))?;
    }
    if configs.output.is_some() {
        pager.draw(stdout, wrong_key, &mut Some(output));
    } else {
        pager.draw(stdout, wrong_key, &mut None);
    }
    Ok(())
}

fn set_test_flags(configs: &Configs, pager: &mut Pager) {
    pager.test_flags.from_line = pager.upper_mark != 0;
    if configs.clean_print {
        pager.test_flags.clean_print = true;
    } else if configs.print_over {
        pager.test_flags.print_over = true;
    } else if configs.silent {
        pager.test_flags.silent = true;
    }
}

/// Pager struct
struct Pager<'a> {
    upper_mark: usize,
    content_rows: usize,
    lines: Vec<&'a str>,
    next_file: Option<&'a str>,
    line_count: usize,
    silent: bool,
    squeeze: bool,
    line_squeezed: usize,
    show_percentage: bool,

    test_flags: TestFlags,
}

/// Test flags
struct TestFlags {
    percentage: bool,
    line_numbers: bool,
    output: bool,
    lines: bool,
    from_line: bool,
    pattern: bool,
    multiple: bool,
    buffer: bool,
    clean_print: bool,
    print_over: bool,
    silent: bool,
    squeeze: bool,
    force_open: bool,
    unknown_key: bool,
}

/// Pager impl
impl<'a> Pager<'a> {
    /// construct a new pager
    fn new(rows: u16, lines: Vec<&'a str>, next_file: Option<&'a str>, options: &Configs) -> Self {
        let line_count = lines.len();
        let content_rows = rows.saturating_sub(1) as usize;

        Self {
            upper_mark: options.from_line,
            content_rows,
            lines,
            next_file,
            line_count,
            silent: options.silent,
            squeeze: options.squeeze,
            line_squeezed: 0,
            show_percentage: options.show_percentage,
            test_flags: TestFlags {
                percentage: false,
                line_numbers: false,
                output: false,
                lines: options.test_config.lines,
                from_line: false,
                pattern: false,
                multiple: false,
                buffer: options.test_config.buffer,
                clean_print: false,
                print_over: false,
                silent: false,
                squeeze: false,
                force_open: options.test_config.force_open,
                unknown_key: false,
            },
        }
    }
    /// check if close
    fn should_close(&mut self) -> bool {
        self.upper_mark
            .saturating_add(self.content_rows)
            .ge(&self.line_count)
    }

    /// page down
    fn page_down(&mut self) {
        let new_upper_mark = self.upper_mark.saturating_add(self.content_rows * 2);
        if new_upper_mark >= self.line_count {
            self.upper_mark = self.line_count.saturating_sub(self.content_rows);
            return;
        }
        self.upper_mark = new_upper_mark;
    }

    /// page up
    fn page_up(&mut self) {
        self.upper_mark = self
            .upper_mark
            .saturating_sub(self.content_rows.saturating_add(self.line_squeezed));

        if self.squeeze {
            for line in self.lines.iter().take(self.upper_mark).rev() {
                if line.is_empty() {
                    self.upper_mark = self.upper_mark.saturating_sub(1);
                } else {
                    break;
                }
            }
        }
    }

    /// next line
    fn next_line(&mut self) {
        self.upper_mark = self.upper_mark.saturating_add(1);
    }

    /// prev line
    fn prev_line(&mut self) {
        self.upper_mark = self.upper_mark.saturating_sub(1);
    }

    /// page resize
    fn page_resize(&mut self, _: u16, row: u16, option_line: Option<u16>) {
        if option_line.is_none() {
            self.content_rows = row.saturating_sub(1) as usize;
        };
    }

    /// bind lines numbered
    fn bind_lines_numbered(&mut self, numbers_line: Vec<&'a str>) {
        self.lines = numbers_line;
    }

    /// draw
    fn draw(
        &mut self,
        stdout: &mut Stdout,
        wrong_key: Option<char>,
        output: &mut Option<&mut String>,
    ) {
        self.draw_lines(stdout, output);
        let lower_mark = self
            .line_count
            .min(self.upper_mark.saturating_add(self.content_rows));
        self.draw_prompt(stdout, lower_mark, wrong_key, output);
        stdout.flush().unwrap();
    }

    /// draw lines
    fn draw_lines(&mut self, stdout: &mut Stdout, output: &mut Option<&mut String>) {
        execute!(stdout, Clear(ClearType::CurrentLine)).unwrap();

        self.line_squeezed = 0;
        let mut previous_line_blank = false;
        let mut displayed_lines = Vec::new();
        let mut iter = self.lines.iter().skip(self.upper_mark);

        while displayed_lines.len() < self.content_rows {
            match iter.next() {
                Some(line) => {
                    if self.squeeze {
                        self.test_flags.squeeze = true;
                        match (line.is_empty(), previous_line_blank) {
                            (true, false) => {
                                previous_line_blank = true;
                                displayed_lines.push(line);
                            }
                            (false, true) => {
                                previous_line_blank = false;
                                displayed_lines.push(line);
                            }
                            (false, false) => displayed_lines.push(line),
                            (true, true) => {
                                self.line_squeezed += 1;
                                self.upper_mark += 1;
                            }
                        }
                    } else {
                        displayed_lines.push(line);
                    }
                }
                None => {
                    self.upper_mark = self.line_count;
                    break;
                }
            }
        }

        for line in displayed_lines {
            if let Some(ref mut output) = output {
                output.push_str(line);
                output.push('\n');
            }
            stdout.write_all(format!("\r{line}\n").as_bytes()).unwrap();
        }
    }

    /// draw prompt
    fn draw_prompt(
        &mut self,
        stdout: &mut Stdout,
        lower_mark: usize,
        wrong_key: Option<char>,
        output: &mut Option<&mut String>,
    ) {
        let status_inner = if lower_mark == self.line_count {
            if self.next_file.is_some() {
                format!("Next file: {}", self.next_file.unwrap_or_default())
            } else {
                "End of file".to_string()
            }
        } else {
            let percentage = (lower_mark as f64 / self.line_count as f64 * 100.0).round() as u16;
            if self.show_percentage {
                format!("{}%", percentage)
            } else {
                ":".to_string()
            }
        };

        let status = format!("{status_inner}");
        let banner = match (self.silent, wrong_key) {
            (true, Some(key)) => {
                format!("{status} [Unknown key: '{key}'. Press 'h' for instructions.]")
            }
            (true, None) => format!("{status}[Press space to continue, 'q' to quit.]"),
            (false, Some(_)) => format!("{status}{BELL}"),
            (false, None) => status,
        };

        let prompt = if self.show_percentage {
            self.test_flags.percentage = true;
            format!("\r{}{}{}", Attribute::Reverse, banner, Attribute::Reset)
        } else {
            format!("\r{}{}{}", banner, Attribute::Reverse, Attribute::Reset)
        };

        if let Some(ref mut output) = output {
            output.push_str(&prompt);
            output.push('\n');
            self.test_flags.output = true;
        }

        write!(stdout, "{}", prompt).unwrap();
    }

    /// cargo test flag
    fn has_any_test_flag(&self) -> bool {
        macro_rules! check_test_flags {
            ($($field:ident),* $(,)?) => {
                false $(|| self.test_flags.$field)*
            }
        }

        check_test_flags!(
            percentage,
            line_numbers,
            output,
            lines,
            from_line,
            pattern,
            multiple,
            buffer,
            clean_print,
            print_over,
            silent,
            squeeze,
            force_open
        )
    }
}

/// search pattern in file
fn search_pattern_in_file(lines: &[&str], pattern: &str) -> Option<usize> {
    if lines.is_empty() || pattern.is_empty() {
        return None;
    }
    for (line_number, line) in lines.iter().enumerate() {
        if line.contains(pattern) {
            return Some(line_number);
        }
    }
    None
}

/// add back message
fn paging_add_back_message(options: &Configs, stdout: &mut Stdout) -> UResult<()> {
    if options.lines.is_some() {
        execute!(stdout, MoveUp(1))?;
        stdout.write_all("\n\r...back 1 page\n".as_bytes())?;
    }
    Ok(())
}

// Break the lines on the cols of the terminal
fn break_buff(buff: &str, cols: usize) -> Vec<&str> {
    // We _could_ do a precise with_capacity here, but that would require scanning the
    // whole buffer. Just guess a value instead.
    let mut lines = Vec::with_capacity(2048);

    for l in buff.lines() {
        lines.append(&mut break_line(l, cols));
    }
    lines
}

/// Break a line on the cols of the terminal
fn break_line(line: &str, cols: usize) -> Vec<&str> {
    let width = UnicodeWidthStr::width(line);
    let mut lines = Vec::new();
    if width < cols {
        lines.push(line);
        return lines;
    }

    let gr_idx = UnicodeSegmentation::grapheme_indices(line, true);
    let mut last_index = 0;
    let mut total_width = 0;
    for (index, grapheme) in gr_idx {
        let width = UnicodeWidthStr::width(grapheme);
        total_width += width;

        if total_width > cols {
            lines.push(&line[last_index..index]);
            last_index = index;
            total_width = width;
        }
    }

    if last_index != line.len() {
        lines.push(&line[last_index..]);
    }
    lines
}

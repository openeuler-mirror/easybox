//! This file is part of the uutils coreutils package.
//
// (c) Lin Guantao <moyihust@gmail.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use clap::{crate_version, Arg, ArgAction, ArgGroup, Command};
use std::fs::File;
use std::io::BufRead;
use std::thread;
use std::time::Duration;
use uucore::error::{UResult, USimpleError};
use uucore::format_usage;
use uucore::libc::EXIT_FAILURE;

/// MEMINFO
pub static MEMINFO: &str = "/proc/meminfo";

///
struct MemInfo {
    mem_total: u64,
    mem_free: u64,
    shared: u64,
    mem_available: u64,
    buffers: u64,
    cached: u64,
    swap_total: u64,
    swap_free: u64,
}

enum Unit {
    Bytes,
    Kilo,
    Mega,
    Giga,
    Tera,
    Peta,
    Kibi,
    Mebi,
    Gibi,
    Tebi,
    Pebi,
}

struct HumanUnit {
    unit: String,
    num: f64,
}

/// Config.
pub struct Config {
    ///
    pub bytes: bool,
    ///
    pub kilo: bool,
    ///
    pub mega: bool,
    ///
    pub giga: bool,
    ///
    pub tera: bool,
    ///
    pub peta: bool,
    ///
    pub kibi: bool,
    ///
    pub mebi: bool,
    ///
    pub gibi: bool,
    ///
    pub tebi: bool,
    ///
    pub pebi: bool,
    ///
    pub human: bool,
    ///
    pub si: bool,
    ///
    pub lohi: bool,
    ///
    pub total: bool,
    ///
    pub seconds: Option<f64>,
    ///
    pub count: Option<u32>,
    ///
    pub wide: bool,
}

/// options
pub mod options {
    ///
    pub static BYTES: &str = "bytes";
    ///
    pub static KILO: &str = "kilo";
    ///
    pub static MEGA: &str = "mega";
    ///
    pub static GIGA: &str = "giga";
    ///
    pub static TERA: &str = "tera";
    ///
    pub static PETA: &str = "peta";
    ///
    pub static KIBI: &str = "kibi";
    ///
    pub static MEBI: &str = "mebi";
    ///
    pub static GIBI: &str = "gibi";
    ///
    pub static TEBI: &str = "tebi";
    ///
    pub static PEBI: &str = "pebi";
    ///
    pub static HUMAN: &str = "human";
    ///
    pub static SI: &str = "si";
    ///
    pub static LOHI: &str = "lohi";
    ///
    pub static TOTAL: &str = "total";
    ///
    pub static SECONDS: &str = "seconds";
    ///
    pub static COUNT: &str = "count";
    ///
    pub static WIDE: &str = "wide";
}

impl Config {
    /// from stdin
    pub fn from(options: &clap::ArgMatches) -> UResult<Self> {
        let seconds = match options.get_one::<f64>(options::SECONDS) {
            Some(s) => {
                // s * 1000000 < 1 in C.
                if *s < 0.0 {
                    return Err(USimpleError::new(
                        EXIT_FAILURE,
                        format!("seconds argument `{}' is not positive number", s),
                    ));
                } else {
                    Some(*s)
                }
            }
            None => None,
        };

        let count = match options.get_one::<u32>(options::COUNT) {
            Some(c) => {
                if *c < 1 {
                    return Err(USimpleError::new(
                        EXIT_FAILURE,
                        format!("failed to parse count argument: '{}'", c),
                    ));
                } else {
                    Some(*c)
                }
            }
            None => None,
        };

        Ok(Self {
            bytes: options.contains_id(options::BYTES),
            kilo: options.contains_id(options::KILO),
            mega: options.contains_id(options::MEGA),
            giga: options.contains_id(options::GIGA),
            tera: options.contains_id(options::TERA),
            peta: options.contains_id(options::PETA),
            kibi: options.contains_id(options::KIBI),
            mebi: options.contains_id(options::MEBI),
            gibi: options.contains_id(options::GIBI),
            tebi: options.contains_id(options::TEBI),
            pebi: options.contains_id(options::PEBI),
            human: options.get_flag(options::HUMAN),
            si: options.get_flag(options::SI),
            lohi: options.get_flag(options::LOHI),
            total: options.get_flag(options::TOTAL),
            seconds,
            count,
            wide: options.get_flag(options::WIDE),
        })
    }
}

///
pub fn parse_free_cmd_args<'a>(
    args: impl Iterator<Item = &'a str>,
    about: &'a str,
    usage: &'a str,
) -> UResult<Config> {
    let app = free_app(about, usage);
    let matches = app.get_matches_from_safe(args)?;

    Ok(Config::from(&matches)?)
}

///
pub fn free_app<'a>(about: &'a str, usage: &'a str) -> Command<'a> {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about(about)
        .override_usage(format_usage(usage))
        .infer_long_args(true)
        .group(ArgGroup::new("unit").args(&[
            "bytes", "kilo", "mega", "giga", "tera", "peta", "kibi", "mebi", "gibi", "tebi", "pebi",
        ]))
        .arg(
            Arg::new(options::BYTES)
                .short('b')
                .long(options::BYTES)
                .help("show output in bytes")
                .display_order(10),
        )
        .arg(
            Arg::new(options::KILO)
                .long(options::KILO)
                .help("show output in kilobytes")
                .display_order(20),
        )
        .arg(
            Arg::new(options::MEGA)
                .long(options::MEGA)
                .help("show output in megabytes")
                .display_order(30),
        )
        .arg(
            Arg::new(options::GIGA)
                .long(options::GIGA)
                .help("show output in gigabytes")
                .display_order(40),
        )
        .arg(
            Arg::new(options::TERA)
                .long(options::TERA)
                .help("show output in terabytes")
                .display_order(50),
        )
        .arg(
            Arg::new(options::PETA)
                .long(options::PETA)
                .help("show output in petabytes")
                .display_order(60),
        )
        .arg(
            Arg::new(options::KIBI)
                .short('k')
                .long(options::KIBI)
                .help("show output in kibibytes")
                .display_order(70),
        )
        .arg(
            Arg::new(options::MEBI)
                .short('m')
                .long(options::MEBI)
                .help("show output in mebibytes")
                .display_order(80),
        )
        .arg(
            Arg::new(options::GIBI)
                .short('g')
                .long(options::GIBI)
                .help("show output in gibibytes")
                .display_order(90),
        )
        .arg(
            Arg::new(options::TEBI)
                .long(options::TEBI)
                .help("show output in tebibytes")
                .display_order(100),
        )
        .arg(
            Arg::new(options::PEBI)
                .long(options::PEBI)
                .help("show output in pebibytes")
                .display_order(110),
        )
        .arg(
            Arg::new(options::HUMAN)
                .short('h')
                .long(options::HUMAN)
                .action(ArgAction::SetTrue)
                .help("show human-readable output")
                .display_order(120),
        )
        .arg(
            Arg::new(options::SI)
                .long(options::SI)
                .action(ArgAction::SetTrue)
                .help("use powers of 1000 not 1024")
                .display_order(130),
        )
        .arg(
            Arg::new(options::LOHI)
                .short('l')
                .long(options::LOHI)
                .action(ArgAction::SetTrue)
                .help("show detailed low and high memory statistics")
                .display_order(140),
        )
        .arg(
            Arg::new(options::TOTAL)
                .short('t')
                .long(options::TOTAL)
                .action(ArgAction::SetTrue)
                .help("show total for RAM + swap")
                .display_order(150),
        )
        .arg(
            Arg::new(options::SECONDS)
                .short('s')
                .long(options::SECONDS)
                .takes_value(true)
                .value_name("N")
                .action(clap::ArgAction::Set)
                .value_parser(clap::value_parser!(f64))
                .help("repeat printing every N seconds")
                .display_order(170),
        )
        .arg(
            Arg::new(options::COUNT)
                .short('c')
                .long(options::COUNT)
                .takes_value(true)
                .value_name("N")
                .action(ArgAction::Set)
                .value_parser(clap::value_parser!(u32))
                .help("repeat printing N times, then exit")
                .display_order(180),
        )
        .arg(
            Arg::new(options::WIDE)
                .short('w')
                .long(options::WIDE)
                .action(ArgAction::SetTrue)
                .help("wide output")
                .display_order(190),
        )
}

/// handle input
pub fn handle_input(config: &Config) -> UResult<()> {
    let mut repeat_count = config.count.unwrap_or(1);
    let repeat_dur = Duration::from_nanos((config.seconds.unwrap_or(1.0) * 1_000_000_000.0) as u64);

    loop {
        print_memory(config);

        if !(config.count.is_none() && config.seconds.is_some()) {
            repeat_count -= 1;
        }

        if repeat_count < 1 {
            break;
        }

        println!("");
        thread::sleep(repeat_dur);
    }

    Ok(())
}

fn print_memory(config: &Config) {
    let unit = analyze_unit(config);
    if config.human {
        hum_print(&get_mem_info(), config.lohi, config.total, config.wide);
    } else {
        raw_print(
            &get_mem_info(),
            unit,
            config.lohi,
            config.total,
            config.wide,
        );
    }
}

/// Get the memory info.
fn get_mem_info() -> MemInfo {
    let mut mem_info = MemInfo {
        mem_total: 0,
        mem_free: 0,
        mem_available: 0,
        buffers: 0,
        cached: 0,
        swap_total: 0,
        swap_free: 0,
        shared: 0,
    };

    let mut meminfo = std::io::BufReader::new(File::open(MEMINFO).unwrap());

    let mut line = String::new();

    loop {
        line.clear();
        meminfo.read_line(&mut line).unwrap();
        if line.is_empty() {
            break;
        }
        let mut parts = line.split_whitespace();
        match parts.next() {
            Some("MemTotal:") => {
                mem_info.mem_total = parts.next().unwrap().parse().unwrap();
            }
            Some("MemFree:") => {
                mem_info.mem_free = parts.next().unwrap().parse().unwrap();
            }
            Some("MemAvailable:") => {
                mem_info.mem_available = parts.next().unwrap().parse().unwrap();
            }
            Some("Buffers:") => {
                mem_info.buffers = parts.next().unwrap().parse().unwrap();
            }
            Some("Cached:") => {
                mem_info.cached = parts.next().unwrap().parse().unwrap();
            }
            Some("SwapTotal:") => {
                mem_info.swap_total = parts.next().unwrap().parse().unwrap();
            }
            Some("SwapFree:") => {
                mem_info.swap_free = parts.next().unwrap().parse().unwrap();
            }
            Some("Shmem:") => {
                mem_info.shared = parts.next().unwrap().parse().unwrap();
            }
            _ => {}
        }
    }

    mem_info
}

fn raw_print(meminfo: &MemInfo, unit: Unit, lohi: bool, t: bool, w: bool) {
    let total = convert_unit(meminfo.mem_total, &unit);
    let used = convert_unit(meminfo.mem_total - meminfo.mem_free, &unit);
    let free = convert_unit(meminfo.mem_free, &unit);
    let shared = convert_unit(meminfo.shared, &unit);
    let buff = convert_unit(meminfo.buffers, &unit);
    let cache = convert_unit(meminfo.cached, &unit);
    let available = convert_unit(meminfo.mem_available, &unit);
    let swap_total = convert_unit(meminfo.swap_total, &unit);
    let swap_used = convert_unit(meminfo.swap_total - meminfo.swap_free, &unit);
    let swap_free = convert_unit(meminfo.swap_free, &unit);

    if w {
        println!("               total        used        free      shared     buffers       cache   available");
        println!(
            "Mem:{:>16}{:>12}{:>12}{:>12}{:>12}{:>12}{:>12}",
            total, used, free, shared, buff, cache, available,
        );
    } else {
        println!(
            "               total        used        free      shared  buff/cache   available"
        );
        println!(
            "Mem:{:>16}{:>12}{:>12}{:>12}{:>12}{:>12}",
            total,
            used - buff - cache,
            free,
            shared,
            buff + cache,
            available
        );
    }
    if lohi {
        println!("Low:{:>16}{:>12}{:>12}", total, total - free, free);
        println!("High:{:>15}{:>12}{:>12}", 0, 0, 0);
    }
    println!("Swap:{:>15}{:>12}{:>12}", swap_total, swap_used, swap_free);
    if t {
        println!(
            "Total:{:>14}{:>12}{:>12}",
            total + swap_total,
            used + swap_used,
            free + swap_free
        );
    }
}

fn convert_unit(value: u64, unit: &Unit) -> u64 {
    match unit {
        Unit::Bytes => value * 1024,
        Unit::Kilo => value * 1024 / 1000,
        Unit::Mega => value * 1024 / 1000000,
        Unit::Giga => value * 1024 / 1000000000,
        Unit::Tera => value * 1024 / 1000000000000,
        Unit::Peta => value * 1024 / 1000000000000000,
        Unit::Kibi => value,
        Unit::Mebi => value / 1024,
        Unit::Gibi => value / (1024 * 1024),
        Unit::Tebi => value / (1024 * 1024 * 1024),
        Unit::Pebi => value / (1024 * 1024 * 1024 * 1024),
    }
}

fn analyze_unit(config: &Config) -> Unit {
    match config {
        _ if config.bytes => Unit::Bytes,
        _ if config.kilo || (config.si && config.kibi) => Unit::Kilo,
        _ if config.mega || (config.si && config.mebi) => Unit::Mega,
        _ if config.giga || (config.si && config.gibi) => Unit::Giga,
        _ if config.tera || (config.si && config.tebi) => Unit::Tera,
        _ if config.peta || (config.si && config.pebi) => Unit::Peta,
        _ if config.kibi => Unit::Kibi,
        _ if config.mebi => Unit::Mebi,
        _ if config.gibi => Unit::Gibi,
        _ if config.tebi => Unit::Tebi,
        _ if config.pebi => Unit::Pebi,
        _ if config.si => Unit::Kilo,
        _ => Unit::Kibi,
    }
}

fn get_suit_unit(num: u64) -> HumanUnit {
    let num: f64 = num as f64;
    let (unit, value) = if num >= 1024.0 * 1024.0 * 1024.0 * 1024.0 {
        ("Pi", num / (1024.0 * 1024.0 * 1024.0 * 1024.0))
    } else if num >= 1024.0 * 1024.0 * 1024.0 {
        ("Ti", num / (1024.0 * 1024.0 * 1024.0))
    } else if num >= 1024.0 * 1024.0 {
        ("Gi", num / (1024.0 * 1024.0))
    } else if num >= 1024.0 {
        ("Mi", num / 1024.0)
    } else if num > 0.0 {
        ("Ki", num)
    } else {
        ("B", 0.0)
    };

    HumanUnit {
        unit: unit.to_string(),
        num: value,
    }
}

fn hum_print(meminfo: &MemInfo, lohi: bool, t: bool, w: bool) {
    let total = get_suit_unit(meminfo.mem_total);
    let used = get_suit_unit(meminfo.mem_total - meminfo.mem_free);
    let free = get_suit_unit(meminfo.mem_free);
    let shared = get_suit_unit(meminfo.shared);
    let buff_cache = get_suit_unit(meminfo.buffers + meminfo.cached);
    let buff = get_suit_unit(meminfo.buffers);
    let cache = get_suit_unit(meminfo.cached);
    let available = get_suit_unit(meminfo.mem_available);
    let swap_total = get_suit_unit(meminfo.swap_total);
    let swap_used = get_suit_unit(meminfo.swap_total - meminfo.swap_free);
    let swap_free = get_suit_unit(meminfo.swap_free);
    if w {
        println!("               total        used        free      shared     buffers       cache   available");
        println!(
            "Mem:       {:>8.1}{} {:>8.1}{}  {:>8.1}{}   {:>8.1}{}  {:>8.1}{} {:>8.1}{} {:>8.1}{}",
            total.num,
            total.unit,
            used.num,
            used.unit,
            free.num,
            free.unit,
            shared.num,
            shared.unit,
            buff.num,
            buff.unit,
            cache.num,
            cache.unit,
            available.num,
            available.unit,
        );
    } else {
        println!(
            "               total        used        free      shared  buff/cache   available"
        );
        println!(
            "Mem:       {:>8.1}{} {:>8.1}{}  {:>8.1}{}   {:>8.1}{}  {:>8.1}{} {:>8.1}{}",
            total.num,
            total.unit,
            get_suit_unit(meminfo.mem_total - meminfo.mem_free - meminfo.buffers - meminfo.cached)
                .num,
            get_suit_unit(meminfo.mem_total - meminfo.mem_free - meminfo.buffers - meminfo.cached)
                .unit,
            free.num,
            free.unit,
            shared.num,
            shared.unit,
            buff_cache.num,
            buff_cache.unit,
            available.num,
            available.unit,
        );
    }
    if lohi {
        println!(
            "Low:       {:>8.1}{} {:>8.1}{}   {:>8.1}{}",
            total.num, total.unit, used.num, used.unit, free.num, free.unit
        );
        println!(
            "High:      {:>8.1}{} {:>8.1}{}   {:>8.1}{}",
            0, "B", 0, "B", 0, "B"
        );
    }
    println!(
        "Swap:      {:>8.1}{} {:>8.1}{}   {:>8.1}{}",
        swap_total.num,
        swap_total.unit,
        swap_used.num,
        swap_used.unit,
        swap_free.num,
        swap_free.unit
    );
    if t {
        println!(
            "Total:     {:>8.1}{} {:>8.1}{} {:>8.1}{}",
            get_suit_unit(meminfo.mem_total + meminfo.swap_total).num,
            get_suit_unit(meminfo.mem_total + meminfo.swap_total).unit,
            get_suit_unit(
                meminfo.mem_total + meminfo.swap_total - meminfo.mem_free - meminfo.swap_free
            )
            .num,
            get_suit_unit(
                meminfo.mem_total + meminfo.swap_total - meminfo.mem_free - meminfo.swap_free
            )
            .unit,
            get_suit_unit(meminfo.mem_free + meminfo.swap_free).num,
            get_suit_unit(meminfo.mem_free + meminfo.swap_free).unit
        );
    }
}

//! This file is part of the uutils coreutils package.
//
// (c) Lin Guantao <moyihust@gmail.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use std::io::BufRead;
use uucore::error::UResult;
use uucore::format_usage;

use std::fs::File;

use clap::{crate_version, Arg, Command};

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
    pub seconds: Option<u32>,
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
    pub fn from(options: &clap::ArgMatches) -> Self {
        Self {
            bytes: options.is_present(options::BYTES),
            kilo: options.is_present(options::KILO),
            mega: options.is_present(options::MEGA),
            giga: options.is_present(options::GIGA),
            tera: options.is_present(options::TERA),
            peta: options.is_present(options::PETA),
            kibi: options.is_present(options::KIBI),
            mebi: options.is_present(options::MEBI),
            gibi: options.is_present(options::GIBI),
            tebi: options.is_present(options::TEBI),
            pebi: options.is_present(options::PEBI),
            human: options.is_present(options::HUMAN),
            si: options.is_present(options::SI),
            lohi: options.is_present(options::LOHI),
            total: options.is_present(options::TOTAL),
            seconds: options
                .value_of(options::SECONDS)
                .map(|v| v.parse().unwrap()),
            count: options.value_of(options::COUNT).map(|v| v.parse().unwrap()),
            wide: options.is_present(options::WIDE),
        }
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

    Ok(Config::from(&matches))
}

///
pub fn free_app<'a>(about: &'a str, usage: &'a str) -> Command<'a> {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about(about)
        .override_usage(format_usage(usage))
        .infer_long_args(true)
        .arg(
            Arg::new("bytes")
                .short('b')
                .long("bytes")
                .help("show output in bytes")
                .conflicts_with_all(&[
                    "kilo", "mega", "giga", "tera", "peta", "kibi", "mebi", "gibi", "tebi", "pebi",
                ]),
        )
        .arg(
            Arg::new("kilo")
                .long("kilo")
                .help("show output in kilobytes")
                .conflicts_with_all(&[
                    "bytes", "mega", "giga", "tera", "peta", "kibi", "mebi", "gibi", "tebi", "pebi",
                ]),
        )
        .arg(
            Arg::new("mega")
                .long("mega")
                .help("show output in megabytes")
                .conflicts_with_all(&[
                    "bytes", "kilo", "giga", "tera", "peta", "kibi", "mebi", "gibi", "tebi", "pebi",
                ]),
        )
        .arg(
            Arg::new("giga")
                .long("giga")
                .help("show output in gigabytes")
                .conflicts_with_all(&[
                    "bytes", "kilo", "mega", "tera", "peta", "kibi", "mebi", "gibi", "tebi", "pebi",
                ]),
        )
        .arg(
            Arg::new("tera")
                .long("tera")
                .help("show output in terabytes")
                .conflicts_with_all(&[
                    "bytes", "kilo", "mega", "giga", "peta", "kibi", "mebi", "gibi", "tebi", "pebi",
                ]),
        )
        .arg(
            Arg::new("peta")
                .long("peta")
                .help("show output in petabytes")
                .conflicts_with_all(&[
                    "bytes", "kilo", "mega", "giga", "tera", "kibi", "mebi", "gibi", "tebi", "pebi",
                ]),
        )
        .arg(
            Arg::new("kibi")
                .short('k')
                .long("kibi")
                .help("show output in kibibytes")
                .conflicts_with_all(&[
                    "bytes", "kilo", "mega", "giga", "tera", "peta", "mebi", "gibi", "tebi", "pebi",
                ]),
        )
        .arg(
            Arg::new("mebi")
                .short('m')
                .long("mebi")
                .help("show output in mebibytes")
                .conflicts_with_all(&[
                    "bytes", "kilo", "mega", "giga", "tera", "peta", "kibi", "gibi", "tebi", "pebi",
                ]),
        )
        .arg(
            Arg::new("gibi")
                .short('g')
                .long("gibi")
                .help("show output in gibibytes")
                .conflicts_with_all(&[
                    "bytes", "kilo", "mega", "giga", "tera", "peta", "kibi", "mebi", "tebi", "pebi",
                ]),
        )
        .arg(
            Arg::new("tebi")
                .long("tebi")
                .help("show output in tebibytes")
                .conflicts_with_all(&[
                    "bytes", "kilo", "mega", "giga", "tera", "peta", "kibi", "mebi", "gibi", "pebi",
                ]),
        )
        .arg(
            Arg::new("pebi")
                .long("pebi")
                .help("show output in pebibytes")
                .conflicts_with_all(&[
                    "bytes", "kilo", "mega", "giga", "tera", "peta", "kibi", "mebi", "gibi", "tebi",
                ]),
        )
        .arg(
            Arg::new("human")
                .short('h')
                .long("human")
                .help("show human-readable output"),
        )
        .arg(
            Arg::new("si")
                .long("si")
                .help("use powers of 1000 not 1024"),
        )
        .arg(
            Arg::new("lohi")
                .short('l')
                .long("lohi")
                .help("show detailed low and high memory statistics"),
        )
        .arg(
            Arg::new("total")
                .short('t')
                .long("total")
                .help("show total for RAM + swap"),
        )
        .arg(
            Arg::new("seconds")
                .short('s')
                .long("seconds")
                .takes_value(true)
                .help("repeat printing every N seconds"),
        )
        .arg(
            Arg::new("count")
                .short('c')
                .long("count")
                .takes_value(true)
                .help("repeat printing N times, then exit"),
        )
        .arg(Arg::new("wide").short('w').long("wide").help("wide output"))
}

/// handle input
pub fn handle_input(config: &Config) -> UResult<()> {
    if config.seconds.is_some() && config.count.is_none() {
        loop {
            print_memory(config);
            std::thread::sleep(std::time::Duration::from_secs(
                config.seconds.unwrap() as u64
            ));
        }
    }
    if config.count.is_some() && config.seconds.is_none() {
        for _ in 0..config.count.unwrap() {
            print_memory(config);
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }
    if config.count.is_some() && config.seconds.is_some() {
        for _ in 0..config.count.unwrap() {
            print_memory(config);
            std::thread::sleep(std::time::Duration::from_secs(
                config.seconds.unwrap() as u64
            ));
        }
    }
    if config.count.is_none() && config.seconds.is_none() {
        print_memory(config);
    }
    Ok(())
}

fn print_memory(config: &Config) {
    let unit = anaylze_unit(config);
    if config.human {
        hum_print(&get_mem_info(), config.lohi, config.total, config.wide);
        return;
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

fn anaylze_unit(config: &Config) -> Unit {
    let mut unit: Unit = Unit::Kibi;
    if config.si {
        if config.bytes {
            unit = Unit::Bytes;
        } else if config.kilo {
            unit = Unit::Kilo;
        } else if config.mega {
            unit = Unit::Mega;
        } else if config.giga {
            unit = Unit::Giga;
        } else if config.tera {
            unit = Unit::Tera;
        } else if config.peta {
            unit = Unit::Peta;
        } else if config.kibi {
            unit = Unit::Kilo;
        } else if config.mebi {
            unit = Unit::Mega;
        } else if config.gibi {
            unit = Unit::Giga;
        } else if config.tebi {
            unit = Unit::Tera;
        } else if config.pebi {
            unit = Unit::Peta;
        }
    } else {
        if config.bytes {
            unit = Unit::Bytes;
        } else if config.kilo {
            unit = Unit::Kilo;
        } else if config.mega {
            unit = Unit::Mega;
        } else if config.giga {
            unit = Unit::Giga;
        } else if config.tera {
            unit = Unit::Tera;
        } else if config.peta {
            unit = Unit::Peta;
        } else if config.kibi {
            unit = Unit::Kibi;
        } else if config.mebi {
            unit = Unit::Mebi;
        } else if config.gibi {
            unit = Unit::Gibi;
        } else if config.tebi {
            unit = Unit::Tebi;
        } else if config.pebi {
            unit = Unit::Pebi;
        }
    }
    unit
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

//! This file is part of the easybox package.
//
// (c) Haopeng Liu <657407891@qq.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use crate::iostat_utils::*;
use chrono::Local;
use clap::{crate_version, Arg, Command};
use std::ffi::OsStr;
use std::fmt::Write;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::io::{self, BufRead};
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;
use std::{process, thread};
use uname_rs::Uname;
use uucore::{
    error::{UResult, USimpleError},
    format_usage,
};

const IOSTAT_CMD_PARSE_ERROR: i32 = 1;

#[derive(Default, Debug, Clone)]
struct StatsCpu {
    cpu_user: u64,
    cpu_nice: u64,
    cpu_sys: u64,
    cpu_idle: u64,
    cpu_iowait: u64,
    cpu_steal: u64,
    cpu_hardirq: u64,
    cpu_softirq: u64,
    cpu_guest: u64,
    cpu_guest_nice: u64,
}

#[derive(Default, Clone, Debug, Copy)]
struct IoStats {
    rd_sectors: u64,
    wr_sectors: u64,
    dc_sectors: u64,
    rd_ios: u64,
    rd_merges: u64,
    wr_ios: u64,
    wr_merges: u64,
    dc_ios: u64,
    dc_merges: u64,
    fl_ios: u64,
    rd_ticks: u32,
    wr_ticks: u32,
    dc_ticks: u32,
    fl_ticks: u32,
    ios_pgr: u32,
    tot_ticks: u32,
    rq_ticks: u32,
}

impl IoStats {
    fn combine(&mut self, other: IoStats) {
        self.rd_sectors += other.rd_sectors;
        self.wr_sectors += other.wr_sectors;
        self.dc_sectors += other.dc_sectors;
        self.rd_ios += other.rd_ios;
        self.rd_merges += other.rd_merges;
        self.wr_ios += other.wr_ios;
        self.wr_merges += other.wr_merges;
        self.dc_ios += other.dc_ios;
        self.dc_merges += other.dc_merges;
        self.fl_ios += other.fl_ios;
        self.rd_ticks += other.rd_ticks;
        self.wr_ticks += other.wr_ticks;
        self.dc_ticks += other.dc_ticks;
        self.fl_ticks += other.fl_ticks;
        self.ios_pgr += other.ios_pgr;
        self.tot_ticks += other.tot_ticks;
        self.rq_ticks += other.rq_ticks;
    }
}
#[derive(Debug, Default)]
struct ExtIoStats {
    r_await: f64,
    w_await: f64,
    d_await: f64,
    f_await: f64,
    rsectors: f64,
    wsectors: f64,
    dsectors: f64,
    sectors: f64,
    rrqm_pc: f64,
    wrqm_pc: f64,
    drqm_pc: f64,
    rarqsz: f64,
    warqsz: f64,
    darqsz: f64,
}
///
struct StatsDisk {
    nr_ios: u64,
    rd_sect: u64,
    wr_sect: u64,
    dc_sect: u64,
    rd_ticks: u32,
    wr_ticks: u32,
    tot_ticks: u32,
    dc_ticks: u32,
}

/// Structure used for extended disk statistics
#[derive(Default)]
struct ExtDiskStats {
    util: f64,
    await1: f64,
    arqsz: f64,
}

impl ExtDiskStats {
    fn compute_ext_disk_stats(&mut self, sdc: &StatsDisk, sdp: &StatsDisk, itv: u64) {
        // Compute utilization
        self.util = if sdc.tot_ticks < sdp.tot_ticks {
            0.0
        } else {
            s_value(sdp.tot_ticks.into(), sdc.tot_ticks.into(), itv)
        };
        // Compute await (average wait time)
        self.await1 = if sdc.nr_ios != sdp.nr_ios {
            ((sdc.rd_ticks - sdp.rd_ticks) as f64
                + (sdc.wr_ticks - sdp.wr_ticks) as f64
                + (sdc.dc_ticks - sdp.dc_ticks) as f64)
                / (sdc.nr_ios - sdp.nr_ios) as f64
        } else {
            0.0
        };

        // Compute arqsz (average request size)
        self.arqsz = if sdc.nr_ios != sdp.nr_ios {
            ((sdc.rd_sect - sdp.rd_sect) as f64
                + (sdc.wr_sect - sdp.wr_sect) as f64
                + (sdc.dc_sect - sdp.dc_sect) as f64)
                / (sdc.nr_ios - sdp.nr_ios) as f64
        } else {
            0.0
        };
    }
}

#[derive(Default, Debug)]
struct IoDevice {
    name: String,
    group: bool,
    major: Option<u32>,
    minor: Option<u32>,
    dev_stats: [IoStats; 2],
}

///
pub struct Config {
    ///
    pub cpu: bool,
    ///
    pub device: bool,
    ///
    pub dec: u8,
    ///
    pub directory: Option<String>,
    ///
    pub alt_directory: Option<String>,
    ///
    pub group: Option<Vec<String>>,
    ///
    pub hide: bool,
    ///
    pub human: bool,
    ///
    pub persistent: Option<String>,
    ///
    pub json: bool,
    ///
    pub kilobytes: bool,
    ///
    pub megabytes: bool,
    ///
    pub timestamp: bool,
    ///
    pub extended: bool,
    ///
    pub omit: bool,
    ///
    pub partitions: Vec<String>,
    ///
    pub pretty: bool,
    ///
    pub registered_device_mapper: bool,
    ///
    pub short: bool,
    ///
    pub zero: bool,
    ///
    pub interval: Option<u64>,
    ///
    pub count: i64,
    ///
    pub devices: Vec<String>,
}

/// options.
///
pub mod options {
    ///
    pub static CPU: &str = "cpu";
    ///
    pub static DEVICE: &str = "device";
    ///
    pub static DEC: &str = "dec";
    ///
    pub static DIRECTORY: &str = "directory";
    ///
    pub static GROUP_NAME: &str = "group_name";
    ///
    pub static HIDE: &str = "hide";
    ///
    pub static HUMAN: &str = "human";
    ///
    pub static PERSISTENT: &str = "persistent";
    ///
    pub static JSON: &str = "json";
    ///
    pub static KILOBYTES: &str = "kilobytes";
    ///
    pub static MEGABYTES: &str = "megabytes";
    ///
    pub static TIMESTAMP: &str = "timestamp";
    ///
    pub static EXTENDED: &str = "extended";
    ///
    pub static OMIT: &str = "omit";
    ///
    pub static PARTITIONS: &str = "partitions";
    ///
    pub static PRETTY: &str = "pretty";
    ///
    pub static REGISTERED_DEVICE_MAPPER: &str = "registered_device_mapper";
    ///
    pub static SHORT: &str = "short";
    ///
    pub static ZERO: &str = "zero";
    ///
    pub static HP: &str = "h";
    ///
    pub static RESERVED: &str = "reserved";
}

impl Config {
    ///
    pub fn from(options: &clap::ArgMatches) -> UResult<Self> {
        let mut cpu = options.is_present(options::CPU);
        let mut device = options.is_present(options::DEVICE);
        if !cpu && !device {
            cpu = true;
            device = true;
        }
        let dec = options.value_of_t::<u8>(options::DEC).unwrap_or(2);
        if dec > 2 {
            return Err(USimpleError::new(
                IOSTAT_CMD_PARSE_ERROR,
                format!("dec must in {{0, 1, 2}}\n"),
            ));
        }
        let other_args: Vec<String> = options
            .get_many::<String>(options::RESERVED)
            .unwrap_or_default()
            .map(|s| s.to_string())
            .collect();
        let group: Vec<String> = options
            .get_many::<String>(options::GROUP_NAME)
            .unwrap_or_default()
            .map(|s| s.to_string())
            .collect();
        if !options.is_present(options::GROUP_NAME) && options.is_present(options::HIDE) {
            return Err(USimpleError::new(
                IOSTAT_CMD_PARSE_ERROR,
                format!("-H must be used with option -g.\n"),
            ));
        }

        let partitions: Vec<String> = options
            .get_one::<String>(options::PARTITIONS)
            .map(|s| {
                s.split(',')
                    .map(|part| part.trim().to_string().to_lowercase())
                    .collect::<Vec<String>>()
            })
            .unwrap_or_default();
        let directory = options
            .value_of(options::DIRECTORY)
            .map(String::from)
            .map_or_else(
                || Ok(None),
                |dir| {
                    if file_exists(&dir) {
                        Ok(Some(dir))
                    } else {
                        Err(USimpleError::new(
                            IOSTAT_CMD_PARSE_ERROR,
                            format!("directory not found: {}\n", dir),
                        ))
                    }
                },
            )?;
        let mut alt_directory: Option<String> = None;
        let mut need_directory: i8 = 0;
        let mut interval = None;
        let mut count = None;

        let mut devices: Vec<String> = vec![];
        let mut reserved_args: Vec<String> = vec![];
        for i in 0..other_args.len() {
            let arg = &other_args[i];
            if need_directory == 1 {
                alt_directory = Some(arg.to_string());
                need_directory = 2;
                continue;
            }

            if arg.starts_with("+f") {
                if !directory.is_none() {
                    return Err(USimpleError::new(IOSTAT_CMD_PARSE_ERROR, ""));
                }
                if let Some(dir) = arg.strip_prefix("+f=") {
                    alt_directory = Some(dir.to_string());
                } else {
                    need_directory = 1;
                }
                continue;
            }
            reserved_args.push(arg.to_string());
        }
        if need_directory == 1 {
            return Err(USimpleError::new(
                IOSTAT_CMD_PARSE_ERROR,
                format!("+f must be followed by a directory\n"),
            ));
        }
        if let Some(dir) = alt_directory.as_ref() {
            if !file_exists(dir) {
                return Err(USimpleError::new(
                    IOSTAT_CMD_PARSE_ERROR,
                    format!("directory not found: {}\n", dir),
                ));
            }
        }
        match reserved_args.len() {
            1 => {
                interval = reserved_args[0].parse::<u64>().map(Some).map_err(|_| {
                    USimpleError::new(
                        IOSTAT_CMD_PARSE_ERROR,
                        format!("invalid interval value: {}\n", reserved_args[0]),
                    )
                })?;
            }
            2 => {
                interval = reserved_args[0].parse::<u64>().map(Some).map_err(|_| {
                    USimpleError::new(
                        IOSTAT_CMD_PARSE_ERROR,
                        format!("invalid interval value: {}\n", reserved_args[0]),
                    )
                })?;
                count = reserved_args[1].parse::<i64>().map(Some).map_err(|_| {
                    USimpleError::new(
                        IOSTAT_CMD_PARSE_ERROR,
                        format!("invalid count value: {}\n", reserved_args[1]),
                    )
                })?;
            }
            len if len > 2 => {
                interval = reserved_args[reserved_args.len() - 2]
                    .parse::<u64>()
                    .map(Some)
                    .map_err(|_| {
                        USimpleError::new(
                            IOSTAT_CMD_PARSE_ERROR,
                            format!(
                                "invalid interval value: {}\n",
                                reserved_args[reserved_args.len() - 2]
                            ),
                        )
                    })?;
                count = reserved_args[reserved_args.len() - 1]
                    .parse::<i64>()
                    .map(Some)
                    .map_err(|_| {
                        USimpleError::new(
                            IOSTAT_CMD_PARSE_ERROR,
                            format!(
                                "invalid count value: {}\n",
                                reserved_args[reserved_args.len() - 1]
                            ),
                        )
                    })?;
                devices = reserved_args[..reserved_args.len() - 2].to_vec();
            }
            _ => {}
        }
        let persistent = if options.is_present(options::PERSISTENT) {
            let persistent_type_dir = get_persistent_type_dir(
                options
                    .value_of(options::PERSISTENT)
                    .unwrap()
                    .to_string()
                    .to_lowercase(),
            );
            match persistent_type_dir {
                Ok(dir) => Some(dir),
                Err(_) => return Err(USimpleError::new(IOSTAT_CMD_PARSE_ERROR, "")),
            }
        } else {
            None
        };
        let output = options
            .value_of(options::JSON)
            .map(String::from)
            .unwrap_or("".to_owned());
        let is_json;
        if output.len() == 0 {
            is_json = false;
        } else if output.len() > 0 && output.to_lowercase() != "json" {
            return Err(USimpleError::new(IOSTAT_CMD_PARSE_ERROR, "only json"));
        } else {
            is_json = true;
        }
        let mut kilobytes = options.is_present(options::KILOBYTES);
        if !kilobytes && !options.is_present(options::MEGABYTES) {
            kilobytes = true;
        }
        Ok(Self {
            cpu: cpu,
            device: device,
            dec: options.value_of_t::<u8>(options::DEC).unwrap_or(2),
            directory: directory,
            alt_directory: alt_directory,
            group: {
                if group.len() > 0 {
                    Some(group)
                } else {
                    None
                }
            },
            hide: options.is_present(options::HIDE),
            human: options.is_present(options::HUMAN) || options.is_present(options::HP),
            persistent: persistent,
            json: is_json,
            kilobytes: kilobytes,
            megabytes: options.is_present(options::MEGABYTES),
            timestamp: options.is_present(options::TIMESTAMP),
            extended: options.is_present(options::EXTENDED),
            omit: options.is_present(options::OMIT),
            partitions: partitions,
            pretty: options.is_present(options::PRETTY)
                || options.is_present(options::HP)
                || options.is_present(options::PERSISTENT),
            registered_device_mapper: options.is_present(options::REGISTERED_DEVICE_MAPPER),
            short: options.is_present(options::SHORT),
            zero: options.is_present(options::ZERO),
            interval: interval,
            count: count.unwrap_or(-1),
            devices: devices,
        })
    }
}

///
pub fn parse_base_cmd_args(args: impl uucore::Args, about: &str, usage: &str) -> UResult<Config> {
    let command = iostat_app(about, usage);
    let arg_list = args.collect_lossy();
    Config::from(&command.try_get_matches_from(arg_list)?)
}

/// iostat_app function to parse the command-line arguments
///
pub fn iostat_app<'a>(about: &'a str, usage: &'a str) -> Command<'a> {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about(about)
        .override_usage(format_usage(usage))
        .infer_long_args(true)
        .arg(Arg::new(options::CPU)
            .short('c')
            .long(options::CPU)
            .help("Display the CPU utilization report"))
        .arg(Arg::new(options::DEVICE)
            .short('d')
            .long(options::DEVICE)
            .help("Display the device utilization report"))
        .arg(Arg::new(options::DEC)
            .long(options::DEC)
            .takes_value(true)
            .help("Specify the number of decimal places to use (0 to 2)"))
        .arg(Arg::new(options::DIRECTORY)
            .short('f')
            .long(options::DIRECTORY)
            .takes_value(true)
            .help("Specify an alternative directory for iostat to read devices statistics"))
        .arg(Arg::new(options::GROUP_NAME)
            .short('g')
            .long(options::GROUP_NAME)
            .takes_value(true)
            .multiple_values(true)
            .help("Display statistics for a group of devices"))
        .arg(Arg::new(options::HIDE)
            .short('H')
            .long(options::HIDE)
            .help("This option must be used with option -g and indicates  that  only\
            \nglobal statistics for the group are to be displayed, and not statistics\
            \nfor individual devices in the group."))
        .arg(Arg::new(options::HUMAN)
            .long(options::HUMAN)
            .help("Print sizes in human readable format"))
        .arg(Arg::new(options::PERSISTENT)
            .short('j')
            .long(options::PERSISTENT)
            .takes_value(true)
            .multiple_values(true)
            .help("-j { ID | LABEL | PATH | UUID | ... } [ device [...] | PARTITIONS ]\
            \n Display persistent device names."))
        .arg(Arg::new(options::JSON)
            .short('o')
            .long(options::JSON)
            .takes_value(true)
            .help("Display the statistics in JSON format"))
        .arg(Arg::new(options::KILOBYTES)
            .short('k')
            .long(options::KILOBYTES)
            .help("Display statistics in kilobytes per second"))
        .arg(Arg::new(options::MEGABYTES)
            .short('m')
            .long(options::MEGABYTES)
            .help("Display statistics in megabytes per second"))
        .arg(Arg::new(options::TIMESTAMP)
            .short('t')
            .long(options::TIMESTAMP)
            .help("Print the time for each report displayed"))
        .arg(Arg::new(options::EXTENDED)
            .short('x')
            .long(options::EXTENDED)
            .help("Display extended statistics"))
        .arg(Arg::new(options::OMIT)
            .short('y')
            .long(options::OMIT)
            .help("Omit first report with statistics since system boot"))
        .arg(Arg::new(options::PARTITIONS)
            .short('p')
            .long(options::PARTITIONS)
            .takes_value(true)
            .help("Display statistics for block devices and all their partitions"))
        .arg(Arg::new(options::PRETTY)
            .long(options::PRETTY)
            .help("Make the Device Utilization Report easier to read by a human"))
        .arg(Arg::new(options::REGISTERED_DEVICE_MAPPER)
            .short('N')
            .long(options::REGISTERED_DEVICE_MAPPER)
            .help("Display the registered device mapper names for any device  mapper\
              \ndevices.  Useful for viewing LVM2 statistics."))
        .arg(Arg::new(options::SHORT)
            .short('s')
            .long(options::SHORT)
            .help("Display a short version of the report that should fit in 80 characters wide screens"))
        .arg(Arg::new(options::ZERO)
            .short('z')
            .long(options::ZERO)
            .help("Tell iostat to omit output for any devices for which there was no activity during the sample period"))
        .arg(Arg::new(options::HP)
            .short('h')
            .help("This option is equivalent to specifying --human --pretty."))
        .arg(Arg::new(options::RESERVED)
            .index(1)
            .multiple_occurrences(true)
            )
}

///
pub fn handle_input(c: &Config) -> UResult<()> {
    let exit_print = json_exit_print(c);
    ctrlc::set_handler(move || {
        println!("{}", exit_print);
        process::exit(0);
    });
    print_gal_header(c)?;
    rw_io_stat_loop(c)?;
    print!("{}", json_exit_print(c));
    Ok(())
}

///
pub fn print_gal_header(c: &Config) -> UResult<()> {
    let uts = Uname::new()?;
    let cpu_nr = get_cpu_nr();
    let cur_date = Local::now().format("%Y-%m-%d").to_string();
    if c.json {
        println!("{}\"sysstat\": {}", LEFT_BRACE, LEFT_BRACE);
        println!("{}\"hosts\": [", return_tab(1));
        println!("{}{}", return_tab(2), LEFT_BRACE);
        println!("{}\"nodename\": \"{}\",", return_tab(3), uts.nodename);
        println!("{}\"sysname\": \"{}\",", return_tab(3), uts.sysname);
        println!("{}\"release\": \"{}\",", return_tab(3), uts.release);
        println!("{}\"machine\": \"{}\",", return_tab(3), uts.machine);
        println!("{}\"number-of-cpus\": \"{}\",", return_tab(3), cpu_nr);
        println!("{}\"date\": \"{}\",", return_tab(3), cur_date);
        print!("{}\"statistics\": [", return_tab(3));
    } else {
        println!(
            "{} {} ({}) \t{} \t_{}_\t({} CPU)",
            uts.sysname, uts.release, uts.nodename, cur_date, uts.machine, cpu_nr
        );
        println!();
    }
    Ok(())
}

fn check_iostat(ioi: &IoStats, ioj: &IoStats) -> bool {
    if (ioi.rd_ios + ioi.wr_ios + ioi.dc_ios + ioi.fl_ios
        < ioj.rd_ios + ioj.wr_ios + ioj.dc_ios + ioj.fl_ios)
        && (ioj.rd_sectors == 0 || ioi.rd_sectors < ioj.rd_sectors)
        && (ioj.wr_sectors == 0 || ioi.wr_sectors < ioj.wr_sectors)
        && (ioj.dc_sectors == 0 || ioi.dc_sectors < ioj.dc_sectors)
    {
        return false;
    }
    return true;
}

///
pub fn rw_io_stat_loop(c: &Config) -> UResult<()> {
    let mut curr = 1;
    let mut st_cpu = vec![StatsCpu::default(); 2];
    let mut uptime_cs: Vec<u64> = vec![0, 2];
    let mut tot_jiffies: Vec<u64> = vec![0, 0];
    let mut device_list = Vec::<IoDevice>::new();
    let iozero = IoStats::default();
    let mut i = 0;

    let mut group_device_list = Vec::<String>::new();
    let mut group_name = String::from("");
    if c.group.is_some() {
        group_name = c.group.as_ref().unwrap()[0].clone();
        group_device_list = c.group.as_ref().unwrap()[1..].to_vec();
    }
    let mut group_device = IoDevice::default();
    group_device.name = format!(" {}", group_name);
    group_device.group = true;
    let mut is_omit = c.omit;
    let mut first_time = true;
    loop {
        uptime_cs[curr] = read_uptime()?;
        if !first_time && c.json {
            print!(",");
        }
        first_time = false;
        if !is_omit && c.json {
            println!("\n{}{}", return_tab(4), LEFT_BRACE);
        }
        if c.timestamp {
            if c.json {
                println!(
                    "{}\"timestamp\": \"{}\",",
                    return_tab(5),
                    Local::now().format("%Y-%m-%d %H:%M:%S")
                );
            } else {
                println!("{}", Local::now().format("%Y-%m-%d %H:%M:%S"));
            }
        }

        if c.cpu {
            let _cpu_read = read_stat_cpu(&mut st_cpu[curr..], 1);
            tot_jiffies[curr] = st_cpu[curr].cpu_user
                + st_cpu[curr].cpu_nice
                + st_cpu[curr].cpu_sys
                + st_cpu[curr].cpu_idle
                + st_cpu[curr].cpu_iowait
                + st_cpu[curr].cpu_hardirq
                + st_cpu[curr].cpu_steal
                + st_cpu[curr].cpu_softirq;
            if !is_omit {
                write_cpu_stat(c, curr, tot_jiffies.clone(), &mut st_cpu);
            }
        }
        if c.device {
            let mut current_group_device_list = group_device_list.clone();
            let mut first_in_group = true;
            let mut group_printed = false;
            if c.partitions.len() > 0 {
                device_list = read_diskstats_stat(c, curr, device_list)?;
            } else {
                device_list = read_sysfs_dlist_stat(c, curr, device_list)?;
            }
            if c.group.is_some() && current_group_device_list.len() == 0 {
                for device in &device_list {
                    current_group_device_list.push(device.name.clone());
                }
            }
            if c.group.is_none() && c.partitions.len() == 0 {
                device_list.sort_by(|a, b| a.name.cmp(&b.name));
            }
            let itv = get_interval(uptime_cs[toggle(curr)], uptime_cs[curr]);
            if !is_omit {
                write_disk_stat_header(c);
            }
            let mut first_device = true;
            for device in &mut device_list {
                if !check_iostat(&device.dev_stats[curr], &device.dev_stats[toggle(curr)]) {
                    device.dev_stats[toggle(curr)] = iozero.clone();
                }
                let ioi = &device.dev_stats[curr];
                let ioj = &device.dev_stats[toggle(curr)];
                if c.partitions.len() == 0
                    && ioi.rd_ios == 0
                    && ioi.wr_ios == 0
                    && ioi.dc_ios == 0
                    && ioi.fl_ios == 0
                {
                    continue;
                }
                if c.zero && check_activity(ioi, ioj) {
                    continue;
                }
                if !c.hide && !is_omit {
                    if !first_device && c.json {
                        println!(",");
                    } else if c.json {
                        println!();
                    }
                    if c.extended {
                        write_ext_stat(c, device, itv, 0, ioi, ioj);
                    } else {
                        write_basic_stat(c, device, *ioi, *ioj, itv);
                    }
                    first_device = false;
                    if !c.json {
                        println!();
                    }
                }
                if c.group.is_none() {
                    continue;
                }
                if current_group_device_list.contains(&device.name) {
                    if first_in_group {
                        group_device.dev_stats[curr] = device.dev_stats[curr].clone();
                        group_device.dev_stats[toggle(curr)] =
                            device.dev_stats[toggle(curr)].clone();
                        first_in_group = false;
                        current_group_device_list.retain(|x| x != &device.name);
                    } else {
                        group_device.dev_stats[curr].combine(device.dev_stats[curr]);
                        current_group_device_list.retain(|x| x != &device.name);
                    }
                }
                if current_group_device_list.len() == 0 && !group_printed {
                    let ioi = &group_device.dev_stats[curr];
                    let ioj = &group_device.dev_stats[toggle(curr)];
                    group_printed = true;
                    if !is_omit {
                        if c.extended {
                            write_ext_stat(c, &group_device, itv, 0, ioi, ioj);
                        } else {
                            write_basic_stat(c, &group_device, *ioi, *ioj, itv);
                        }
                        println!();
                    }
                }
            }
            if c.group.is_some() && !group_printed {
                let ioi = &group_device.dev_stats[curr];
                let ioj = &group_device.dev_stats[toggle(curr)];
                if !is_omit {
                    if c.extended {
                        write_ext_stat(c, &group_device, itv, 0, ioi, ioj);
                    } else {
                        write_basic_stat(c, &group_device, *ioi, *ioj, itv);
                    }
                }
            }
            if c.json && !is_omit {
                println!("\n{}]", return_tab(5));
            }
        }
        if !is_omit {
            if c.json {
                print!("{}{}", return_tab(4), RIGHT_BRACE);
            } else {
                println!();
                println!();
            }
        }

        is_omit = false;
        curr = toggle(curr);
        if c.count > 0 {
            i += 1;
        }
        if !c.interval.is_none() {
            if i == c.count {
                break;
            }
            thread::sleep(Duration::from_secs(c.interval.unwrap_or(0)));
        } else {
            break;
        }
    }
    Ok(())
}

fn check_activity(ioi: &IoStats, ioj: &IoStats) -> bool {
    if ioi.rd_ios == ioj.rd_ios
        && ioi.wr_ios == ioj.wr_ios
        && ioi.dc_ios == ioj.dc_ios
        && ioi.fl_ios == ioj.fl_ios
    {
        return true;
    } else {
        false
    }
}

fn write_cpu_stat(c: &Config, curr: usize, tot_jiffies: Vec<u64>, st_cpu: &mut Vec<StatsCpu>) {
    let deltot_jiffies = get_interval(tot_jiffies[toggle(curr)], tot_jiffies[curr]);
    if c.json {
        write_json_cpu_stat(c, curr, deltot_jiffies, st_cpu);
    } else {
        write_plain_cpu_stat(c, curr, deltot_jiffies, st_cpu);
    }
}

fn write_plain_cpu_stat(c: &Config, curr: usize, deltot_jiffies: u64, st_cpu: &mut Vec<StatsCpu>) {
    println!("avg-cpu:  %user   %nice %system %iowait  %steal   %idle");
    print!("       ");
    let values = vec![
        ll_sp_value(
            st_cpu[toggle(curr)].cpu_user,
            st_cpu[curr].cpu_user,
            deltot_jiffies,
        ),
        ll_sp_value(
            st_cpu[toggle(curr)].cpu_nice,
            st_cpu[curr].cpu_nice,
            deltot_jiffies,
        ),
        ll_sp_value(
            st_cpu[toggle(curr)].cpu_sys
                + st_cpu[toggle(curr)].cpu_softirq
                + st_cpu[toggle(curr)].cpu_hardirq,
            st_cpu[curr].cpu_sys + st_cpu[curr].cpu_softirq + st_cpu[curr].cpu_hardirq,
            deltot_jiffies,
        ),
        ll_sp_value(
            st_cpu[toggle(curr)].cpu_iowait,
            st_cpu[curr].cpu_iowait,
            deltot_jiffies,
        ),
        ll_sp_value(
            st_cpu[toggle(curr)].cpu_steal,
            st_cpu[curr].cpu_steal,
            deltot_jiffies,
        ),
    ];
    cprintf_xpc(
        c.human,
        Some("XHIGH"),
        values.len(),
        7,
        c.dec.into(),
        &values,
    );

    let value = vec![{
        if st_cpu[curr].cpu_idle < st_cpu[toggle(curr)].cpu_idle {
            0.0
        } else {
            ll_sp_value(
                st_cpu[toggle(curr)].cpu_idle,
                st_cpu[curr].cpu_idle,
                deltot_jiffies,
            )
        }
    }];
    cprintf_xpc(c.human, Some("XLOW"), values.len(), 7, c.dec.into(), &value);
    println!();
    println!();
}

fn write_json_cpu_stat(c: &Config, curr: usize, deltot_jiffies: u64, st_cpu: &mut Vec<StatsCpu>) {
    let values = vec![
        ll_sp_value(
            st_cpu[toggle(curr)].cpu_user,
            st_cpu[curr].cpu_user,
            deltot_jiffies,
        ),
        ll_sp_value(
            st_cpu[toggle(curr)].cpu_nice,
            st_cpu[curr].cpu_nice,
            deltot_jiffies,
        ),
        ll_sp_value(
            st_cpu[toggle(curr)].cpu_sys
                + st_cpu[toggle(curr)].cpu_softirq
                + st_cpu[toggle(curr)].cpu_hardirq,
            st_cpu[curr].cpu_sys + st_cpu[curr].cpu_softirq + st_cpu[curr].cpu_hardirq,
            deltot_jiffies,
        ),
        ll_sp_value(
            st_cpu[toggle(curr)].cpu_iowait,
            st_cpu[curr].cpu_iowait,
            deltot_jiffies,
        ),
        ll_sp_value(
            st_cpu[toggle(curr)].cpu_steal,
            st_cpu[curr].cpu_steal,
            deltot_jiffies,
        ),
        if st_cpu[curr].cpu_idle < st_cpu[toggle(curr)].cpu_idle {
            0.0
        } else {
            ll_sp_value(
                st_cpu[toggle(curr)].cpu_idle,
                st_cpu[curr].cpu_idle,
                deltot_jiffies,
            )
        },
    ];
    print!("{}\"avg-cpu\":  {}\"user\": {:.2}, \"nice\": {:.2}, \"system\": {:.2}, \"iowait\": {:.2}, \"steal\": {:.2}, \"idle\": {:.2}{}",
    return_tab(5),LEFT_BRACE,values[0],values[1],values[2],values[3],values[4],values[5],RIGHT_BRACE);
    if c.device {
        println!(",");
    } else {
        println!();
    }
}

fn read_stat_cpu(st_cpu: &mut [StatsCpu], nr_alloc: usize) -> isize {
    let path = Path::new(STAT);
    let file = File::open(&path).unwrap_or_else(|err| {
        eprintln!("Cannot open {}: {}", STAT, err);
        std::process::exit(2);
    });

    let mut cpu_read = 0;
    let reader = io::BufReader::new(file);

    for line in reader.lines() {
        let line = line.unwrap();

        if line.starts_with("cpu ") {
            let mut stats = StatsCpu::default();
            let parts: Vec<&str> = line.split_whitespace().collect();

            if parts.len() >= 11 {
                stats.cpu_user = u64::from_str(parts[1]).unwrap_or(0);
                stats.cpu_nice = u64::from_str(parts[2]).unwrap_or(0);
                stats.cpu_sys = u64::from_str(parts[3]).unwrap_or(0);
                stats.cpu_idle = u64::from_str(parts[4]).unwrap_or(0);
                stats.cpu_iowait = u64::from_str(parts[5]).unwrap_or(0);
                stats.cpu_hardirq = u64::from_str(parts[6]).unwrap_or(0);
                stats.cpu_softirq = u64::from_str(parts[7]).unwrap_or(0);
                stats.cpu_steal = u64::from_str(parts[8]).unwrap_or(0);
                stats.cpu_guest = u64::from_str(parts[9]).unwrap_or(0);
                stats.cpu_guest_nice = u64::from_str(parts[10]).unwrap_or(0);
            }

            if cpu_read == 0 {
                st_cpu[0] = stats;
                cpu_read = 1;
            }

            if nr_alloc == 1 {
                break;
            }
        } else if line.starts_with("cpu") {
            let mut stats = StatsCpu::default();
            let parts: Vec<&str> = line.split_whitespace().collect();
            let proc_nr = usize::from_str(parts[0].trim_start_matches("cpu")).unwrap_or(0);

            if parts.len() >= 11 {
                stats.cpu_user = u64::from_str(parts[1]).unwrap_or(0);
                stats.cpu_nice = u64::from_str(parts[2]).unwrap_or(0);
                stats.cpu_sys = u64::from_str(parts[3]).unwrap_or(0);
                stats.cpu_idle = u64::from_str(parts[4]).unwrap_or(0);
                stats.cpu_iowait = u64::from_str(parts[5]).unwrap_or(0);
                stats.cpu_hardirq = u64::from_str(parts[6]).unwrap_or(0);
                stats.cpu_softirq = u64::from_str(parts[7]).unwrap_or(0);
                stats.cpu_steal = u64::from_str(parts[8]).unwrap_or(0);
                stats.cpu_guest = u64::from_str(parts[9]).unwrap_or(0);
                stats.cpu_guest_nice = u64::from_str(parts[10]).unwrap_or(0);
            }

            if proc_nr + 2 > nr_alloc {
                return -1;
            }

            st_cpu[proc_nr + 1] = stats;

            if proc_nr + 2 > cpu_read {
                cpu_read = proc_nr + 2;
            }
        }
    }

    cpu_read as isize
}

fn get_device_name(c: &Config, device_name: String) -> String {
    let persistent_type_dir = match &c.persistent {
        Some(dir) => dir,
        None => return device_name,
    };

    let persistent_path = Path::new(persistent_type_dir);

    let mut entries: Vec<PathBuf> = match fs::read_dir(persistent_path) {
        Ok(entries) => entries.flatten().map(|e| e.path()).collect(),
        Err(_) => return device_name,
    };
    entries.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
    for entry in entries {
        let link_target = match fs::read_link(&entry) {
            Ok(target) => target,
            Err(_) => continue,
        };

        if link_target.file_name() == Some(OsStr::new(&device_name)) {
            return entry
                .file_name()
                .unwrap_or(OsStr::new(&device_name))
                .to_str()
                .unwrap()
                .to_string();
        }
    }

    device_name
}

fn read_sysfs_dlist_stat(
    c: &Config,
    curr: usize,
    dev_list: Vec<IoDevice>,
) -> Result<Vec<IoDevice>, io::Error> {
    return read_sysfs_all_devices_stat_work(c, curr, dev_list);
}

fn read_diskstats_stat(
    c: &Config,
    curr: usize,
    dev_list: Vec<IoDevice>,
) -> Result<Vec<IoDevice>, io::Error> {
    return read_diskstats_stat_work(c, curr, dev_list);
}

fn read_diskstats_stat_work(
    c: &Config,
    curr: usize,
    mut dev_list: Vec<IoDevice>,
) -> Result<Vec<IoDevice>, io::Error> {
    let file = File::open(DISKSTATS)?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        let mut sdev = IoStats::default();

        let fields: Vec<&str> = line.split_whitespace().collect();
        let mut major = None;
        let mut minor = None;
        let mut dev_name = String::new();
        if fields.len() >= 14 {
            major = Some(fields[0].parse::<u32>().unwrap_or(0));
            minor = Some(fields[1].parse::<u32>().unwrap_or(0));
            dev_name = fields[2].to_string();
            sdev.rd_ios = fields[3].parse::<u64>().unwrap_or(0);
            sdev.rd_merges = fields[4].parse::<u64>().unwrap_or(0);
            sdev.rd_sectors = fields[5].parse::<u64>().unwrap_or(0);
            sdev.rd_ticks = fields[6].parse::<u32>().unwrap_or(0);
            sdev.wr_ios = fields[7].parse::<u64>().unwrap_or(0);
            sdev.wr_merges = fields[8].parse::<u64>().unwrap_or(0);
            sdev.wr_sectors = fields[9].parse::<u64>().unwrap_or(0);
            sdev.wr_ticks = fields[10].parse::<u32>().unwrap_or(0);
            sdev.ios_pgr = fields[11].parse::<u32>().unwrap_or(0);
            sdev.tot_ticks = fields[12].parse::<u32>().unwrap_or(0);
            sdev.rq_ticks = fields[13].parse::<u32>().unwrap_or(0);

            if fields.len() >= 18 {
                sdev.dc_ios = fields[14].parse::<u64>().unwrap_or(0);
                sdev.dc_merges = fields[15].parse::<u64>().unwrap_or(0);
                sdev.dc_sectors = fields[16].parse::<u64>().unwrap_or(0);
                sdev.dc_ticks = fields[17].parse::<u32>().unwrap_or(0);
            }

            if fields.len() >= 20 {
                sdev.fl_ios = fields[18].parse::<u64>().unwrap_or(0);
                sdev.fl_ticks = fields[19].parse::<u32>().unwrap_or(0);
            }
        } else if fields.len() == 7 {
            sdev.rd_ios = fields[3].parse::<u64>().unwrap_or(0);
            sdev.rd_sectors = fields[4].parse::<u64>().unwrap_or(0);
            sdev.wr_ios = fields[5].parse::<u64>().unwrap_or(0);
            sdev.wr_sectors = fields[6].parse::<u64>().unwrap_or(0);
        } else {
            continue;
        }
        let mut match_device: bool = false;
        if c.partitions.len() > 0 && c.partitions[0] == "all" {
            match_device = true;
        } else {
            for p in &c.partitions {
                if dev_name.starts_with(p) {
                    match_device = true;
                    break;
                }
            }
        }
        if !match_device {
            continue;
        }
        match find_device_in_list(&dev_list, &fields[2].to_string()) {
            Some(idx) => {
                dev_list[idx].dev_stats[curr] = sdev;
            }
            None => {
                let mut d = IoDevice::default();
                d.name = fields[2].to_string();
                d.major = major;
                d.minor = minor;
                d.dev_stats[curr] = sdev;
                d.dev_stats[toggle(curr)] = IoStats::default();
                dev_list.push(d);
            }
        };
    }

    Ok(dev_list)
}

fn read_sysfs_all_devices_stat_work(
    c: &Config,
    curr: usize,
    mut dev_list: Vec<IoDevice>,
) -> Result<Vec<IoDevice>, io::Error> {
    let mut entries = Vec::new();

    if c.directory.is_none() {
        let dir_entries = fs::read_dir(SYSFS_BLOCK)?;
        for entry in dir_entries {
            entries.push(entry);
        }
    }
    if c.directory.is_some() {
        let dir_entries = fs::read_dir(c.directory.as_ref().unwrap())?;
        for entry in dir_entries {
            entries.push(entry);
        }
    } else if c.alt_directory.is_some() {
        let dir_entries = fs::read_dir(c.alt_directory.as_ref().unwrap())?;
        for entry in dir_entries {
            entries.push(entry);
        }
    }

    for entry in entries {
        let entry = entry?;

        let file_name = entry.file_name().into_string().unwrap();
        if file_name == "." || file_name == ".." {
            continue;
        }
        if !c.devices.is_empty() && !c.devices.contains(&file_name) {
            continue;
        }
        let dfile = format!("{}/{}", entry.path().to_str().unwrap(), "stat");
        if !file_exists(&dfile) {
            continue;
        }
        let sdev = read_sysfs_file_stat_work(&dfile)?;
        match find_device_in_list(&dev_list, &file_name) {
            Some(idx) => {
                dev_list[idx].dev_stats[curr] = sdev;
                continue;
            }
            None => (),
        };
        let mut d = IoDevice::default();
        match get_major_minor_nr(&file_name) {
            Ok((major, minor)) => {
                d.major = Some(major);
                d.minor = Some(minor);
            }
            Err(e) => {
                eprintln!("Failed to get major and minor numbers: {}", e);
            }
        }
        d.dev_stats[curr] = sdev;
        d.dev_stats[toggle(curr)] = IoStats::default();
        d.name = file_name;
        dev_list.push(d);
    }

    Ok(dev_list)
}

fn find_device_in_list(dev_list: &Vec<IoDevice>, name: &str) -> Option<usize> {
    for (i, d) in dev_list.iter().enumerate() {
        if d.name == name {
            return Some(i);
        }
    }
    None
}

fn read_sysfs_file_stat_work(filename: &str) -> Result<IoStats, io::Error> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);

    let mut sdev = IoStats::default();

    // Read the first line from the file
    if let Some(Ok(line)) = reader.lines().next() {
        let fields: Vec<u64> = line
            .split_whitespace()
            .filter_map(|s| s.parse().ok())
            .collect();

        if fields.len() >= 11 {
            sdev.rd_ios = fields[0];
            sdev.rd_merges = fields[1];
            sdev.rd_sectors = fields[2];
            sdev.rd_ticks = fields[3] as u32;
            sdev.wr_ios = fields[4];
            sdev.wr_merges = fields[5];
            sdev.wr_sectors = fields[6];
            sdev.wr_ticks = fields[7] as u32;
            sdev.ios_pgr = fields[8] as u32;
            sdev.tot_ticks = fields[9] as u32;
            sdev.rq_ticks = fields[10] as u32;
            if fields.len() >= 15 {
                sdev.dc_ios = fields[11];
                sdev.dc_merges = fields[12];
                sdev.dc_sectors = fields[13];
                sdev.dc_ticks = fields[14] as u32;
            }
            if fields.len() >= 17 {
                sdev.fl_ios = fields[15];
                sdev.fl_ticks = fields[16] as u32;
            }
        } else if fields.len() >= 4 {
            sdev.rd_ios = fields[0];
            sdev.rd_sectors = fields[1];
            sdev.wr_ios = fields[4];
            sdev.wr_sectors = fields[3];
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid number of fields in the stat file",
            ));
        }
    }
    Ok(sdev)
}

fn write_disk_stat_header(c: &Config) {
    if c.json {
        print!("{}\"disks\": [", return_tab(5));
        return;
    }
    let mut units = "kB";
    let mut spc = " ";
    if c.megabytes {
        units = "MB";
        spc = " ";
    }
    if !c.pretty {
        print!("Device       ");
    }
    if c.extended {
        if c.short {
            print!(
                "      tps     {}{}/s    rqm/s   await  areq-sz  aqu-sz  %%util",
                spc, units
            );
        } else {
            print!(
                "     r/s    {}r{}/s   rrqm/s  %rrqm r_await rareq-sz",
                spc, units
            );
            print!(
                "     w/s    {}w{}/s   wrqm/s  %wrqm w_await wareq-sz",
                spc, units
            );
            print!(
                "     d/s    {}d{}/s   drqm/s  %drqm d_await dareq-sz",
                spc, units
            );
            print!("     f/s f_await  aqu-sz  %util");
        }
    } else {
        if c.short {
            print!(
                "      tps   {}{}_read/s    {}{}_w+d/s   {}{}_read    {}{}_w+d",
                spc, units, spc, units, spc, units, spc, units
            );
        } else {
            print!(
            "      tps   {}{}_read/s   {}{}_wrtn/s   {}{}_dscd/s   {}{}_read   {}{}_wrtn   {}{}_dscd",
            spc, units, spc, units, spc, units, spc, units, spc, units, spc, units,
        );
        }
    }
    if c.pretty {
        print!(" Device");
    }
    println!();
}

fn write_basic_stat(c: &Config, device: &IoDevice, ioi: IoStats, ioj: IoStats, itv: u64) {
    let mut fctr = 2;
    if c.megabytes {
        fctr = 2048;
    }
    if c.kilobytes {
        fctr = 2;
    }
    let mut rd_sec = ioi.rd_sectors.wrapping_sub(ioj.rd_sectors);
    if ioi.rd_sectors < ioj.rd_sectors && ioj.rd_sectors <= 0xffffffff {
        rd_sec &= 0xffffffff;
    }
    let mut wr_sec = ioi.wr_sectors.wrapping_sub(ioj.wr_sectors);
    if ioi.wr_sectors < ioj.wr_sectors && ioj.wr_sectors <= 0xffffffff {
        wr_sec &= 0xffffffff;
    }
    let mut dc_sec = ioi.dc_sectors.wrapping_sub(ioj.dc_sectors);
    if ioi.dc_sectors < ioj.dc_sectors && ioj.dc_sectors <= 0xffffffff {
        dc_sec &= 0xffffffff;
    }

    let mut device_name = device.name.clone();
    if !device.group {
        if c.registered_device_mapper {
            let map_device_name =
                transform_devmapname(device.major.unwrap_or(0), device.minor.unwrap_or(0));
            if map_device_name.is_some() {
                device_name = map_device_name.unwrap();
            } else {
                device_name = get_devmapname(&device_name).unwrap_or(device_name);
            }
        } else {
            device_name = get_device_name(c, device_name);
        }
    }
    if c.json {
        write_json_basic_stat(c, device_name, ioi, ioj, itv, fctr, rd_sec, wr_sec, dc_sec);
    } else {
        write_plain_basic_stat(c, device_name, ioi, ioj, itv, fctr, rd_sec, wr_sec, dc_sec);
    }
}
fn write_json_basic_stat(
    c: &Config,
    device_name: String,
    ioi: IoStats,
    ioj: IoStats,
    itv: u64,
    fctr: u64,
    rd_sec: u64,
    wr_sec: u64,
    dc_sec: u64,
) {
    let mut output = String::new();
    let indent = return_tab(6);

    write!(
        output,
        "{}{{\"disk_device\": \"{}\", \"tps\": {:.2}, ",
        indent,
        device_name,
        s_value(
            ioj.rd_ios + ioj.wr_ios + ioj.dc_ios,
            ioi.rd_ios + ioi.wr_ios + ioi.dc_ios,
            itv
        )
    )
    .unwrap();

    let line = if c.kilobytes {
        format!(
            "\"kB_read/s\": {:.2}, \"kB_wrtn/s\": {:.2}, \"kB_dscd/s\": {:.2}, \"kB_read\": {}, \"kB_wrtn\": {}, \"kB_dscd\": {}}}",
            s_value(ioj.rd_sectors, ioi.rd_sectors, itv) / fctr as f64,
            s_value(ioj.wr_sectors, ioi.wr_sectors, itv) / fctr as f64,
            s_value(ioj.dc_sectors, ioi.dc_sectors, itv) / fctr as f64,
            rd_sec / fctr as u64,
            wr_sec / fctr as u64,
            dc_sec / fctr as u64
        )
    } else if c.megabytes {
        format!(
            "\"MB_read/s\": {:.2}, \"MB_wrtn/s\": {:.2}, \"MB_dscd/s\": {:.2}, \"MB_read\": {}, \"MB_wrtn\": {}, \"MB_dscd\": {}}}",
            s_value(ioj.rd_sectors, ioi.rd_sectors, itv) / fctr as f64,
            s_value(ioj.wr_sectors, ioi.wr_sectors, itv) / fctr as f64,
            s_value(ioj.dc_sectors, ioi.dc_sectors, itv) / fctr as f64,
            rd_sec / fctr as u64,
            wr_sec / fctr as u64,
            dc_sec / fctr as u64
        )
    } else {
        format!(
            "\"Blk_read/s\": {:.2}, \"Blk_wrtn/s\": {:.2}, \"Blk_dscd/s\": {:.2}, \"Blk_read\": {}, \"Blk_wrtn\": {}, \"Blk_dscd\": {}}}",
            s_value(ioj.rd_sectors, ioi.rd_sectors, itv) / fctr as f64,
            s_value(ioj.wr_sectors, ioi.wr_sectors, itv) / fctr as f64,
            s_value(ioj.dc_sectors, ioi.dc_sectors, itv) / fctr as f64,
            rd_sec / fctr as u64,
            wr_sec / fctr as u64,
            dc_sec / fctr as u64
        )
    };
    output.push_str(&line);
    print!("{}", output);
}
fn write_plain_basic_stat(
    c: &Config,
    device_name: String,
    ioi: IoStats,
    ioj: IoStats,
    itv: u64,
    fctr: u64,
    rd_sec: u64,
    wr_sec: u64,
    dc_sec: u64,
) {
    let rsectors = s_value(ioj.rd_sectors, ioi.rd_sectors, itv) / fctr as f64;
    let wsectors = s_value(ioj.wr_sectors, ioi.wr_sectors, itv) / fctr as f64;
    let dsectors = s_value(ioj.dc_sectors, ioi.dc_sectors, itv) / fctr as f64;

    if !c.pretty {
        print!("{}", SC_ITEM_NAME);
        print!("{:13}", device_name);
        print!("{}", SC_NORMAL);
    }
    cprintf_f(
        false,
        false,
        1,
        8,
        c.dec.into(),
        &[s_value(
            ioj.rd_ios + ioj.wr_ios + ioj.dc_ios,
            ioi.rd_ios + ioi.wr_ios + ioi.dc_ios,
            itv,
        )],
    );
    if c.short {
        cprintf_f(
            false,
            false,
            2,
            12,
            c.dec.into(),
            &[rsectors, wsectors + dsectors],
        );
        cprintf_u64(c.human, 2, 10, &[rd_sec / fctr, (wr_sec + dc_sec) / fctr])
    } else {
        cprintf_f(
            c.human,
            false,
            3,
            12,
            c.dec.into(),
            &[rsectors, wsectors, dsectors],
        );
        cprintf_u64(
            c.human,
            3,
            10,
            &[rd_sec / fctr, wr_sec / fctr, dc_sec / fctr],
        );
    }

    if c.pretty {
        print!("{}", SC_ITEM_NAME);
        print!(" {:13}", device_name);
        print!("{}", SC_NORMAL);
    }
}

fn write_ext_stat(
    c: &Config,
    device: &IoDevice,
    itv: u64,
    hpart: u8,
    ioi: &IoStats,
    ioj: &IoStats,
) {
    let mut xds: ExtDiskStats = ExtDiskStats::default();
    let mut xios = ExtIoStats::default();

    if hpart == 4 || hpart == 0 || c.short {
        let sdc = compute_sdc_sdp(ioi);
        let sdp = compute_sdc_sdp(ioj);
        xds.compute_ext_disk_stats(&sdc, &sdp, itv);
    }
    xios.rsectors = compute_sectors(ioi.rd_sectors, ioj.rd_sectors, itv);
    xios.wsectors = compute_sectors(ioi.wr_sectors, ioj.wr_sectors, itv);
    xios.dsectors = compute_sectors(ioi.dc_sectors, ioj.dc_sectors, itv);

    if c.short {
        xios.sectors = xios.rsectors + xios.wsectors + xios.dsectors;
    } else {
        compute_extended_stats(hpart, ioi, ioj, &mut xios);
    }

    if c.json {
        write_json_ext_stat(c, device, ioi, ioj, itv, &xds, &xios);
    } else {
        write_plain_ext_stat(c, device, ioi, ioj, itv, hpart, &xds, &xios);
    }
}

fn compute_sdc_sdp(ioi: &IoStats) -> StatsDisk {
    StatsDisk {
        nr_ios: ioi.rd_ios + ioi.wr_ios + ioi.dc_ios,
        tot_ticks: ioi.tot_ticks,
        rd_ticks: ioi.rd_ticks,
        wr_ticks: ioi.wr_ticks,
        dc_ticks: ioi.dc_ticks,
        rd_sect: ioi.rd_sectors,
        wr_sect: ioi.wr_sectors,
        dc_sect: ioi.dc_sectors,
    }
}

fn compute_sectors(io_current: u64, io_previous: u64, itv: u64) -> f64 {
    if io_current < io_previous {
        0.0
    } else {
        s_value(io_previous, io_current, itv)
    }
}

fn compute_extended_stats(hpart: u8, ioi: &IoStats, ioj: &IoStats, xios: &mut ExtIoStats) {
    if hpart == 1 || hpart == 0 {
        xios.rrqm_pc = compute_rrqm_pc(ioi, ioj);
        xios.r_await = compute_r_await(ioi, ioj);
        xios.rarqsz = compute_rarqsz(ioi, ioj);
    }
    if hpart == 2 || hpart == 0 {
        xios.wrqm_pc = compute_wrqm_pc(ioi, ioj);
        xios.w_await = compute_w_await(ioi, ioj);
        xios.warqsz = compute_warqsz(ioi, ioj);
    }
    if hpart == 3 || hpart == 0 {
        xios.drqm_pc = compute_drqm_pc(ioi, ioj);
        xios.d_await = compute_d_await(ioi, ioj);
        xios.darqsz = compute_darqsz(ioi, ioj);
    }
    if hpart == 4 || hpart == 0 {
        xios.f_await = compute_f_await(ioi, ioj);
    }
}

fn compute_rrqm_pc(ioi: &IoStats, ioj: &IoStats) -> f64 {
    let delta = (ioi.rd_merges - ioj.rd_merges) + (ioi.rd_ios - ioj.rd_ios);
    if delta > 0 {
        (ioi.rd_merges - ioj.rd_merges) as f64 / delta as f64 * 100.0
    } else {
        0.0
    }
}

fn compute_wrqm_pc(ioi: &IoStats, ioj: &IoStats) -> f64 {
    let delta = (ioi.wr_merges - ioj.wr_merges) + (ioi.wr_ios - ioj.wr_ios);
    if delta > 0 {
        (ioi.wr_merges - ioj.wr_merges) as f64 / delta as f64 * 100.0
    } else {
        0.0
    }
}

fn compute_drqm_pc(ioi: &IoStats, ioj: &IoStats) -> f64 {
    let delta = (ioi.dc_merges - ioj.dc_merges) + (ioi.dc_ios - ioj.dc_ios);
    if delta > 0 {
        (ioi.dc_merges - ioj.dc_merges) as f64 / delta as f64 * 100.0
    } else {
        0.0
    }
}

fn compute_r_await(ioi: &IoStats, ioj: &IoStats) -> f64 {
    if ioi.rd_ios > ioj.rd_ios {
        (ioi.rd_ticks - ioj.rd_ticks) as f64 / (ioi.rd_ios - ioj.rd_ios) as f64
    } else {
        0.0
    }
}

fn compute_w_await(ioi: &IoStats, ioj: &IoStats) -> f64 {
    if ioi.wr_ios > ioj.wr_ios {
        (ioi.wr_ticks - ioj.wr_ticks) as f64 / (ioi.wr_ios - ioj.wr_ios) as f64
    } else {
        0.0
    }
}

fn compute_d_await(ioi: &IoStats, ioj: &IoStats) -> f64 {
    if ioi.dc_ios > ioj.dc_ios {
        (ioi.dc_ticks - ioj.dc_ticks) as f64 / (ioi.dc_ios - ioj.dc_ios) as f64
    } else {
        0.0
    }
}

fn compute_f_await(ioi: &IoStats, ioj: &IoStats) -> f64 {
    if ioi.fl_ios > ioj.fl_ios {
        (ioi.fl_ticks - ioj.fl_ticks) as f64 / (ioi.fl_ios - ioj.fl_ios) as f64
    } else {
        0.0
    }
}

fn compute_rarqsz(ioi: &IoStats, ioj: &IoStats) -> f64 {
    if ioi.rd_ios > ioj.rd_ios {
        (ioi.rd_sectors - ioj.rd_sectors) as f64 / (ioi.rd_ios - ioj.rd_ios) as f64
    } else {
        0.0
    }
}

fn compute_warqsz(ioi: &IoStats, ioj: &IoStats) -> f64 {
    if ioi.wr_ios > ioj.wr_ios {
        (ioi.wr_sectors - ioj.wr_sectors) as f64 / (ioi.wr_ios - ioj.wr_ios) as f64
    } else {
        0.0
    }
}

fn compute_darqsz(ioi: &IoStats, ioj: &IoStats) -> f64 {
    if ioi.dc_ios > ioj.dc_ios {
        (ioi.dc_sectors - ioj.dc_sectors) as f64 / (ioi.dc_ios - ioj.dc_ios) as f64
    } else {
        0.0
    }
}

fn write_json_ext_stat(
    c: &Config,
    device: &IoDevice,
    ioi: &IoStats,
    ioj: &IoStats,
    itv: u64,
    xds: &ExtDiskStats,
    xios: &ExtIoStats,
) {
    print!("{}{{\"disk_device\": \"{}\", ", return_tab(6), device.name);
    let mut fctr = 2;
    if c.megabytes {
        fctr = 2048;
    }
    if c.kilobytes {
        fctr = 2;
    }

    if c.short {
        let tps = if ioi.rd_ios + ioi.wr_ios + ioi.dc_ios < ioj.rd_ios + ioj.wr_ios + ioj.dc_ios {
            0.0
        } else {
            s_value(
                ioj.rd_ios + ioj.wr_ios + ioj.dc_ios,
                ioi.rd_ios + ioi.wr_ios + ioi.dc_ios,
                itv,
            )
        };

        print!("\"tps\": {:.2}, \"", tps);
        if c.megabytes {
            print!("MB/s");
        } else if c.kilobytes {
            print!("kB/s");
        } else {
            print!("sec/s");
        }

        print!(
            "\": {:.2}, \"rqm/s\": {:.2}, \"await\": {:.2}, \"areq-sz\": {:.2}, \"aqu-sz\": {:.2}, ",
            xios.sectors as f64 / fctr as f64,
            if ioi.rd_merges + ioi.wr_merges + ioi.dc_merges < ioj.rd_merges + ioj.wr_merges + ioj.dc_merges {
                0.0
            } else {
                s_value(
                    ioj.rd_merges + ioj.wr_merges + ioj.dc_merges,
                    ioi.rd_merges + ioi.wr_merges + ioi.dc_merges,
                    itv,
                )
            },
            xds.await1,
            xds.arqsz / 2.0,
            if ioi.rq_ticks < ioj.rq_ticks {
                0.0
            } else {
                s_value(ioj.rq_ticks.into(), ioi.rq_ticks.into(), itv) / 1000.0
            }
        );
    } else {
        print!(
            "\"r/s\": {:.2}, \"w/s\": {:.2}, \"d/s\": {:.2}, \"f/s\": {:.2}, ",
            if ioi.rd_ios < ioj.rd_ios {
                0.0
            } else {
                s_value(ioj.rd_ios, ioi.rd_ios, itv)
            },
            if ioi.wr_ios < ioj.wr_ios {
                0.0
            } else {
                s_value(ioj.wr_ios, ioi.wr_ios, itv)
            },
            if ioi.dc_ios < ioj.dc_ios {
                0.0
            } else {
                s_value(ioj.dc_ios, ioi.dc_ios, itv)
            },
            if ioi.fl_ios < ioj.fl_ios {
                0.0
            } else {
                s_value(ioj.fl_ios, ioi.fl_ios, itv)
            }
        );

        if c.megabytes {
            print!(
                "\"rMB/s\": {:0.2}, \"wMB/s\": {:0.2}, \"dMB/s\": {:0.2}, ",
                xios.rsectors as f64 / fctr as f64,
                xios.wsectors as f64 / fctr as f64,
                xios.dsectors as f64 / fctr as f64
            );
        } else if c.kilobytes {
            print!(
                "\"rkB/s\": {:0.2}, \"wkB/s\":{:0.2}, \"dkB/s\": {:0.2}, ",
                xios.rsectors as f64 / fctr as f64,
                xios.wsectors as f64 / fctr as f64,
                xios.dsectors as f64 / fctr as f64
            );
        } else {
            print!(
                "\"rsec/s\": {:0.2}, \"wsec/s\": {:0.2}, \"dsec/s\": {:0.2}, ",
                xios.rsectors as f64 / fctr as f64,
                xios.wsectors as f64 / fctr as f64,
                xios.dsectors as f64 / fctr as f64
            );
        }

        print!(
            "\"rrqm/s\": {:.2}, \"wrqm/s\": {:.2}, \"drqm/s\": {:.2}, \"rrqm\": {:.2}, \"wrqm\": {:.2}, \"drqm\": {:.2}, \"r_await\": {:.2}, \"w_await\": {:.2}, \"d_await\": {:.2}, \"f_await\": {:.2}, \"rareq-sz\": {:.2}, \"wareq-sz\": {:.2}, \"dareq-sz\": {:.2}, \"aqu-sz\": {:.2}, ",
            if ioi.rd_merges < ioj.rd_merges {
                0.0
            } else {
                s_value(ioj.rd_merges, ioi.rd_merges, itv)
            },
            if ioi.wr_merges < ioj.wr_merges {
                0.0
            } else {
                s_value(ioj.wr_merges, ioi.wr_merges, itv)
            },
            if ioi.dc_merges < ioj.dc_merges {
                0.0
            } else {
                s_value(ioj.dc_merges, ioi.dc_merges, itv)
            },
            xios.rrqm_pc,
            xios.wrqm_pc,
            xios.drqm_pc,
            xios.r_await,
            xios.w_await,
            xios.d_await,
            xios.f_await,
            xios.rarqsz / 2.0,
            xios.warqsz / 2.0,
            xios.darqsz / 2.0,
            if ioi.rq_ticks < ioj.rq_ticks {
                0.0
            } else {
                s_value(ioj.rq_ticks.into(), ioi.rq_ticks.into(), itv) / 1000.0
            }
        );
    }

    print!("\"util\": {:.2}}}", xds.util / 10.0);
}

fn write_plain_ext_stat(
    c: &Config,
    device: &IoDevice,
    ioi: &IoStats,
    ioj: &IoStats,
    itv: u64,
    hpart: u8,
    xds: &ExtDiskStats,
    xios: &ExtIoStats,
) {
    let mut fctr = 2;
    if c.megabytes {
        fctr = 2048;
    }
    if c.kilobytes {
        fctr = 2;
    }
    let device_name = device.name.clone();

    if device.group {
        return;
    }

    if !c.pretty {
        print!("{}", SC_ITEM_NAME);
        print!("{:13}", device_name);
        print!("{}", SC_NORMAL);
    }

    if c.short {
        let tps = if ioi.rd_ios + ioi.wr_ios + ioi.dc_ios < ioj.rd_ios + ioj.wr_ios + ioj.dc_ios {
            0.0
        } else {
            s_value(
                ioj.rd_ios + ioj.wr_ios + ioj.dc_ios,
                ioi.rd_ios + ioi.wr_ios + ioi.dc_ios,
                itv,
            )
        };
        cprintf_f(false, false, 1, 8, c.dec.into(), &[tps]);
        let mut sectors = xios.rsectors + xios.wsectors + xios.dsectors;
        if !c.human {
            sectors = sectors / fctr as f64;
        }
        cprintf_f(c.human, false, 1, 9, c.dec.into(), &[sectors]);
        let rqm_s = if ioi.rd_merges + ioi.wr_merges + ioi.dc_merges
            < ioj.rd_merges + ioj.wr_merges + ioj.dc_merges
        {
            0.0
        } else {
            s_value(
                ioj.rd_merges + ioj.wr_merges + ioj.dc_merges,
                ioi.rd_merges + ioi.wr_merges + ioi.dc_merges,
                itv,
            )
        };
        cprintf_f(false, false, 1, 8, c.dec.into(), &[rqm_s]);
        cprintf_f(false, false, 1, 7, c.dec.into(), &[xds.await1]);
        cprintf_f(c.human, false, 1, 8, c.dec.into(), &[xds.arqsz / 2.0]);
        cprintf_f(
            false,
            false,
            1,
            7,
            c.dec.into(),
            &[if ioi.rq_ticks < ioj.rq_ticks {
                0.0
            } else {
                s_value(ioj.rq_ticks as u64, ioi.rq_ticks as u64, itv) / 1000.0
            }],
        );
        cprintf_xpc(
            c.human,
            Some("XHIGH"),
            1,
            6,
            c.dec.into(),
            &[xds.util / 10.0],
        );
    } else {
        if hpart == 1 || hpart == 0 {
            cprintf_f(
                false,
                false,
                1,
                7,
                c.dec.into(),
                &[if ioi.rd_ios < ioj.rd_ios {
                    0.0
                } else {
                    s_value(ioj.rd_ios, ioi.rd_ios, itv)
                }],
            );
            cprintf_f(
                c.human,
                false,
                1,
                9,
                c.dec.into(),
                &[xios.rsectors / fctr as f64],
            );
            /* rrqm/s */
            cprintf_f(
                c.human,
                false,
                1,
                8,
                c.dec.into(),
                &[if ioi.rd_merges < ioj.rd_merges {
                    0.0
                } else {
                    s_value(ioj.rd_merges, ioi.rd_merges, itv)
                }],
            );
            cprintf_xpc(c.human, Some("XLOW0"), 1, 6, c.dec.into(), &[xios.rrqm_pc]);
            cprintf_f(false, false, 1, 7, c.dec.into(), &[xios.r_await]);
            cprintf_f(c.human, false, 1, 8, c.dec.into(), &[xios.rarqsz / 2.0]);
        }

        if hpart == 2 || hpart == 0 {
            cprintf_f(
                false,
                false,
                1,
                7,
                c.dec.into(),
                &[if ioi.wr_ios < ioj.wr_ios {
                    0.0
                } else {
                    s_value(ioj.wr_ios, ioi.wr_ios, itv)
                }],
            );
            cprintf_f(
                c.human,
                false,
                1,
                9,
                c.dec.into(),
                &[xios.wsectors / fctr as f64],
            );
            /* rrqm/s */
            cprintf_f(
                c.human,
                false,
                1,
                8,
                c.dec.into(),
                &[if ioi.wr_merges < ioj.wr_merges {
                    0.0
                } else {
                    s_value(ioj.wr_merges, ioi.wr_merges, itv)
                }],
            );
            cprintf_xpc(c.human, Some("XLOW0"), 1, 6, c.dec.into(), &[xios.wrqm_pc]);
            cprintf_f(false, false, 1, 7, c.dec.into(), &[xios.w_await]);
            cprintf_f(c.human, false, 1, 8, c.dec.into(), &[xios.warqsz / 2.0]);
        }

        if hpart == 3 || hpart == 0 {
            cprintf_f(
                false,
                false,
                1,
                7,
                c.dec.into(),
                &[if ioi.dc_ios < ioj.dc_ios {
                    0.0
                } else {
                    s_value(ioj.dc_ios, ioi.dc_ios, itv)
                }],
            );
            cprintf_f(
                c.human,
                false,
                1,
                9,
                c.dec.into(),
                &[xios.dsectors / fctr as f64],
            );
            /* rrqm/s */
            cprintf_f(
                c.human,
                false,
                1,
                8,
                c.dec.into(),
                &[if ioi.dc_merges < ioj.dc_merges {
                    0.0
                } else {
                    s_value(ioj.dc_merges, ioi.dc_merges, itv)
                }],
            );
            cprintf_xpc(c.human, Some("XLOW0"), 1, 6, c.dec.into(), &[xios.drqm_pc]);
            cprintf_f(false, false, 1, 7, c.dec.into(), &[xios.d_await]);
            cprintf_f(c.human, false, 1, 8, c.dec.into(), &[xios.darqsz / 2.0]);
        }

        if hpart == 4 || hpart == 0 {
            cprintf_f(
                false,
                false,
                1,
                7,
                c.dec.into(),
                &[if ioi.fl_ios < ioj.fl_ios {
                    0.0
                } else {
                    s_value(ioj.fl_ios, ioi.fl_ios, itv)
                }],
            );
            cprintf_f(false, false, 1, 7, c.dec.into(), &[xios.f_await]);
            cprintf_f(
                false,
                false,
                1,
                7,
                c.dec.into(),
                &[if ioi.rq_ticks < ioj.rq_ticks {
                    0.0
                } else {
                    s_value(ioj.rq_ticks as u64, ioi.rq_ticks as u64, itv) / 1000.0
                }],
            );
            cprintf_xpc(
                c.human,
                Some("XHIGH"),
                1,
                6,
                c.dec.into(),
                &[xds.util / 10.0],
            );
        }
    }

    if c.pretty {
        print!("{}", SC_ITEM_NAME);
        print!(" {}", device_name);
        print!("{}", SC_NORMAL);
    }
}

fn json_exit_print(c: &Config) -> String {
    let mut output = String::new();
    if c.json {
        output += format!("\n{}]\n", return_tab(3)).as_str();
        output += format!("{}{}", return_tab(2), RIGHT_BRACE).as_str();
        output += format!("\n{}]\n", return_tab(1)).as_str();
        output += format!("{}{}", RIGHT_BRACE, RIGHT_BRACE).as_str();
    }
    return output;
}

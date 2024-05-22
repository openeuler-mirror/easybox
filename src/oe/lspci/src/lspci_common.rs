//! This file is part of the easybox package.
//
// (c) Haopeng Liu <657407891@qq.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use clap::{crate_version, Arg, Command};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::path::Path;
use std::process;
use std::process::Command as Cmd;
use uucore::error::{UResult, UUsageError};
use uucore::format_usage;
///
pub mod lspci_header;
use crate::lspci_common::lspci_header::*;
const SYS_FS_SLOT: &str = "/sys/bus/pci/slots";
const DIR_NAME: &str = "/sys/bus/pci/devices/";
const PCI_ID_FILE1: &str = "/usr/share/hwdata/pci.ids";
const PCI_ID_FILE2: &str = "/usr/share/misc/pci.ids";
const DNS_DOMAIN: &str = "pci.id.ucw.cz";

///
pub static LSPCI_CMD_PARSE_ERROR: i32 = 1;

#[derive(Debug, Copy, Clone, PartialEq)]
///
pub struct PciSlotFilter {
    ///
    domain: Option<u32>,
    ///
    bus: Option<u32>,
    ///
    slot: Option<u32>,
    ///
    func: Option<u32>,
}
///
pub struct PciIDFilter {
    ///
    vendor: Option<u32>,
    ///
    device: Option<u32>,
    ///
    device_class: Option<u32>,
    ///
    prog_if: Option<u32>,
}

///
pub struct Config {
    ///
    pub m: u32,
    ///
    pub tree: bool,
    ///
    pub verbose: u64,
    ///
    pub kernel: bool,
    ///
    pub hex_dump: u64,
    ///
    pub bus_centric: bool,
    ///
    pub domain: bool,
    ///
    pub path: u64,
    ///
    pub number: u32,
    ///
    pub dns: u64,
    ///
    pub selection_slot: PciSlotFilter,
    ///
    pub selection_id: PciIDFilter,
    ///
    pub db_file: String,
    ///
    pub pci_ids_vendor: HashMap<(u32, Option<u32>, Option<u32>, Option<u32>), String>,
    ///
    pub pci_ids_class: HashMap<(u32, Option<u32>, Option<u32>), String>,
    ///
    pub map_mode: bool,
    ///
    pub phy_slot_map: HashMap<String, String>,
}

/// options.
///
pub mod options {
    ///
    pub static MM: &str = "machine";
    ///
    pub static TREE: &str = "tree";
    ///
    pub static VERBOSE: &str = "verbose";
    ///
    pub static KERNEL: &str = "kernel";
    ///
    pub static HEX_DUMP: &str = "hexdump";
    ///
    pub static BUS_CENTRIC: &str = "bus_centric";
    ///
    pub static DOMAIN: &str = "domain";
    ///
    pub static PATH: &str = "path";
    ///
    pub static NUMBER: &str = "number";
    ///
    pub static DNS: &str = "dns";
    ///
    pub static QDNS: &str = "qdns";
    ///
    pub static SLOT: &str = "slot";
    ///
    pub static ID: &str = "id";
    ///
    pub static DB: &str = "db";
    ///
    pub static KERNEL_FILE: &str = "kernel_file";
    ///
    pub static MAP_MODE: &str = "map_mode";
}

#[derive(Debug)]
///
pub struct PciDev {
    ///
    pub device_class: Option<u32>,
    ///
    pub class_id: Option<u32>,
    ///
    pub subclass_id: Option<u32>,
    ///
    pub prog_if: Option<u32>,
    ///
    pub vendor_id: Option<u32>,
    ///
    pub device_id: Option<u32>,
    ///
    pub revision: Option<u32>,
    ///
    pub subvendor_id: Option<u32>,
    ///
    pub subdevice_id: Option<u32>,
    ///
    pub config: Vec<u8>,
    ///
    pub config_len: u32,
    ///
    pub irq: Option<u32>,
    ///
    pub numa_node: Option<u32>,
    ///
    pub label: Option<String>,
    ///
    pub phy_slot: Option<String>,
    ///
    pub base_addr: Vec<u64>,
    ///
    pub flags: Vec<u64>,
    ///
    pub size: Vec<u64>,
}

///
pub struct Device {
    ///
    pub ppath: String,
    ///
    pub name: String,
    ///
    pub dev: PciDev,
}

///
pub struct Bridge {
    ///
    slot: PciSlotFilter,
    ///
    secondary: u32,
    ///
    subordinate: u32,
    ///
    br_dev: Device,
    ///
    pci_bus: bool,
    ///
    child: Vec<Bridge>,
}

impl Config {
    ///
    pub fn from(options: &clap::ArgMatches) -> UResult<Self> {
        let p_occurrences = options.occurrences_of(options::PATH);
        let pp = match p_occurrences {
            0 => 0,
            1 => 1,
            _ => 2,
        };
        let m_occurrences = options.occurrences_of(options::MM);
        let mm = match m_occurrences {
            0 => 0,
            1 => 1,
            _ => 2,
        };
        let v_occurrences = options.occurrences_of(options::VERBOSE);
        let v = match v_occurrences {
            0 => 0,
            1 => 1,
            2 => 2,
            _ => 3,
        };
        let n_occurrences = options.occurrences_of(options::NUMBER);
        let n = match n_occurrences {
            0 => 0,
            1 => 1,
            2 => 2,
            _ => 2,
        };
        let x_occurrences = options.occurrences_of(options::HEX_DUMP);

        let dns = {
            if options.is_present(options::QDNS) {
                3
            } else {
                options.occurrences_of(options::DNS)
            }
        };
        let slot_filter: PciSlotFilter;
        if options.is_present(options::SLOT) {
            let f_slot: String = options.value_of(options::SLOT).unwrap().to_string();
            let filter = pci_filter_parse_slot(&f_slot);
            match filter {
                Ok(s) => slot_filter = s,
                Err(e) => {
                    return Err(UUsageError::new(
                        LSPCI_CMD_PARSE_ERROR,
                        format!("-s: {}", e),
                    ));
                }
            }
        } else {
            slot_filter = PciSlotFilter {
                domain: None,
                bus: None,
                slot: None,
                func: None,
            };
        }

        let id_filter: PciIDFilter;
        if options.is_present(options::ID) {
            let f_id: String = options.value_of(options::ID).unwrap().to_string();
            let filter = pci_filter_parse_id(&f_id);
            match filter {
                Ok(s) => id_filter = s,
                Err(e) => {
                    return Err(UUsageError::new(
                        LSPCI_CMD_PARSE_ERROR,
                        format!("-d: {}", e),
                    ))
                }
            }
        } else {
            id_filter = PciIDFilter {
                vendor: None,
                device: None,
                device_class: None,
                prog_if: None,
            };
        }

        let db_file: String;
        if options.is_present(options::DB) {
            db_file = options.value_of(options::DB).unwrap().to_string();
        } else {
            let db_file_path = Path::new(PCI_ID_FILE1);

            if db_file_path.exists() {
                db_file = PCI_ID_FILE1.to_string();
            } else {
                db_file = PCI_ID_FILE2.to_string();
            }
        }

        let (pci_ids_vendor, pci_ids_class) = parse_pci_ids(db_file.as_str());
        Ok(Self {
            m: mm,
            tree: options.is_present(options::TREE),
            verbose: v,
            kernel: options.is_present(options::KERNEL),
            hex_dump: x_occurrences,
            bus_centric: options.is_present(options::BUS_CENTRIC),
            domain: options.is_present(options::DOMAIN),
            path: pp,
            number: n,
            dns: dns,
            selection_slot: slot_filter,
            selection_id: id_filter,
            db_file: db_file,
            pci_ids_vendor: pci_ids_vendor,
            pci_ids_class: pci_ids_class,
            map_mode: options.is_present(options::MAP_MODE),
            phy_slot_map: get_slots(),
        })
    }
}

///
pub fn parse_base_cmd_args(args: impl uucore::Args, about: &str, usage: &str) -> UResult<Config> {
    let command = lspci_app(about, usage);
    let arg_list = args.collect_lossy();
    Config::from(&command.try_get_matches_from(arg_list)?)
}

///
pub fn lspci_app<'a>(about: &'a str, usage: &'a str) -> Command<'a> {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about(about)
        .override_usage(format_usage(usage))
        .infer_long_args(true)
        .arg(
            Arg::new(options::MM)
                .short('m')
                .long(options::MM)
                .multiple_occurrences(true)
                .help("-mm\tProduce machine-readable output\n\
                    -m\tFor an obsolete format"),
        )
        .arg(
            Arg::new(options::TREE)
                .short('t')
                .long(options::TREE)
                .help("Show bus tree"),
        )
        .arg(
            Arg::new(options::VERBOSE)
                .short('v')
                .multiple_occurrences(true)
                .long(options::VERBOSE)
                .help("Be verbose (-vv or -vvv for higher verbosity)"),
        )
        .arg(
            Arg::new(options::KERNEL)
                .short('k')
                .long(options::KERNEL)
                .help("-k\tShow kernel drivers handling each device\n"),
        )
        .arg(
            Arg::new(options::HEX_DUMP)
                .short('x')
                .long(options::HEX_DUMP)
                .multiple_occurrences(true)
                .help("-x\tShow hex-dump of the standard part of the config space\n\
                -xxx\tShow hex-dump of the whole config space (dangerous; root only)\n\
                -xxxx\tShow hex-dump of the 4096-byte extended config space (root only)"),
        )
        .arg(
            Arg::new(options::BUS_CENTRIC)
                .short('b')
                .long(options::BUS_CENTRIC)
                .help("Bus-centric view (addresses and IRQ's as seen by the bus)"),
        )
        .arg(
            Arg::new(options::DOMAIN)
                .short('D')
                .long(options::DOMAIN)
                .help("Always show domain numbers"),
        )
        .arg(
            Arg::new(options::PATH)
                .short('P')
                .multiple_occurrences(true)
                .long(options::PATH)
                .help("-P\tDisplay bridge path in addition to bus and device number\n\
                    -PP\tDisplay bus path in addition to bus and device number"),
        )
        .arg(
            Arg::new(options::MAP_MODE)
                .short('M')
                .multiple_occurrences(true)
                .long(options::MAP_MODE)
                .help("-M\tEnable `bus mapping' mode (dangerous; root only)"),
        )
        .arg(
            Arg::new(options::NUMBER)
                .short('n')
                .long(options::NUMBER)
                .multiple_occurrences(true)
                .help("-n\tShow numeric ID's\n\
                    -nn\tShow both textual and numeric ID's (names & numbers)"),
        )
        .arg(
            Arg::new(options::DNS)
                .short('q')
                .multiple_occurrences(true)
                .long(options::DNS)
                .help("-q\tQuery the PCI ID database for unknown ID's via DNS\n\
                    -qq\tAs above, but re-query locally cached entries"),
        )
        .arg(
            Arg::new(options::QDNS)
                .short('Q')
                .long(options::QDNS)
                .help("Query the PCI ID database for all ID's via DNS"),
        )
        .arg(
            Arg::new(options::SLOT)
                .short('s')
                .long(options::SLOT)
                .value_name("SLOT")
                .takes_value(true)
                .help("Show only devices in selected slots: [[[[<domain>]:]<bus>]:][<slot>][.[<func>]]"),
        )
        .arg(
            Arg::new(options::ID)
                .short('d')
                .long(options::ID)
                .value_name("ID")
                .takes_value(true)
                .help("Show only devices with specified ID's: [<vendor>]:[<device>][:<class>]"),
        )
        .arg(
            Arg::new(options::DB)
                .short('i')
                .long(options::DB)
                .value_name("database")
                .takes_value(true)
                .help("Use specified ID database instead of /usr/share/hwdata/pci.ids"),
        )
}

/// Read the file where the device saves the physical slot
///
pub fn get_slots() -> HashMap<String, String> {
    let entries = fs::read_dir(SYS_FS_SLOT);
    let mut map = HashMap::new();
    match entries {
        Ok(entries) => {
            for entry in entries {
                let entry = entry.unwrap();
                let path = entry.path();
                if path.is_dir() {
                    let directory_name = path.file_name().and_then(|name| name.to_str()).unwrap();
                    let address_file_path = path.join("address");
                    if let Ok(mut file) = fs::File::open(address_file_path) {
                        let mut contents = String::new();
                        let _ = file.read_to_string(&mut contents);
                        map.insert(contents.trim().to_string(), directory_name.to_string());
                    }
                }
            }
        }
        Err(_) => return map,
    }
    map
}

/// Parse a hex field
///
pub fn parse_hex_field(hex_str: &str, maskp: Option<&u32>, max: u32) -> Result<u32, String> {
    let mut out: u32 = 0;
    let mut mask: u32 = !0;
    let mut bound: u32 = 0;
    let string = hex_str;

    // Trim the "0x" or "0X" prefix if it exists
    let mut iter = hex_str.chars();
    if hex_str.starts_with("0x") || hex_str.starts_with("0X") {
        iter.next();
        iter.next();
    }

    for c in string.chars() {
        let digit: u32;

        if let Some(_maskp) = maskp {
            if c == 'x' || c == 'X' {
                out = out << 4;
                bound = (bound << 4) | 1;
                mask = mask << 4;
                continue;
            }
        }

        if c.is_digit(16) {
            digit = c.to_digit(16).unwrap();
        } else {
            return Err(String::from("Invalid character in field"));
        }

        out = (out << 4) | digit;
        bound = (bound << 4) | digit;
        mask = (mask << 4) | 0xf;

        if bound > max {
            return Err(String::from("Value exceeds maximum"));
        }
    }

    Ok(out as u32)
}

/// Parse a hex string
///
fn parse_hex_string(hex_string: Option<&str>) -> Result<Option<u32>, String> {
    if !hex_string.is_some() {
        return Ok(None);
    }

    let mut stripped_string = hex_string.unwrap();

    if stripped_string.is_empty() {
        return Ok(None);
    }

    if stripped_string.starts_with("0x") || stripped_string.starts_with("0X") {
        stripped_string = &stripped_string[2..];
    }

    match u32::from_str_radix(stripped_string, 16) {
        Ok(number) => Ok(Some(number)),
        Err(_) => Err(String::from("Invalid hexadecimal string")),
    }
}

/// Split a string into n fields
///
pub fn split_to_fields(
    string: &str,
    delimiter: char,
    n: usize,
) -> Result<Vec<Option<&str>>, String> {
    let parts_str: Vec<&str> = string.split(delimiter).collect();
    let parts: Vec<Option<&str>> = parts_str.into_iter().map(Some).collect();
    if parts.len() < n {
        let mut result = parts.clone();
        result.resize(n, None);
        Ok(result)
    } else if parts.len() == n {
        Ok(parts)
    } else {
        Err(String::from("Too many fields"))
    }
}

/// Parse a slot to a PciSlotFilter
///
pub fn pci_filter_parse_slot(slots: &str) -> Result<PciSlotFilter, String> {
    let mut domain: Option<u32> = None;
    let mut bus: Option<u32> = None;
    let mut slot: Option<u32> = None;
    let mut func: Option<u32> = None;
    let fields = split_to_fields(slots, ':', 3)?;

    let mut i = 0;
    if fields[2].is_some() {
        let domain_field = parse_hex_string(fields[0]);
        match domain_field {
            Ok(domain_field) => domain = domain_field,
            Err(_err) => return Err(String::from("Invalid domain number")),
        }
        i += 1;
    }
    if fields[i + 1].is_some() {
        let bus_field = parse_hex_string(fields[i]);
        match bus_field {
            Ok(bus_field) => bus = bus_field,
            Err(_err) => return Err(String::from("Invalid bus number")),
        }
        i += 1;
    }
    if fields[i].is_some() {
        let sfields = split_to_fields(fields[i].unwrap(), '.', 2);
        match sfields {
            Ok(sfields) => {
                let slot_field = parse_hex_string(sfields[0]);
                match slot_field {
                    Ok(slot_field) => slot = slot_field,
                    Err(_err) => return Err(String::from("Invalid slot number")),
                }
                let func_field = parse_hex_string(sfields[1]);
                match func_field {
                    Ok(func_field) => func = func_field,
                    Err(_err) => return Err(String::from("Invalid function number")),
                }
            }
            Err(_err) => return Err(String::from("Invalid slot/function number")),
        }
    }

    Ok(PciSlotFilter {
        domain,
        bus,
        slot,
        func,
    })
}

/// Parse an ID to a PciIDFilter
///
pub fn pci_filter_parse_id(ids: &str) -> Result<PciIDFilter, String> {
    let vendor: Option<u32>;
    let device: Option<u32>;
    let device_class: Option<u32>;
    let prog_if: Option<u32>;

    if !ids.contains(':') {
        return Err(String::from("At least two fields must be given"));
    }

    let fields = split_to_fields(ids, ':', 4)?;

    let vendor_field = parse_hex_string(fields[0]);
    match vendor_field {
        Ok(vendor_field) => vendor = vendor_field,
        Err(_err) => return Err(String::from("Invalid vendor ID")),
    }

    let device_field = parse_hex_string(fields[1]);
    match device_field {
        Ok(device_field) => device = device_field,
        Err(_err) => return Err(String::from("Invalid device ID")),
    }

    let device_class_field = parse_hex_string(fields[2]);
    match device_class_field {
        Ok(device_class_field) => device_class = device_class_field,
        Err(_err) => return Err(String::from("Invalid class code")),
    }

    let prog_if_field = parse_hex_string(fields[3]);
    match prog_if_field {
        Ok(prog_if_field) => prog_if = prog_if_field,
        Err(_err) => return Err(String::from("Invalid programming interface code")),
    }

    Ok(PciIDFilter {
        vendor,
        device,
        device_class,
        prog_if,
    })
}

/// Filter a device
///
pub fn device_filter(d: &Device, c: &Config, id_filter: bool) -> bool {
    let domain = pci_filter_parse_slot(d.name.as_str());
    let slot_filter = &c.selection_slot;
    match domain {
        Ok(domain) => {
            let filter_mismatch = |opt_filter: Option<u32>, domain_value: Option<u32>| {
                opt_filter.map_or(false, |filter_value| filter_value != domain_value.unwrap())
            };

            if filter_mismatch(slot_filter.domain, domain.domain)
                || filter_mismatch(slot_filter.bus, domain.bus)
                || filter_mismatch(slot_filter.slot, domain.slot)
                || filter_mismatch(slot_filter.func, domain.func)
            {
                return false;
            }
        }
        Err(_err) => return false,
    }

    if !id_filter {
        return true;
    }
    let filter = &c.selection_id;

    let filter_mismatch = |opt_filter: Option<u32>, dev_value| {
        opt_filter.map_or(false, |filter_value| filter_value != dev_value)
    };

    if filter_mismatch(filter.vendor, d.dev.vendor_id.unwrap())
        || filter_mismatch(filter.device, d.dev.device_id.unwrap())
        || (filter.device_class.is_some() && {
            let dev_device: u32 = d.dev.class_id.unwrap_or(0) * 0x100 + d.dev.subclass_id.unwrap();
            filter.device_class.unwrap() != dev_device
        })
        || filter_mismatch(filter.prog_if, d.dev.prog_if.unwrap())
    {
        return false;
    }

    true
}

/// Read a directory
///
pub fn read_dir(path: &str) -> Vec<String> {
    std::fs::read_dir(path)
        .unwrap()
        .map(|f| f.unwrap().file_name().to_str().unwrap().to_string())
        .collect()
}

/// Read a file
///
pub fn read_file(path: &str) -> Result<String, std::io::Error> {
    match std::fs::read_to_string(path) {
        Ok(content) => Ok(content),
        Err(error) => Err(error),
    }
}

/// Parse the pci.ids file
///
fn parse_pci_ids(
    file_path: &str,
) -> (
    HashMap<(u32, Option<u32>, Option<u32>, Option<u32>), String>,
    HashMap<(u32, Option<u32>, Option<u32>), String>,
) {
    let mut devices_vendor: HashMap<(u32, Option<u32>, Option<u32>, Option<u32>), String> =
        HashMap::new();
    let mut current_vendor: Option<u32> = None;
    let mut current_device: Option<u32> = None;
    let mut current_subvendor: Option<u32>;
    let mut current_subdevice: Option<u32>;
    let mut process_class = false;

    let mut devices_class: HashMap<(u32, Option<u32>, Option<u32>), String> = HashMap::new();
    let mut current_class: Option<u32> = None;
    let mut current_subclass: Option<u32> = None;
    let mut current_progif: Option<u32>;

    let db_file_path = Path::new(file_path);

    if !db_file_path.exists() {
        return (devices_vendor, devices_class);
    }
    let mut file = File::open(file_path).expect("Failed to open file");
    let mut contents = String::new();
    let _ = file.read_to_string(&mut contents);
    let lines: Vec<&str> = contents.split('\n').collect();

    for line in lines {
        let trimmed_line = line.trim();
        if trimmed_line == "# List of known device classes, subclasses and programming interfaces" {
            process_class = true;
            continue;
        }

        if trimmed_line.is_empty() || trimmed_line.starts_with('#') {
            continue;
        }
        if !process_class {
            let parts: Vec<&str> = trimmed_line.split_whitespace().collect();
            let num_tabs = line.chars().take_while(|&c| c == '\t').count();
            match num_tabs {
                0 => {
                    current_vendor = Some(u32::from_str_radix(parts[0], 16).unwrap());
                    current_device = None;
                    current_subvendor = None;
                    current_subdevice = None;
                    devices_vendor.insert(
                        (
                            current_vendor.unwrap(),
                            current_device,
                            current_subvendor,
                            current_subdevice,
                        ),
                        parts[1..].join(" ").to_string(),
                    );
                }
                1 => {
                    current_device = Some(u32::from_str_radix(parts[0], 16).unwrap());
                    current_subvendor = None;
                    current_subdevice = None;
                    if let Some(vendor) = current_vendor {
                        devices_vendor.insert(
                            (vendor, current_device, current_subvendor, current_subdevice),
                            parts[1..].join(" ").to_string(),
                        );
                    }
                }
                2 => {
                    current_subvendor = Some(u32::from_str_radix(parts[0], 16).unwrap());
                    current_subdevice = Some(u32::from_str_radix(parts[1], 16).unwrap());
                    if let (Some(vendor), Some(device), Some(subvendor)) =
                        (current_vendor, current_device, current_subvendor)
                    {
                        devices_vendor.insert(
                            (vendor, Some(device), Some(subvendor), current_subdevice),
                            parts[2..].join(" ").to_string(),
                        );
                    }
                }
                _ => {}
            }
        } else {
            let parts: Vec<&str> = trimmed_line.split_whitespace().collect();
            let num_tabs = line.chars().take_while(|&c| c == '\t').count();
            match num_tabs {
                0 => {
                    current_class = Some(u32::from_str_radix(parts[1], 16).unwrap());
                    current_subclass = None;
                    current_progif = None;
                    devices_class.insert(
                        (current_class.unwrap(), current_subclass, current_progif),
                        parts[2..].join(" ").to_string(),
                    );
                }
                1 => {
                    current_subclass = Some(u32::from_str_radix(parts[0], 16).unwrap());
                    current_progif = None;
                    if let Some(class) = current_class {
                        devices_class.insert(
                            (class, current_subclass, current_progif),
                            parts[1..].join(" ").to_string(),
                        );
                    }
                }
                2 => {
                    current_progif = Some(u32::from_str_radix(parts[0], 16).unwrap());
                    if let (Some(class), Some(device), Some(subclass)) =
                        (current_class, current_subclass, current_progif)
                    {
                        devices_class.insert(
                            (class, Some(device), Some(subclass)),
                            parts[1..].join(" ").to_string(),
                        );
                    }
                }
                _ => {}
            }
        }
    }
    (devices_vendor, devices_class)
}

/// Read a file and returns its content as u32
///
pub fn read_u32_from_file(path: &str) -> Option<u32> {
    match read_file(path) {
        Ok(content) => {
            if content.len() > 2 && content.starts_with("0x") {
                let trimmed_content = &content[..content.len() - 1];

                if trimmed_content.len() > 2 {
                    let hex_digits = &trimmed_content[2..];
                    return u32::from_str_radix(hex_digits, 16).ok();
                }
            }
            None
        }
        Err(_) => None,
    }
}

/// Read the device file and returns a Device
///
pub fn read_device(df: &str, c: &Config) -> Device {
    let path = format!("{}/{}", DIR_NAME, df);
    let class_id = read_u32_from_file(format!("{}/{}", path, "class").as_str());
    let class_id_str = format!("{:06X}", class_id.unwrap());
    let mut bytes = [0u8; 4096];
    let mut file = File::open(format!("{}/{}", path, "config").as_str()).unwrap();
    let bytes_read = file.read(&mut bytes).unwrap();

    let irq;
    match read_file(format!("{}/{}", path, "irq").as_str()) {
        Ok(content) => {
            irq = Some(u32::from_str_radix(&content[..content.len() - 1], 10).unwrap());
        }
        Err(_) => irq = None,
    }
    let label;
    match read_file(format!("{}/{}", path, "label").as_str()) {
        Ok(content) => {
            label = Some(content[..content.len() - 1].to_string());
        }
        Err(_) => label = None,
    }
    let phy_slot;
    let address = df[0..df.len() - 2].to_string();
    if c.phy_slot_map.contains_key(&address) {
        let value = c.phy_slot_map.get(&address).unwrap();
        phy_slot = value.to_string();
    } else {
        phy_slot = String::new();
    }

    let mut base_addr = vec![0; 6];
    let mut flags = vec![0; 6];
    let mut size = vec![0; 6];
    let mut i = 0;
    if let Ok(file) = File::open(format!("{}/{}", path, "resource").as_str()) {
        let reader = BufReader::new(file);
        for line in reader.lines() {
            if let Ok(line) = line {
                let values: Vec<&str> = line.split_whitespace().collect();
                let mut start = 0;
                let mut end = 0;
                let mut flag = 0;
                if values.len() >= 3 {
                    if let Ok(number) = u64::from_str_radix(&values[0][2..], 16) {
                        start = number;
                    }
                    if let Ok(number) = u64::from_str_radix(&values[1][2..], 16) {
                        end = number;
                    }
                    if let Ok(number) = u64::from_str_radix(&values[2][2..], 16) {
                        flag = number;
                    }
                }
                let flag_value;
                let start_value;
                if end < start {
                    start_value = 0;
                } else if end != 0 {
                    start_value = end - start + 1;
                } else {
                    start_value = 0;
                }
                if i < 6 {
                    flags[i] = flag;
                    flag_value = flag & PCI_ADDR_FLAG_MASK;
                    base_addr[i] = start | flag_value;
                    size[i] = start_value;
                }
            }
            i = i + 1;
        }
    }
    let pci_dev = PciDev {
        device_class: Some(u32::from_str_radix(&class_id_str[0..4], 16).unwrap()),
        class_id: Some(u32::from_str_radix(&class_id_str[0..2], 16).unwrap()),
        subclass_id: Some(u32::from_str_radix(&class_id_str[2..4], 16).unwrap()),
        prog_if: Some(u32::from_str_radix(&class_id_str[4..6], 16).unwrap()),
        vendor_id: read_u32_from_file(format!("{}/{}", path, "vendor").as_str()),
        device_id: read_u32_from_file(format!("{}/{}", path, "device").as_str()),
        subvendor_id: read_u32_from_file(format!("{}/{}", path, "subsystem_vendor").as_str()),
        subdevice_id: read_u32_from_file(format!("{}/{}", path, "subsystem_device").as_str()),
        revision: read_u32_from_file(format!("{}/{}", path, "revision").as_str()),
        config: bytes[0..bytes_read].to_vec(),
        config_len: bytes_read as u32,
        irq: irq,
        numa_node: read_u32_from_file(format!("{}/{}", path, "numa_node").as_str()),
        label: label,
        phy_slot: Some(String::from(phy_slot)),
        base_addr: base_addr,
        flags: flags,
        size: size,
    };
    let d = Device {
        ppath: String::from(DIR_NAME),
        name: String::from(df),
        dev: pci_dev,
    };

    d
}

/// Scan the /sys/bus/pci/devices directory
///
pub fn read_devices(c: &Config) -> Vec<Device> {
    let mut devices = read_dir(DIR_NAME);
    devices.sort();
    let mut devices_vec: Vec<Device> = Vec::new();
    for device_file in devices {
        devices_vec.push(read_device(&device_file, c));
    }
    devices_vec
}

/// Query the DNS server
///
fn run_dig(dnsname: &str) -> String {
    let output = Cmd::new("dig")
        .arg(dnsname)
        .arg("TXT")
        .arg("+short")
        .output()
        .unwrap();

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        if let Some(start_index) = stdout.find('=') {
            let value = &stdout[start_index + 1..&stdout.len() - 2];
            return String::from(value);
        }
        return String::new();
    } else {
        return String::new();
    }
}

/// Query the PCI ID database via DNS
///
pub fn id_lookup_net(
    name: &str,
    id1: u32,
    id2: Option<u32>,
    id3: Option<u32>,
    id4: Option<u32>,
) -> String {
    let dns_name;
    match name {
        "vendor" => {
            dns_name = format!("{:0>4x}", id1);
        }
        "device" => {
            dns_name = format!("{:0>4x}.{:0>4x}", id2.unwrap_or(0), id1);
        }
        "class" => {
            dns_name = format!("{:0>2x}.c", id1);
        }
        "subclass" => {
            dns_name = format!("{:0>2x}.{:0>2x}.c", id2.unwrap_or(0), id1);
        }
        "subsystem" => {
            dns_name = format!(
                "{:0>4x}.{:0>4x}.{:0>4x}.{:0>4x}",
                id4.unwrap_or(0),
                id3.unwrap_or(0),
                id2.unwrap_or(0),
                id1
            );
        }
        "gen_subsys" => {
            dns_name = format!("{:0>2x}.{:0>2x}.s", id2.unwrap_or(0), id1);
        }
        "prog_if" => {
            dns_name = format!(
                "{:0>2x}.{:0>2x}.{:0>2x}.c",
                id3.unwrap_or(0),
                id2.unwrap_or(0),
                id1
            );
        }
        _ => {
            return String::new();
        }
    }
    let dnsquery = format!("{}.{}", dns_name, DNS_DOMAIN);
    let dnsresult = run_dig(&dnsquery);
    if dnsresult == String::new() && name == "subclass" {
        return format!(
            "{} [{:02x}{:02x}]",
            id_lookup_net("class", id1, id2, id3, id4),
            id1,
            id2.unwrap()
        );
    }
    return dnsresult;
}

/// Choose the method to query the database based on config
///
pub fn id_lookup(
    c: &Config,
    name: &str,
    id1: u32,
    id2: Option<u32>,
    id3: Option<u32>,
    id4: Option<u32>,
) -> String {
    if c.dns == 3 {
        return id_lookup_net(name, id1, id2, id3, id4);
    }
    let ret = pci_id_lookup(c, name, id1, id2, id3, id4);
    if ret == String::new() {
        if c.dns == 1 {
            return id_lookup_net(name, id1, id2, id3, id4);
        } else if c.dns == 2 {
            return pci_id_lookup(c, name, id1, id2, id3, id4);
        }
    }
    return ret;
}

/// Query the PCI ID database
///
pub fn pci_id_lookup(
    c: &Config,
    name: &str,
    id1: u32,
    id2: Option<u32>,
    id3: Option<u32>,
    id4: Option<u32>,
) -> String {
    let matchname;
    if name == "subsystem" || name == "gen_subsys" {
        matchname = "device";
    } else if name == "subclass" {
        matchname = "class";
    } else {
        matchname = name;
    }
    match matchname {
        "vendor" => {
            let vendor_name = String::from(
                c.pci_ids_vendor
                    .get(&(id1, id2, id3, id4))
                    .unwrap_or(&String::new()),
            );
            return vendor_name;
        }
        "class" => {
            if let Some(class_name) = c.pci_ids_class.get(&(id1, id2, None)) {
                return class_name.to_string();
            } else {
                if let Some(class_name) = c.pci_ids_class.get(&(id1, None, None)) {
                    if c.number == 2 {
                        return class_name.to_string();
                    }
                    return format!("{} [{:02x}{:02x}]", class_name, id1, id2.unwrap());
                }
            }
            return String::new();
        }
        "device" => {
            let device_name = String::from(
                c.pci_ids_vendor
                    .get(&(id1, id2, id3, id4))
                    .unwrap_or(&String::new()),
            );
            return device_name;
        }
        "progif" => {
            let prog_if = String::from(
                c.pci_ids_class
                    .get(&(id1, id2, id3))
                    .unwrap_or(&String::new()),
            );
            return prog_if;
        }
        _ => {
            return String::new();
        }
    }
}

/// Query subsystem
///
pub fn id_lookup_subsys(
    c: &Config,
    name: &str,
    id1: u32,
    id2: Option<u32>,
    id3: Option<u32>,
    id4: Option<u32>,
) -> String {
    let d = id_lookup(c, "subsystem", id1, id2, id3, id4);
    if d.is_empty() {
        return id_lookup(c, "gen_subsys", id3.unwrap(), id4, None, None);
    }
    if id3.unwrap() == id1 && id4 == id2 {
        return id_lookup(c, name, id1, id2, None, None);
    }
    return d;
}

/// Format the output
///
pub fn pci_format(c: &Config, name: String, id: String, unknown: &str) -> String {
    if c.number == 1 {
        return format!("{}", id);
    }
    if c.number == 2 {
        if name.is_empty() {
            return format!("{} [{}]", unknown, id);
        }
        return format!("{} [{}]", name, id);
    }
    if name.is_empty() {
        return format!("{} {}", unknown, id);
    }
    return name;
}

/// Format the output pair
///
pub fn pci_format_pair(c: &Config, v: String, d: String, id: String) -> String {
    if c.number == 1 {
        return format!("{}", id);
    }
    if c.number == 2 {
        if v.is_empty() {
            return format!("Device [{}]", id);
        }
        if d.is_empty() {
            return format!("{} Device [{}]", v, id);
        }
        return format!("{} {} [{}]", v, d, id);
    }
    if v.is_empty() {
        return format!("Device {}", id);
    }
    if d.is_empty() {
        return format!("{} Device {}", v, id[5..].to_string());
    }
    return format!("{} {}", v, d);
}

/// Lookup the name
///
pub fn pci_lookup_name(d: &Device, name: &str, c: &Config) -> String {
    let p = &d.dev;
    match name {
        "class" => {
            return pci_format(
                c,
                id_lookup(
                    c,
                    "subclass",
                    p.class_id.unwrap(),
                    p.subclass_id,
                    None,
                    None,
                ),
                format!("{:02x}{:02x}", p.class_id.unwrap(), p.subclass_id.unwrap()),
                "Class",
            );
        }
        "vendor" => {
            return pci_format(
                c,
                id_lookup(c, name, p.vendor_id.unwrap(), None, None, None),
                format!("{:04x}", p.vendor_id.unwrap()),
                "Vendor",
            );
        }
        "device" => {
            return pci_format(
                c,
                id_lookup(c, name, p.vendor_id.unwrap(), p.device_id, None, None),
                format!("{:04x}", p.device_id.unwrap()),
                "Device",
            );
        }
        "vendor_device" => {
            let vendor = id_lookup(c, "vendor", p.vendor_id.unwrap(), None, None, None);
            let device = id_lookup(c, "device", p.vendor_id.unwrap(), p.device_id, None, None);
            return pci_format_pair(
                c,
                vendor,
                device,
                format!("{:04x}:{:04x}", p.vendor_id.unwrap(), p.device_id.unwrap()),
            );
        }
        "subsystem_vendor" => {
            return pci_format(
                c,
                id_lookup(c, "vendor", p.subvendor_id.unwrap(), None, None, None),
                format!("{:04x}", p.subvendor_id.unwrap()),
                "Unknown Vendor",
            );
        }
        "subsystem_device" => {
            return pci_format(
                c,
                id_lookup_subsys(
                    c,
                    "device",
                    p.vendor_id.unwrap(),
                    p.device_id,
                    p.subvendor_id,
                    p.subdevice_id,
                ),
                format!("{:04x}", p.subdevice_id.unwrap()),
                "Device",
            );
        }
        "subsystem" => {
            let v = id_lookup(c, "vendor", p.subvendor_id.unwrap(), None, None, None);
            let d = id_lookup_subsys(
                c,
                "device",
                p.vendor_id.unwrap(),
                p.device_id,
                p.subvendor_id,
                p.subdevice_id,
            );
            return pci_format_pair(
                c,
                v,
                d,
                format!(
                    "{:04x}:{:04x}",
                    p.subvendor_id.unwrap(),
                    p.subdevice_id.unwrap()
                ),
            );
        }
        "progif" => {
            let prog_if = id_lookup(
                c,
                "progif",
                p.class_id.unwrap(),
                p.subclass_id,
                p.prog_if,
                None,
            );
            return prog_if;
        }
        _ => {
            return String::new();
        }
    }
}

/// Show the slot path
///
pub fn show_slot_path(d: &Device, c: &Config) {
    if c.path == 0 {
        let parts: Vec<&str> = d.name.splitn(4, |c| c == ':' || c == '.').collect();
        print!("{:02}:{:02}.{:01}", parts[1], parts[2], parts[3]);
        return;
    }
    let filename = format!("{}/{}", DIR_NAME, d.name);
    if let Ok(target_path) = fs::read_link(filename) {
        let path = target_path.to_str().unwrap();
        let keyword = "devices/pci";
        let remain: &str;
        if let Some(index) = path.find(keyword) {
            remain = &path[(index + keyword.len())..];
        } else {
            return;
        }
        let mut parts: Vec<&str> = remain.split('/').collect();
        parts.remove(0);
        for i in 0..parts.len() {
            if i > 0 {
                print!("/");
            }
            let cp: Vec<&str> = parts[i].splitn(4, |c| c == ':' || c == '.').collect();
            if i == parts.len() - 1 && parts.len() > 1 {
                if c.path > 1 {
                    print!("{:02}:{:02}.{:01}", cp[1], cp[2], cp[3]);
                } else {
                    print!("{:02}.{:01}", cp[2], cp[3]);
                }
                continue;
            }
            print!("{:02}:{:02}.{:01}", cp[1], cp[2], cp[3]);
        }
    }
}

/// Show the slot name
///
pub fn show_slot_name(d: &Device, c: &Config) {
    let parts: Vec<&str> = d.name.splitn(4, |c| c == ':' || c == '.').collect();
    if c.domain {
        print!("{}:", parts[0]);
    }
    show_slot_path(d, c)
}

fn print_shell_escaped(c: &str) {
    print!(" \"");
    for ch in c.chars() {
        match ch {
            '"' | '\\' => print!("\\{}", ch),
            _ => print!("{}", ch),
        }
    }
    print!("\"");
}

/// Output devices in terse mode
///
pub fn show_terse(d: &Device, c: &Config) {
    show_slot_name(d, c);
    print!(
        " {}: {}",
        pci_lookup_name(d, "class", c),
        pci_lookup_name(d, "vendor_device", c),
    );

    if d.dev.revision.unwrap() != 0 {
        print!(" (rev {:02})", d.dev.revision.unwrap());
    }
    if c.verbose > 0 {
        let pr = d.dev.prog_if.unwrap();
        let x = pci_lookup_name(d, "progif", c).to_string();
        if pr > 0 || x.len() > 0 {
            print!(" (prog-if {:02x}", pr);
            if x.len() > 0 {
                print!(" [{}]", x);
            }
            print!(")");
        }
    }
    println!();
    if c.verbose > 0 || c.kernel {
        if d.dev.label.is_some() {
            let label = d.dev.label.as_ref().unwrap();
            print!("\tDeviceName: {}", label);
        }
        if d.dev.subvendor_id.is_some()
            && d.dev.subvendor_id.unwrap() != 0
            && d.dev.subvendor_id.unwrap() != 0xffff
        {
            println!("\tSubsystem: {}", pci_lookup_name(d, "subsystem", c));
        }
    }
}

/// Output devices in machine mode
///
pub fn show_machine(d: &Device, c: &Config) {
    show_slot_name(d, c);
    print_shell_escaped(&pci_lookup_name(d, "class", c));
    print_shell_escaped(&pci_lookup_name(d, "vendor", c));
    print_shell_escaped(&pci_lookup_name(d, "device", c));
    if d.dev.revision.unwrap() != 0 {
        print!(" -r{:02x}", d.dev.revision.unwrap());
    }
    print!(" -p{:02x}", d.dev.prog_if.unwrap());

    if d.dev.subvendor_id.is_some()
        && d.dev.subvendor_id.unwrap() != 0
        && d.dev.subvendor_id.unwrap() != 0xffff
    {
        print_shell_escaped(&pci_lookup_name(d, "subsystem_vendor", c));
        print_shell_escaped(&pci_lookup_name(d, "subsystem_device", c));
    } else {
        print_shell_escaped("");
        print_shell_escaped("");
    }
    println!();
}

/// Check if the access to configuration space is valid
///
pub fn check_conf_range(d: &Device, pos: usize, len: usize) -> bool {
    if pos + len > d.dev.config_len as usize {
        eprintln!(
            "Internal bug: Accessing non-read configuration byte at position {:x}",
            pos
        );
        process::exit(0x0100);
    }
    return true;
}

/// Get a byte from configuration space
///
pub fn get_conf_byte(d: &Device, pos: usize) -> u8 {
    check_conf_range(d, pos, 1);
    return d.dev.config[pos];
}

/// Get a word from configuration space
///
pub fn get_conf_word(d: &Device, pos: usize) -> u16 {
    check_conf_range(d, pos, 2);
    return d.dev.config[pos] as u16 | ((d.dev.config[pos + 1] as u16) << 8) as u16;
}

fn get_conf_long(d: &Device, pos: usize) -> u32 {
    check_conf_range(d, pos, 4);
    (d.dev.config[pos as usize] as u32)
        | ((d.dev.config[(pos + 1) as usize] as u32) << 8)
        | ((d.dev.config[(pos + 2) as usize] as u32) << 16)
        | ((d.dev.config[(pos + 3) as usize] as u32) << 24)
}

///
pub fn show_hex_dump(d: &Device, c: &Config) {
    if d.dev.config.len() == 0 {
        println!("WARNING: Cannot show hex-dump of the config space");
    }
    let mut cnt = d.dev.config_len;

    if c.hex_dump >= 3 && d.dev.config_len >= 256 {
        cnt = 256;
        if c.hex_dump >= 4 && d.dev.config_len >= 4096 {
            cnt = 4096;
        }
    }

    for i in 0..cnt {
        if i & 15 == 0 {
            print!("{:02x}:", i);
        }
        print!(" {:02x}", get_conf_byte(d, i as usize));
        if i & 15 == 15 {
            println!();
        }
    }
}

/// Get the name of the kernel driver
///
pub fn show_kernel(d: &Device) {
    let driver = format!("{}/{}/driver", DIR_NAME, d.name);

    if let Ok(target_path) = fs::read_link(driver.clone()) {
        println!(
            "\tKernel driver in use: {}",
            Path::new(&target_path)
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
        );
    }
    let modules = format!("{}/module", driver);
    if let Ok(target_path) = fs::read_link(modules) {
        println!(
            "\tKernel modules: {}",
            Path::new(&target_path)
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
        );
    }
}

/// Get the status of a PCI device
///
pub fn pci_flag(x: u16, y: u16) -> char {
    if x & y == 0 {
        return '-';
    }
    return '+';
}

/// Show the base address of a PCI device
///
pub fn show_bases(d: &Device, cnt: usize, c: &Config) {
    let cmd = get_conf_word(d, PCI_COMMAND as usize);
    for i in 0..cnt {
        let pos = d.dev.base_addr[i];
        let len = d.dev.size[i];
        let ioflg = d.dev.flags[i];
        let mut flg = get_conf_long(d, PCI_BASE_ADDRESS_0 as usize + 4 * i) as u64;
        if flg == 0xffffffff {
            flg = 0;
        }
        if pos == 0 && len == 0 {
            continue;
        }
        print!("\t");
        if c.verbose > 1 {
            print!("Region {}: ", i);
        }
        let hw_lower = get_conf_long(d, PCI_BASE_ADDRESS_0 as usize + 4 * i);
        if flg & PCI_BASE_ADDRESS_SPACE_IO as u64 != 0 {
            let a = pos & PCI_BASE_ADDRESS_IO_MASK as u64;
            print!("I/O ports at ");
            if a != 0 || (cmd & PCI_COMMAND_IO as u16) != 0 {
                print!("{:0>4x}", a);
            } else if hw_lower != 0 {
                print!("<ignored>");
            } else {
                print!("<unassigned>");
            }
        } else {
            let t = flg & PCI_BASE_ADDRESS_MEM_TYPE_MASK as u64;
            let a = pos & PCI_ADDR_MEM_MASK as u64;
            print!("Memory at ");
            if a != 0 {
                print!("{:0>4x}", a);
            } else if hw_lower != 0 {
                print!("<ignored>");
            } else {
                print!("<unassigned>");
            }
            let typ;
            if t == PCI_BASE_ADDRESS_MEM_TYPE_32 as u64 {
                typ = "32-bit";
            } else if t == PCI_BASE_ADDRESS_MEM_TYPE_64 as u64 {
                typ = "64-bit";
            } else if t == PCI_BASE_ADDRESS_MEM_TYPE_1M as u64 {
                typ = "low-1m";
            } else {
                typ = "type 3";
            }
            let typ2;
            if flg & PCI_BASE_ADDRESS_MEM_PREFETCH as u64 != 0 {
                typ2 = "";
            } else {
                typ2 = "non-";
            }
            print!(" ({}, {}prefetchable)", typ, typ2);
            if cmd & PCI_COMMAND_MEMORY as u16 == 0 {
                print!(" [disabled]");
            }
        }
        if (ioflg & PCI_IORESOURCE_PCI_EA_BEI as u64) != 0 {
            print!(" [enhanced]");
        }
        show_size(len);
        println!();
    }
}

/// Show the capabilities of a PCI device
///
pub fn show_size(mut x: u64) {
    static SUFFIX: [&str; 5] = ["", "K", "M", "G", "T"];
    if x == 0 {
        return;
    }
    let mut idx = 0;
    for _i in 0..5 {
        if x % 1024 != 0 {
            break;
        }
        x /= 1024;
        idx += 1;
    }
    print!(" [size={}{}]", x, SUFFIX[idx]);
}

///
pub fn show_htype0(d: &Device, c: &Config) {
    show_bases(d, 6, c);
    show_caps(d, PCI_CAPABILITY_LIST as usize);
}
///
pub fn show_htype1(d: &Device, c: &Config) {
    show_bases(d, 2, c);
    show_caps(d, PCI_CAPABILITY_LIST as usize);
}
///
pub fn show_htype2(d: &Device, c: &Config) {
    show_bases(d, 1, c);
    show_caps(d, PCI_CAPABILITY_LIST as usize);
}

///
pub fn show_htype_unknown(d: &Device, c: &Config) {
    if c.bus_centric {
        return;
    }
    show_bases(d, 6, c);
}

///
pub fn show_caps(d: &Device, mut wh: usize) {
    if get_conf_word(d, PCI_STATUS as usize) & PCI_STATUS_CAP_LIST as u16 != 0 {
        wh = get_conf_byte(d, wh as usize) as usize & !3;
        while wh != 0 {
            print!("\tCapabilities: ");
            if wh >= 64 {
                print!("<access denied>\n");
                break;
            } else {
                print!("[{:0>2x}] \n", wh);
            }
        }
    }
}

/// Output devices in verbose mode
///
pub fn show_verbose(d: &Device, c: &Config) {
    let htype = get_conf_byte(d, PCI_HEADER_TYPE) & 0x7f;
    let class = d.dev.device_class.unwrap();
    let bist;
    let max_lat;
    let min_gnt;
    show_terse(d, c);

    match htype {
        PCI_HEADER_TYPE_NORMAL => {
            if class == PCI_CLASS_BRIDGE_PCI as u32 {
                println!(
                    "\t!!! Invalid class {:04x} for header type {:2x}",
                    class, htype
                );
            }
            bist = get_conf_byte(d, PCI_BIST);
            max_lat = get_conf_byte(d, PCI_MAX_LAT);
            min_gnt = get_conf_byte(d, PCI_MAX_LAT);
        }
        PCI_HEADER_TYPE_BRIDGE => {
            if (class >> 8) != PCI_BASE_CLASS_BRIDGE as u32 {
                println!(
                    "\t!!! Invalid class {:04x} for header type {:2x}",
                    class, htype
                );
            }
            bist = get_conf_byte(d, PCI_BIST);
            max_lat = 0;
            min_gnt = 0;
        }
        PCI_HEADER_TYPE_CARDBUS => {
            if (class >> 8) != PCI_BASE_CLASS_BRIDGE as u32 {
                println!(
                    "\t!!! Invalid class {:04x} for header type {:2x}",
                    class, htype
                );
            }
            bist = get_conf_byte(d, PCI_BIST);
            max_lat = 0;
            min_gnt = 0;
        }
        _ => {
            bist = 0;
            max_lat = 0;
            min_gnt = 0;
        }
    }

    if c.verbose > 1 {
        let cmd = get_conf_word(d, PCI_COMMAND as usize);
        let status = get_conf_word(d, PCI_STATUS as usize);
        println!("\tControl: I/O{} Mem{} BusMaster{} SpecCycle{} MemWINV{} VGASnoop{} ParErr{} Stepping{} SERR{} FastB2B{} DisINTx{}",
            pci_flag(cmd, PCI_COMMAND_IO as u16),
            pci_flag(cmd, PCI_COMMAND_MEMORY as u16),
            pci_flag(cmd, PCI_COMMAND_MASTER as u16),
            pci_flag(cmd, PCI_COMMAND_SPECIAL as u16),
            pci_flag(cmd, PCI_COMMAND_INVALIDATE as u16),
            pci_flag(cmd, PCI_COMMAND_VGA_PALETTE as u16),
            pci_flag(cmd, PCI_COMMAND_PARITY as u16),
            pci_flag(cmd, PCI_COMMAND_WAIT as u16),
            pci_flag(cmd, PCI_COMMAND_SERR as u16),
            pci_flag(cmd, PCI_COMMAND_FAST_BACK as u16),
            pci_flag(cmd, PCI_COMMAND_DISABLE_INTX as u16));
        let speed = status & PCI_STATUS_DEVSEL_MASK;
        let speed_char;
        match speed {
            PCI_STATUS_DEVSEL_SLOW => speed_char = "slow",
            PCI_STATUS_DEVSEL_MEDIUM => speed_char = "medium",
            PCI_STATUS_DEVSEL_FAST => speed_char = "fast",
            _ => speed_char = "??",
        }
        println!("\tStatus: Cap{} 66MHz{} UDF{} FastB2B{} ParErr{} DEVSEL={} >TAbort{} <TAbort{} <MAbort{} >SERR{} <PERR{} INTx{}",
            pci_flag(status, PCI_STATUS_CAP_LIST as u16),
            pci_flag(status, PCI_STATUS_66MHZ as u16),
            pci_flag(status, PCI_STATUS_UDF as u16),
            pci_flag(status, PCI_STATUS_FAST_BACK as u16),
            pci_flag(status, PCI_STATUS_PARITY as u16),
            speed_char,
            pci_flag(status, PCI_STATUS_SIG_TARGET_ABORT as u16),
            pci_flag(status, PCI_STATUS_REC_TARGET_ABORT as u16),
            pci_flag(status, PCI_STATUS_REC_MASTER_ABORT as u16),
            pci_flag(status, PCI_STATUS_SIG_SYSTEM_ERROR as u16),
            pci_flag(status, PCI_STATUS_DETECTED_PARITY as u16),
            pci_flag(status, PCI_STATUS_INTX as u16));
        if cmd & 0x04 != 0 {
            let latency = get_conf_byte(d, PCI_LATENCY_TIMER as usize);
            let cache_line = get_conf_byte(d, PCI_CACHE_LINE_SIZE as usize);
            print!("\tLatency: {}", latency);
            if min_gnt != 0 || max_lat != 0 {
                print!(" (");
                if min_gnt != 0 {
                    print!("{}ns min", min_gnt * 250);
                }
                if min_gnt != 0 && max_lat != 0 {
                    print!(",");
                }
                if max_lat != 0 {
                    print!("{}ns max", max_lat * 250);
                }
                print!(")");
            }
            if cache_line != 0 {
                print!(", Cache Line Size: {} bytes", cache_line * 4);
            }
            println!();
        }
    }

    if d.dev.phy_slot.is_some() && d.dev.phy_slot.as_ref().unwrap().len() > 0 {
        println!(
            "\tPhysical Slot: {}",
            <Option<std::string::String> as Clone>::clone(&d.dev.phy_slot).unwrap()
        );
    }

    if c.verbose > 1 {
        let int_pin = get_conf_byte(d, PCI_INTERRUPT_PIN as usize);

        if d.dev.irq.is_some() {
            if int_pin != 0 {
                let ch_a = 'A' as u32;
                let int_pin_char = int_pin as u32 + ch_a - 1;
                println!(
                    "\tInterrupt: pin {} routed to IRQ {}",
                    std::char::from_u32(int_pin_char.into()).unwrap(),
                    d.dev.irq.unwrap()
                );
            } else {
                println!("\tInterrupt: pin ? routed to IRQ {}", d.dev.irq.unwrap());
            }
        }
        if d.dev.numa_node.is_some() {
            println!("\tNUMA node: {}", d.dev.numa_node.unwrap());
        }
        println!();
    }

    if c.verbose <= 1 {
        let cmd = get_conf_word(d, PCI_COMMAND as usize);
        let status = get_conf_word(d, PCI_STATUS as usize);
        let latency = get_conf_byte(d, PCI_LATENCY_TIMER as usize);
        print!("\tFlags: ");
        if cmd & PCI_COMMAND_MASTER as u16 != 0 {
            print!("bus master, ");
        }
        if cmd & PCI_COMMAND_VGA_PALETTE as u16 != 0 {
            print!("VGA palette snoop, ");
        }
        if cmd & PCI_COMMAND_WAIT as u16 != 0 {
            print!("stepping, ");
        }
        if cmd & PCI_COMMAND_FAST_BACK != 0 {
            print!("fast Back2Back, ");
        }
        if status & PCI_STATUS_66MHZ as u16 != 0 {
            print!("66MHz, ");
        }
        if status & PCI_STATUS_UDF as u16 != 0 {
            print!("user-definable features, ");
        }
        let speed = status & PCI_STATUS_DEVSEL_MASK;
        let speed_char;
        match speed {
            PCI_STATUS_DEVSEL_SLOW => speed_char = "slow",
            PCI_STATUS_DEVSEL_MEDIUM => speed_char = "medium",
            PCI_STATUS_DEVSEL_FAST => speed_char = "fast",
            _ => speed_char = "??",
        }
        print!("{} devsel", speed_char);
        if cmd & PCI_COMMAND_MASTER as u16 != 0 {
            print!(", latency {}", latency);
        }
        if d.dev.irq.is_some() && d.dev.irq.unwrap() != 0 {
            print!(", IRQ {}", d.dev.irq.unwrap());
        }
        if d.dev.numa_node.is_some() {
            println!(", NUMA node {}", d.dev.numa_node.unwrap());
        }
        println!();
    }
    if bist & PCI_BIST_CAPABLE != 0 {
        if bist & PCI_BIST_START != 0 {
            print!("\tBIST is running\n");
        } else {
            print!("\tBIST result: {:2x}\n", bist & PCI_BIST_CODE_MASK);
        }
    }

    match htype {
        PCI_HEADER_TYPE_NORMAL => {
            show_htype0(d, c);
        }
        PCI_HEADER_TYPE_BRIDGE => {
            show_htype1(d, c);
        }
        PCI_HEADER_TYPE_CARDBUS => {
            show_htype2(d, c);
        }
        _ => {
            show_htype_unknown(d, c);
        }
    }
}

///
pub fn find_bridges(brs: &Vec<Bridge>, slot: PciSlotFilter) -> usize {
    for i in 0..brs.len() {
        if brs[i].slot == slot {
            return i;
        }
    }
    return usize::MAX;
}

///
pub fn get_sec_sub(d: &Device, path: &str) -> (u32, u32) {
    let ht = get_conf_byte(d, PCI_HEADER_TYPE as usize) & 0x7f;
    let class = d.dev.class_id.unwrap();
    let tmp = (ht == PCI_HEADER_TYPE_BRIDGE) || (ht == PCI_HEADER_TYPE_CARDBUS);
    let sec;
    let sub;
    if class >> 8 == PCI_BASE_CLASS_BRIDGE as u32 && tmp {
        if ht == PCI_HEADER_TYPE_BRIDGE {
            sub = get_conf_byte(d, PCI_SUBORDINATE_BUS as usize) as u32;
            sec = get_conf_byte(d, PCI_SECONDARY_BUS as usize) as u32;
        } else {
            sub = get_conf_byte(d, PCI_CB_SUBORDINATE_BUS as usize) as u32;
            sec = get_conf_byte(d, PCI_CB_CARD_BUS as usize) as u32;
        }
    } else {
        sub = 0;
        sec = 0;
    }
    if sec != 0 || sub != 0 {
        return (sec, sub);
    }
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let file_name = entry.file_name();
                let input = format!("{}", file_name.to_string_lossy());
                let parts: Vec<&str> = input.split(':').collect();
                if parts.len() >= 2 {
                    if let Ok(num) = u32::from_str_radix(parts[1], 16) {
                        return (num, num);
                    }
                } else {
                    println!("Invalid input format.");
                }
            }
        }
    }
    return (0, 0);
}

///
pub fn grow_tree(c: &Config) -> Vec<Bridge> {
    let devices = read_devices(c);
    let mut brs: Vec<Bridge> = Vec::new();
    for device in devices {
        if device_filter(&device, c, true) {
            let filename = format!("{}/{}", DIR_NAME, device.name);
            if let Ok(target_path) = fs::read_link(filename) {
                let path = target_path.to_str().unwrap();
                let keyword = "devices/pci";
                let remain: &str;
                if let Some(index) = path.find(keyword) {
                    remain = &path[(index + keyword.len())..];
                } else {
                    continue;
                }
                let mut parts: Vec<&str> = remain.split('/').collect();
                parts.remove(0);
                let path_bus_str = format!("{}/{}/pci_bus", DIR_NAME, device.name);
                let path_bus = Path::new(path_bus_str.as_str());
                let (sec, sub) = get_sec_sub(&device, &path_bus_str);
                if parts.len() == 1 {
                    let br = Bridge {
                        slot: pci_filter_parse_slot(parts[0]).unwrap(),
                        secondary: sec as u32,
                        subordinate: sub as u32,
                        br_dev: device,
                        pci_bus: path_bus.exists(),
                        child: Vec::new(),
                    };
                    brs.push(br);
                } else if parts.len() > 1 {
                    let br = Bridge {
                        slot: pci_filter_parse_slot(parts[parts.len() - 1]).unwrap(),
                        secondary: sec as u32,
                        subordinate: sub as u32,
                        br_dev: device,
                        pci_bus: path_bus.exists(),
                        child: Vec::new(),
                    };
                    let mut current_brs = &mut brs;
                    for i in 0..parts.len() - 1 {
                        let idx =
                            find_bridges(&current_brs, pci_filter_parse_slot(parts[i]).unwrap());
                        if idx != usize::MAX {
                            current_brs = &mut current_brs[idx].child;
                        }
                    }
                    current_brs.push(br);
                } else {
                    continue;
                }
            }
        }
    }
    return brs;
}

///
pub fn print_spaces(count: usize) {
    for _ in 0..count {
        print!(" ");
    }
}

///
pub fn generate_spaces(count: usize) -> String {
    let spaces: String = std::iter::repeat(' ').take(count).collect();
    spaces
}

///
pub fn show_tree_dev(b: &Bridge, c: &Config, line: &str) {
    let sp: String;
    if !b.pci_bus {
        sp = String::from("");
    } else if b.secondary == 0 {
        sp = String::from("--");
    } else if b.secondary == b.subordinate {
        sp = format!("-[{:0>2x}]--", b.subordinate);
    } else {
        sp = format!("-[{:0>2x}-{:0>2x}]--", b.secondary, b.subordinate);
    }
    if b.child.len() > 0 {
        let p = format!(
            "{}{:0>2x}.{:1x}{}",
            line,
            b.slot.slot.unwrap(),
            b.slot.func.unwrap(),
            sp
        );
        show_tree_bus(&b.child, c, &p);
        return;
    }
    print!(
        "{}{:0>2x}.{:1x}{}",
        line,
        b.slot.slot.unwrap(),
        b.slot.func.unwrap(),
        sp
    );

    if c.verbose > 0 && !b.pci_bus {
        print!("  {}", pci_lookup_name(&b.br_dev, "vendor_device", c));
    }
    println!();
}

///
pub fn show_tree_bus(brs: &Vec<Bridge>, c: &Config, line: &str) {
    let mut s = line.to_string();
    for i in 0..brs.len() {
        if i != 0 {
            s = line.replace("+", "|").replace(|c: char| c != '|', " ");
        }
        if i == 0 && brs.len() == 1 {
            s += "--";
        } else if i == brs.len() - 1 {
            s += "\\-";
        } else {
            s += "+-";
        }
        show_tree_dev(&brs[i], c, &s);
    }
}

///
pub fn show_tree_bridge(brs: &Vec<Bridge>, c: &Config, line: &str) {
    show_tree_bus(brs, c, line);
}

///
pub fn show_forest(brs: &Vec<Bridge>, c: &Config) {
    let s = format!("-[{:0>4x}:{:0>2x}]-", 0, 0);
    show_tree_bridge(brs, c, &s);
}

///
pub fn map_the_bus(c: &Config) {
    println!("WARNING: Bus mapping can be reliable only with direct hardware access enabled.\n");
    let devices = read_devices(c);
    let brs = grow_tree(c);
    let mut map_bridges: HashMap<u32, Vec<String>> = HashMap::new();
    let mut map_via: HashMap<u32, String> = HashMap::new();
    for i in 0..brs.len() {
        let b = &brs[i];
        let via_v = format!(
            "Entered via {:0>2x}:{:0>2x}.{}",
            b.slot.bus.unwrap(),
            b.slot.slot.unwrap(),
            b.slot.func.unwrap()
        );
        if b.child.len() > 0 {
            for cb in &b.child {
                let via_k = cb.slot.bus.unwrap();
                if let Some(value) = map_via.get_mut(&via_k) {
                    *value = String::from(via_v.clone());
                } else {
                    map_via.insert(via_k, String::from(via_v.clone()));
                }
            }
        }
    }
    for device in devices {
        if device_filter(&device, c, true) {
            show_device(&device, c);
            let path_bus_str = format!("{}/{}/pci_bus", DIR_NAME, device.name);
            let path_bus = Path::new(path_bus_str.as_str());
            if path_bus.exists() {
                let header = get_conf_byte(&device, PCI_HEADER_TYPE as usize) & 0x7;
                let np;
                let ns;
                let nl;
                match header {
                    PCI_HEADER_TYPE_BRIDGE => {
                        np = PCI_PRIMARY_BUS as usize;
                        ns = PCI_SECONDARY_BUS as usize;
                        nl = PCI_SUBORDINATE_BUS as usize;
                    }
                    PCI_HEADER_TYPE_CARDBUS => {
                        np = PCI_CB_PRIMARY_BUS as usize;
                        ns = PCI_CB_CARD_BUS as usize;
                        nl = PCI_CB_SUBORDINATE_BUS as usize;
                    }
                    _ => {
                        np = PCI_PRIMARY_BUS as usize;
                        ns = PCI_SECONDARY_BUS as usize;
                        nl = PCI_SUBORDINATE_BUS as usize;
                    }
                }
                let this = get_conf_byte(&device, np);
                let first = get_conf_byte(&device, ns);
                let last = get_conf_byte(&device, nl);
                let slot = pci_filter_parse_slot(&device.name).unwrap();
                println!(
                    "## {:0>2x}:{:0>2x}.{} is a bridge from {:0>2x} to {:0>2x}-{:0>2x}",
                    slot.bus.unwrap(),
                    slot.slot.unwrap(),
                    slot.func.unwrap(),
                    this,
                    first,
                    last
                );
                let bus_k = slot.bus.unwrap();
                if let Some(value) = map_bridges.get_mut(&bus_k) {
                    let line = format!(
                        "\t{:0>2x}.{} Bridge to {:0>2x}-{:0>2x}\n",
                        slot.slot.unwrap(),
                        slot.func.unwrap(),
                        first,
                        last
                    );
                    value.push(line);
                } else {
                    let mut map_br_v0 = Vec::new();
                    map_br_v0.push(String::from("Primary host bus\n"));
                    let line = format!(
                        "\t{:0>2x}.{} Bridge to {:0>2x}-{:0>2x}\n",
                        slot.slot.unwrap(),
                        slot.func.unwrap(),
                        first,
                        last
                    );
                    map_br_v0.push(line);
                    map_bridges.insert(bus_k, map_br_v0);
                }
            }
        }
    }
    if map_bridges.len() == 0 {
        let mut map_br_v0 = Vec::new();
        map_br_v0.push(String::from("Primary host bus\n"));
        map_bridges.insert(0, map_br_v0);
    }
    println!("\nSummary of buses:\n");
    for i in 0..256 {
        if let Some(value) = map_bridges.get(&i) {
            print!("{:0>2x}: ", i);
            print!("{}", value[0]);
            for j in (1..value.len()).rev() {
                print!("{}", value[j]);
            }
        }
        if let Some(value) = map_via.get(&i) {
            print!("{:0>2x}:", i);
            println!(" {}", value);
        }
    }
}

///
pub fn show_device(d: &Device, c: &Config) {
    if c.m > 0 {
        show_machine(d, c);
    } else {
        if c.verbose > 0 {
            show_verbose(d, c);
        } else {
            show_terse(d, c);
        }
        if c.kernel || c.verbose > 0 {
            show_kernel(d);
        }
    }
    if c.hex_dump > 0 {
        show_hex_dump(d, c);
    }
    if c.hex_dump > 0 || c.verbose > 0 {
        println!();
    }
}

///
pub fn show(config: &Config) {
    let devices = read_devices(config);
    for device in devices {
        if device_filter(&device, config, true) {
            show_device(&device, config);
        }
    }
}

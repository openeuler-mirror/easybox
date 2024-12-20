//! This file is part of the easybox package.
//
// (c) Xu Biang <xubiang@foxmail.com>
// (c) Chen Yuchen <yuchen@isrc.iscas.ac.cn>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use clap::{crate_version, Arg, ArgMatches, Command};
use errno::errno;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{
    ffi::CString,
    fs::File,
    io::{BufRead, BufReader},
    mem, vec,
};
use uucore::{
    error::{UResult, USimpleError, UUsageError},
    format_usage,
    libc::{
        arpreq, ifreq, sockaddr, sockaddr_storage, AF_INET, ATF_COM, ATF_NETMASK, ATF_PERM,
        ATF_PUBL, ATF_USETRAILERS, ENOENT, ENXIO, IFNAMSIZ, SOCK_DGRAM,
    },
    net_tools::{
        get_aftype, get_hwntype, get_hwtype, AFType, HWType, FLAG_NUM, FLAG_SYM, _PATH_PROCNET_ARP,
    },
};

use crate::arp_unsafe::{
    ifru_hwaddr_wrapper, ioctl_delete_arp_wrapper, ioctl_get_hardware_address_wrapper,
    ioctl_set_arp_wrapper, memcpy_wrapper, socket_wrapper, zeroed_wrapper,
};

///
pub static DFLT_HW: &str = "ether";
///
pub static DFLT_AF: &str = "inet";
///
static E_OPTERR: i32 = 3;

///
pub mod options {
    ///
    pub static VERBOSE: &str = "verbose";
    ///
    pub static VERSION: &str = "version";
    ///
    pub static ALL: &str = "all";
    ///
    pub static LINUX_STYLE: &str = "linux_style";
    ///
    pub static DELETE: &str = "delete";
    ///
    pub static FILE: &str = "file";
    ///
    pub static NUMERIC: &str = "numeric";
    ///
    pub static SET: &str = "set";
    ///
    pub static PROTOCOL: &str = "protocol";
    ///
    pub static HWTYPE: &str = "hw-type";
    ///
    pub static DEVICE: &str = "device";
    ///
    pub static USE_DEVICE: &str = "use-device";
    ///
    pub static SYMBOLIC: &str = "symbolic";
    ///
    pub static ARGS: &str = "args";
}

#[derive(Clone)]
///
pub enum ARPMode {
    ///
    ShowEntry,
    ///
    ShowAllEntries,
    ///
    ProcessEtherFile,
    ///
    DeleteEntry,
    ///
    SetEntry,
}

#[derive(Clone)]
///
pub struct Config {
    /// all entries
    pub all: bool,
    /// true=linux style, false=BSD style
    pub linux_style: bool,
    /// be verbose
    pub verbose: bool,
    /// do not resolve addresses
    pub numeric: i32,
    /// network interface (e.g. eth0), set by -i
    pub device: Option<String>,
    /// whether read hardware address (e.g. MAC) from given device, set by -D
    pub use_device: bool,
    /// protocol family, should be inet in arp
    pub protocol: AFType,
    /// hardware type, default ether, set by -H -t
    pub hardware: HWType,
    /// use symbolic names, not yet supported
    pub symbolic: i32,
    ///
    pub mode: ARPMode,
    ///
    pub sockfd: i32,
    ///
    pub hw_set: bool,
    ///
    pub show_entry_args: Vec<String>,
    ///
    pub delete_entry_args: Vec<String>,
    ///
    pub process_file_args: Vec<String>,
    ///
    pub set_entry_args: Vec<String>,
}

impl Config {
    ///
    pub fn from(arg_matches: ArgMatches) -> UResult<Self> {
        let numeric = if arg_matches.get_flag(options::NUMERIC) {
            FLAG_NUM
        } else {
            0
        };

        let device = arg_matches.get_one::<String>(options::DEVICE).cloned();

        let symbolic = if arg_matches.get_flag(options::SYMBOLIC) {
            eprintln!("arp: -N not yet supported.");
            FLAG_SYM
        } else {
            0
        };

        let mut mode = ARPMode::ShowEntry;
        if arg_matches.get_flag(options::ALL) {
            mode = ARPMode::ShowAllEntries;
        }
        if arg_matches.contains_id(options::FILE) {
            mode = ARPMode::ProcessEtherFile;
        }
        if arg_matches.contains_id(options::DELETE) {
            mode = ARPMode::DeleteEntry;
        }
        if arg_matches.contains_id(options::SET) {
            mode = ARPMode::SetEntry;
        }

        let protocol = match arg_matches.get_one::<AFType>(options::PROTOCOL).cloned() {
            Some(af) => {
                if af.af != AF_INET {
                    return Err(USimpleError::new(
                        -1,
                        format!("{}: kernel only supports 'inet'.", &af.name),
                    ));
                }
                af
            }
            None => get_aftype(DFLT_AF).unwrap().clone(),
        };

        let mut hw_set = false;
        let hardware = match arg_matches.get_one::<HWType>(options::HWTYPE).cloned() {
            Some(hw) => {
                if hw.alen <= 0 {
                    return Err(USimpleError::new(
                        -1,
                        format!("{}: hardware type without ARP support.", &hw.name),
                    ));
                }
                hw_set = true;
                hw
            }
            None => match get_hwtype(DFLT_HW) {
                Some(hw_type) => hw_type,
                None => {
                    return Err(USimpleError::new(
                        -1,
                        format!("{}: hardware type not supported!", DFLT_HW),
                    ))
                }
            }
            .clone(),
        };

        let sockfd = socket_wrapper(AF_INET, SOCK_DGRAM, 0);
        if sockfd < 0 {
            eprintln!("socket");
            return Err((-1).into());
        }

        let linux_style = match mode {
            ARPMode::ShowEntry => true,
            _ => arg_matches.get_flag(options::LINUX_STYLE),
        };

        let show_entry_args = arg_matches
            .get_many::<String>(options::ARGS)
            .unwrap_or_default()
            .map(std::string::ToString::to_string)
            .collect();

        let delete_entry_args: Vec<String> = arg_matches
            .get_many::<String>(options::DELETE)
            .unwrap_or_default()
            .map(std::string::ToString::to_string)
            .collect();

        let process_file_args: Vec<String> = arg_matches
            .get_many::<String>(options::FILE)
            .unwrap_or_default()
            .map(std::string::ToString::to_string)
            .collect();

        let set_entry_args: Vec<String> = arg_matches
            .get_many::<String>(options::SET)
            .unwrap_or_default()
            .map(std::string::ToString::to_string)
            .collect();

        Ok(Self {
            all: arg_matches.get_flag(options::ALL),
            linux_style,
            verbose: arg_matches.get_flag(options::VERBOSE),
            numeric,
            device,
            use_device: arg_matches.get_flag(options::USE_DEVICE),
            protocol,
            hardware,
            symbolic,
            mode,
            sockfd,
            hw_set,
            show_entry_args,
            delete_entry_args,
            process_file_args,
            set_entry_args,
        })
    }
}

///
pub fn parse_arp_cmd_args(
    args: impl uucore::Args,
    about: &str,
    usage: &str,
    after_help: &str,
) -> UResult<Config> {
    let command = arp_app(about, usage, after_help);
    let arg_list = args.collect_lossy();
    match command.try_get_matches_from(arg_list) {
        Ok(arg_matches) => Config::from(arg_matches),
        Err(e) => {
            if e.kind() == clap::ErrorKind::ValueValidation {
                Err(USimpleError::new(
                    -1,
                    e.to_string().strip_prefix("error: ").unwrap_or_default(),
                ))
            } else {
                e.exit();
            }
        }
    }
}

///
pub fn arp_app<'a>(about: &'a str, usage: &'a str, after_help: &'a str) -> Command<'a> {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about(about)
        .after_help(after_help)
        .override_usage(format_usage(usage))
        .infer_long_args(true)
        .group(clap::ArgGroup::new("actions").args(&[options::DELETE, options::SET, options::FILE]))
        .arg(
            Arg::new(options::ALL)
                .short('a')
                .long(options::ALL)
                .action(clap::ArgAction::SetTrue)
                .help("display (all) hosts in alternative (BSD) style")
                .display_order(10),
        )
        .arg(
            Arg::new(options::LINUX_STYLE)
                .short('e')
                .action(clap::ArgAction::SetTrue)
                .help("display (all) hosts in default (Linux) style")
                .display_order(20),
        )
        .arg(
            Arg::new(options::SET)
                .short('s')
                .long(options::SET)
                .action(clap::ArgAction::Set)
                .multiple_values(true)
                .min_values(0)
                .value_name("host")
                .help("set a new ARP entry")
                .display_order(30),
        )
        .arg(
            Arg::new(options::DELETE)
                .short('d')
                .long(options::DELETE)
                .action(clap::ArgAction::Set)
                .multiple_values(true)
                .min_values(0)
                .value_name("host")
                .help("delete a specified entry")
                .display_order(40),
        )
        .arg(
            Arg::new(options::VERBOSE)
                .short('v')
                .long(options::VERBOSE)
                .action(clap::ArgAction::SetTrue)
                .help("be verbose")
                .display_order(50),
        )
        .arg(
            Arg::new(options::NUMERIC)
                .short('n')
                .long(options::NUMERIC)
                .action(clap::ArgAction::SetTrue)
                .help("don't resolve names")
                .display_order(60),
        )
        .arg(
            Arg::new(options::DEVICE)
                .short('i')
                .long(options::DEVICE)
                .action(clap::ArgAction::Set)
                .value_name("if")
                .help("specify network interface (e.g. eth0)")
                .display_order(70),
        )
        .arg(
            Arg::new(options::USE_DEVICE)
                .short('D')
                .long(options::USE_DEVICE)
                .action(clap::ArgAction::SetTrue)
                .help("read <hwaddr> from given device")
                .display_order(80),
        )
        .arg(
            Arg::new(options::PROTOCOL)
                .short('p')
                .visible_short_alias('A')
                .long(options::PROTOCOL)
                .action(clap::ArgAction::Set)
                .value_name("af")
                .value_parser(clap::value_parser!(AFType))
                .help("specify protocol family")
                .display_order(90),
        )
        .arg(
            Arg::new(options::FILE)
                .short('f')
                .long(options::FILE)
                .action(clap::ArgAction::Set)
                .min_values(0)
                .help("read new entries from file or from /etc/ethers")
                .display_order(100),
        )
        .arg(
            Arg::new(options::HWTYPE)
                .short('H')
                .short_alias('t')
                .long(options::HWTYPE)
                .action(clap::ArgAction::Set)
                .value_parser(clap::value_parser!(HWType))
                .help("specify hardware address type")
                .hide(true),
        )
        .arg(
            Arg::new(options::SYMBOLIC)
                .short('N')
                .long(options::SYMBOLIC)
                .action(clap::ArgAction::SetTrue)
                .hide(true),
        )
        .arg(
            Arg::new(options::ARGS)
                .hide(true)
                .action(clap::ArgAction::Set)
                .multiple_values(true),
        )
}

///
pub fn handle_input(config: Config) -> UResult<()> {
    match config.mode {
        ARPMode::ShowEntry => arp_show(&config),
        ARPMode::ShowAllEntries => arp_show(&config),
        ARPMode::ProcessEtherFile => arp_file(&config),
        ARPMode::DeleteEntry => arp_del(&config),
        ARPMode::SetEntry => arp_set(&config),
    }
}

/// Print the contents of an ARP request block.
fn arp_disp(name: &str, ip: &str, xhw: &HWType, arp_flags: i32, hwa: &str, mask: &str, dev: &str) {
    print!("{} ({}) at ", name, ip);

    if (arp_flags & ATF_COM) == 0 {
        if (arp_flags & ATF_PUBL) != 0 {
            print!("<from_interface> ")
        } else {
            print!("<incomplete> ")
        }
    } else {
        print!("{} [{}] ", hwa, xhw.name);
    }

    if arp_flags & ATF_NETMASK != 0 {
        print!("netmask {} ", mask);
    }
    if arp_flags & ATF_PERM != 0 {
        print!("PERM ");
    }
    if arp_flags & ATF_PUBL != 0 {
        print!("PUB ");
    }
    if (arp_flags & ATF_USETRAILERS) != 0 {
        print!("TRAIL ");
    }

    println!("on {}", dev);
}

static FIRST_CALL_ARP_DISP_2: AtomicBool = AtomicBool::new(true);

/// Print the contents of an ARP request block.
fn arp_disp_2(name: &str, xhw: &HWType, arp_flags: i32, hwa: &str, mask: &str, dev: &str) {
    if FIRST_CALL_ARP_DISP_2.swap(false, Ordering::SeqCst) {
        println!(
            "Address                  HWtype  HWaddress           Flags Mask            Iface"
        );
    }

    let mut flags: String = String::new();
    if arp_flags & ATF_COM != 0 {
        flags.push('C')
    }
    if arp_flags & ATF_PERM != 0 {
        flags.push('M')
    }
    if arp_flags & ATF_PUBL != 0 {
        flags.push('P')
    }
    if (arp_flags & ATF_USETRAILERS) != 0 {
        flags.push('T')
    }

    let mask = if arp_flags & ATF_NETMASK == 0 {
        ""
    } else {
        mask
    };

    print!("{:<23.23}  ", name);

    if arp_flags & ATF_COM == 0 {
        if arp_flags & ATF_PUBL != 0 {
            print!("{:<8.8}{:<20.20}", "*", "<from_interface>");
        } else {
            print!("{:<8.8}{:<20.20}", "", "<incomplete>");
        }
    } else {
        print!("{:<8.8}{:<20.20}", xhw.name, hwa);
    }

    println!("{:<6.6}{:<15.15} {}", flags, mask, dev);
}

/// Display the contents of the ARP cache in the kernel.
fn arp_show(config: &Config) -> UResult<()> {
    let mut ss: sockaddr_storage = zeroed_wrapper();
    let mut ip: String = String::new();
    let mut hwa: String = String::new();
    let mut mask: String = String::new();
    let mut line: String = String::new();
    let mut dev: String = String::new();
    let mut typ: i32 = 0;
    let mut flags: i32 = 0;
    let mut entries: i32 = 0;
    let mut showed: i32 = 0;
    let device = config.device.clone().unwrap_or_default();

    let ap = &config.protocol;

    let host = if let Some(h) = config.show_entry_args.first() {
        if let Err(e) = ap.input.unwrap()(0, h, &mut ss) {
            return Err(USimpleError::new(-1, e));
        }
        ap.sprint.unwrap()(&ss, 1).unwrap_or_default()
    } else {
        String::new()
    };

    /* Open the PROCps kernel table. */
    let fd = match File::open(_PATH_PROCNET_ARP) {
        Ok(v) => v,
        Err(_) => {
            eprintln!("{}: {}", _PATH_PROCNET_ARP, errno());
            return Err((-1).into());
        }
    };
    let mut reader = BufReader::new(fd);

    /* Bypass header -- read until newline */
    reader.read_line(&mut line)?;
    loop {
        line.clear();
        /* Read the ARP cache entries. */
        if reader.read_line(&mut line)? == 0 {
            break;
        }

        let num = line
            .split_ascii_whitespace()
            .take(6)
            .enumerate()
            .fold(0, |acc, (i, item)| {
                match i {
                    0 => ip = item.to_string(),
                    1 => {
                        if let Some(stripped_item) = item.strip_prefix("0x") {
                            typ = stripped_item.parse().unwrap_or(0)
                        }
                    }
                    2 => {
                        if let Some(stripped_item) = item.strip_prefix("0x") {
                            flags = stripped_item.parse().unwrap_or(0)
                        }
                    }
                    3 => hwa = item.to_string(),
                    4 => mask = item.to_string(),
                    5 => dev = item.to_string(),
                    _ => (),
                }
                acc + 1
            });

        if num < 4 {
            break;
        }

        if num == 5 {
            /*
             * This happens for incomplete ARP entries for which there is
             * no hardware address in the line.
             */
            line.split_ascii_whitespace()
                .take(5)
                .enumerate()
                .fold(0, |acc, (i, item)| {
                    match i {
                        0 => ip = item.to_string(),
                        1 => {
                            if let Some(stripped_item) = item.strip_prefix("0x") {
                                typ = stripped_item.parse().unwrap_or(0)
                            }
                        }
                        2 => {
                            if let Some(stripped_item) = item.strip_prefix("0x") {
                                flags = stripped_item.parse().unwrap_or(0)
                            }
                        }
                        3 => mask = item.to_string(),
                        4 => dev = item.to_string(),
                        _ => (),
                    }
                    acc + 1
                });
            hwa = String::from("");
        }

        entries += 1;
        /* if the user specified hw-type differs, skip it */
        if config.hw_set && typ != config.hardware.typ {
            continue;
        }

        /* if the user specified address differs, skip it */
        if !host.is_empty() & !ip.eq(&host) {
            continue;
        }

        /* if the user specified device differs, skip it */
        if !device.is_empty() & !dev.eq(&device) {
            continue;
        }

        showed += 1;
        let mut hostname: String;
        /* This IS ugly but it works -be */
        if config.numeric != 0 {
            hostname = String::from("?");
        } else {
            if ap.input.unwrap()(0, &ip, &mut ss).is_err() {
                hostname = ip.clone();
            } else {
                hostname = ap.sprint.unwrap()(&ss, config.numeric | 0x8000).unwrap_or_default();
            }
            if hostname.eq(&ip) {
                hostname = String::from("?");
            }
        };

        let xhw = get_hwntype(typ).unwrap_or_else(|| get_hwtype(DFLT_HW).unwrap());
        if config.linux_style {
            hostname = if hostname.eq("?") {
                ip.clone()
            } else {
                hostname
            };
            arp_disp_2(&hostname, xhw, flags, &hwa, &mask, &dev);
        } else {
            arp_disp(&hostname, &ip, xhw, flags, &hwa, &mask, &dev);
        }
    }

    if config.verbose {
        println!(
            "Entries: {}\tSkipped: {}\tFound: {}",
            entries,
            entries - showed,
            showed
        );
    }

    if showed == 0 {
        if !host.is_empty() && !config.all {
            let hostname = config.show_entry_args.first().unwrap();
            println!("{} ({}) -- no entry", hostname, host);
        } else if config.hw_set || !host.is_empty() || !device.is_empty() {
            println!("arp: in {} entries no match found.", entries);
        }
    }

    Ok(())
}

/// Delete an entry from the ARP cache.
fn arp_del(config: &Config) -> UResult<()> {
    let mut req: arpreq = zeroed_wrapper();
    let mut ss: sockaddr_storage = zeroed_wrapper();
    let mut device: String = config.device.clone().unwrap_or_default();

    /* Resolve the host name. */
    let mut host;
    match config.delete_entry_args.first() {
        Some(h) => host = h,
        None => return Err(USimpleError::new(-1, "need host name")),
    }
    if config.protocol.input.unwrap()(0, host, &mut ss).is_err() {
        config.protocol.herror.unwrap()(host);
        return Err((-1).into());
    }
    memcpy_wrapper(
        &ss as *const sockaddr_storage as *const u8,
        &mut req.arp_pa as *mut sockaddr as *mut u8,
        mem::size_of::<sockaddr>(),
    );

    if config.hw_set {
        req.arp_ha.sa_family = config.hardware.typ as u16;
    }

    req.arp_flags = ATF_PERM;

    let mut flags = 0;
    let mut args = config.delete_entry_args[1..].iter();
    while let Some(arg) = args.next() {
        if config.verbose {
            eprintln!("args={}", arg);
        }
        match arg {
            _ if arg == "pub" => flags |= 1,
            _ if arg == "priv" => flags |= 2,
            _ if arg == "temp" => flags &= !ATF_PERM,
            _ if arg == "trail" => flags |= ATF_USETRAILERS,
            _ if arg == "dontpub" => {}
            _ if arg == "auto" => {}
            _ if arg == "dev" => match args.next() {
                Some(dev) => {
                    device = dev.to_string();
                }
                None => return Err(UUsageError::new(E_OPTERR, "need dev name")),
            },
            _ if arg == "netmask" => match args.next() {
                Some(mask) => {
                    if mask != "255.255.255.255" {
                        host = mask;
                        if config.protocol.input.unwrap()(0, mask, &mut ss).is_err() {
                            config.protocol.herror.unwrap()(mask);
                            return Err((-1).into());
                        }
                        memcpy_wrapper(
                            &ss as *const sockaddr_storage as *const u8,
                            &mut req.arp_netmask as *mut sockaddr as *mut u8,
                            mem::size_of::<sockaddr>(),
                        );
                        flags |= ATF_NETMASK;
                    }
                }
                None => return Err(UUsageError::new(E_OPTERR, "need netmask value")),
            },
            _ => {
                return Err(UUsageError::new(
                    E_OPTERR,
                    format!("unknown modifier: {}", arg),
                ))
            }
        }
    }

    // if neither priv nor pub is given, work on both
    if flags == 0 {
        flags = 3;
    }
    memcpy_wrapper(
        device.as_ptr(),
        req.arp_dev.as_mut_ptr() as *mut u8,
        device.len().min(16),
    );

    /* unfortuatelly the kernel interface does not allow us to
    delete private entries anlone, so we need this hack
    to avoid "not found" errors if we try both. */
    let mut deleted = false;
    let mut dontpub = false;

    /* Call the kernel. */
    if flags & 2 != 0 {
        if config.verbose {
            eprintln!("arp: SIOCDARP(dontpub)");
        }
        if ioctl_delete_arp_wrapper(config.sockfd, req).is_err() {
            if errno() == errno::Errno(ENXIO) || errno() == errno::Errno(ENOENT) {
                if flags & 1 != 0 {
                    dontpub = true;
                } else {
                    println!("No ARP entry for {}", host);
                    return Err((-1).into());
                }
            } else {
                eprintln!("SIOCDARP(dontpub): {}", errno());
                return Err((-1).into());
            }
        } else {
            deleted = true;
        }
    }

    if dontpub || (!deleted && (flags & 1 != 0)) {
        req.arp_flags |= ATF_PUBL;
        if config.verbose {
            eprintln!("arp: SIOCDARP(pub)");
        }
        if ioctl_delete_arp_wrapper(config.sockfd, req).is_err() {
            if errno() == errno::Errno(ENXIO) || errno() == errno::Errno(ENOENT) {
                println!("No ARP entry for {}", host);
                return Err((-1).into());
            } else {
                eprintln!("SIOCDARP(pub): {}", errno());
                return Err((-1).into());
            }
        }
    }

    Ok(())
}

/// Get the hardware address to a specified interface name.
fn arp_getdevhw(
    config: &Config,
    ifname: &str,
    sa: &mut sockaddr,
    hw: Option<HWType>,
) -> Result<(), i32> {
    let mut ifr: ifreq = zeroed_wrapper();

    let c_ifname = CString::new(ifname).unwrap_or_default();
    for (i, byte) in c_ifname.as_bytes_with_nul().iter().enumerate() {
        if i >= IFNAMSIZ {
            break;
        }
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            ifr.ifr_name[i] = *byte as i8;
        };
        #[cfg(target_arch = "aarch64")]
        {
            ifr.ifr_name[i] = *byte;
        };
    }

    if ioctl_get_hardware_address_wrapper(config.sockfd, &mut ifr).is_err() {
        eprintln!("arp: cant get HW-Address for `{}': {}.", ifname, errno());
        return Err(-1);
    }

    if let Some(h) = hw {
        if ifru_hwaddr_wrapper(ifr).sa_family != h.typ as u16 {
            eprintln!("arp: protocol type mismatch.");
            return Err(-1);
        }
    }

    memcpy_wrapper(
        &ifru_hwaddr_wrapper(ifr) as *const sockaddr as *const u8,
        sa as *mut sockaddr as *mut u8,
        mem::size_of::<sockaddr>(),
    );

    if config.verbose {
        let xhw = get_hwntype(ifru_hwaddr_wrapper(ifr).sa_family.into())
            .filter(|hw| hw.print.is_some())
            .unwrap_or_else(|| get_hwntype(-1).unwrap());
        let hwaddr = {
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            {
                xhw.print.unwrap()(ifru_hwaddr_wrapper(ifr).sa_data.to_vec())
            }
            #[cfg(target_arch = "aarch64")]
            {
                xhw.print.unwrap()(
                    ifru_hwaddr_wrapper(ifr)
                        .sa_data
                        .iter()
                        .map(|&x| x as i8)
                        .collect::<Vec<i8>>(),
                )
            }
        };
        eprintln!(
            "arp: device `{}' has HW address {} `{}'.",
            ifname, xhw.name, hwaddr
        );
    }

    Ok(())
}

/// Set an entry in the ARP cache.
fn arp_set(config: &Config) -> UResult<()> {
    let mut req: arpreq = zeroed_wrapper();
    let mut ss: sockaddr_storage = zeroed_wrapper();
    let mut device: String = config.device.clone().unwrap_or_default();

    /* Resolve the host name. */
    let host = match config.set_entry_args.first() {
        Some(h) => h,
        None => return Err(USimpleError::new(-1, "need host name")),
    };
    if config.protocol.input.unwrap()(0, host, &mut ss).is_err() {
        config.protocol.herror.unwrap()(host);
        return Err((-1).into());
    }
    memcpy_wrapper(
        &ss as *const sockaddr_storage as *const u8,
        &mut req.arp_pa as *mut sockaddr as *mut u8,
        mem::size_of::<sockaddr>(),
    );

    /* Fetch the hardware address. */
    let hw_addr = match config.set_entry_args.get(1) {
        Some(a) => a,
        None => return Err(USimpleError::new(-1, "need hardware address")),
    };
    if config.use_device {
        let hw = if config.hw_set {
            Some(config.hardware.clone())
        } else {
            None
        };
        if arp_getdevhw(config, hw_addr, &mut req.arp_ha, hw).is_err() {
            return Err((-1).into());
        }
    } else {
        if config.hardware.input.unwrap()(hw_addr, &mut ss).is_err() {
            return Err(USimpleError::new(-1, "invalid hardware address"));
        }
        memcpy_wrapper(
            &ss as *const sockaddr_storage as *const u8,
            &mut req.arp_ha as *mut sockaddr as *mut u8,
            mem::size_of::<sockaddr>(),
        );
    }

    /* Check out any modifiers. */
    let mut flags = ATF_PERM | ATF_COM;
    let mut args = config.set_entry_args[2..].iter();
    while let Some(arg) = args.next() {
        match arg {
            _ if arg == "temp" => flags &= !ATF_PERM,
            _ if arg == "pub" => flags |= ATF_PUBL,
            _ if arg == "priv" => flags &= !ATF_PUBL,
            _ if arg == "trail" => flags |= ATF_USETRAILERS,
            _ if arg == "dontpub" => {}
            _ if arg == "auto" => {}
            _ if arg == "dev" => match args.next() {
                Some(dev) => {
                    device = dev.to_string();
                }
                None => return Err(UUsageError::new(E_OPTERR, "need dev name")),
            },
            _ if arg == "netmask" => match args.next() {
                Some(mask) => {
                    if mask != "255.255.255.255" {
                        if config.protocol.input.unwrap()(0, mask, &mut ss).is_err() {
                            config.protocol.herror.unwrap()(mask);
                            return Err((-1).into());
                        }
                        memcpy_wrapper(
                            &ss as *const sockaddr_storage as *const u8,
                            &mut req.arp_netmask as *mut sockaddr as *mut u8,
                            mem::size_of::<sockaddr>(),
                        );
                        flags |= ATF_NETMASK;
                    }
                }
                None => return Err(UUsageError::new(E_OPTERR, "need netmask value")),
            },
            _ => {
                return Err(UUsageError::new(
                    E_OPTERR,
                    format!("unknown modifier: {}", arg),
                ))
            }
        }
    }

    /* Fill in the remainder of the request. */
    req.arp_flags = flags;
    memcpy_wrapper(
        device.as_ptr(),
        req.arp_dev.as_mut_ptr() as *mut u8,
        device.len().min(16),
    );

    /* Call the kernel. */
    if config.verbose {
        eprintln!("arp: SIOCSARP()");
    }
    if ioctl_set_arp_wrapper(config.sockfd, req).is_err() {
        eprintln!("SIOCSARP: {}", errno());
        return Err((-1).into());
    }

    Ok(())
}

/// Split the input string into multiple fields.
fn getargs(input: &str) -> Vec<String> {
    let temp = input.trim().to_string();
    let mut sp = String::new();
    let mut chars = temp.chars().peekable();
    let mut want: Option<char> = None;
    let mut arguments: Vec<String> = vec![];

    while let Some(&c) = chars.peek() {
        match c {
            ' ' | '\t' if want.is_none() => {
                chars.next();
                if !sp.is_empty() {
                    arguments.push(sp.clone());
                    sp.clear();
                }
            }
            '"' | '\'' if want.is_none() => {
                want = Some(c);
                chars.next();
            }
            '"' | '\'' if Some(c) == want => {
                want = None;
                chars.next();
            }
            '\\' if want.is_some() => {
                chars.next();
                if let Some(&next) = chars.peek() {
                    sp.push(next);
                    chars.next();
                }
            }
            _ => {
                sp.push(c);
                chars.next();
            }
        }
    }

    if !sp.is_empty() {
        arguments.push(sp);
    }

    arguments
}

/// Process an EtherFile.
fn arp_file(config: &Config) -> UResult<()> {
    let default_file = String::from("/etc/ethers");
    let name = config.process_file_args.first().unwrap_or(&default_file);

    let file = match File::open(name) {
        Ok(f) => f,
        Err(_) => {
            return Err(USimpleError::new(
                -1,
                format!("arp: cannot open etherfile {} !", name),
            ))
        }
    };

    /* Read the lines in the file. */
    let mut linenr = 0;
    let mut line_buf = String::new();
    let mut reader = BufReader::new(file);
    loop {
        line_buf.clear();
        if reader.read_line(&mut line_buf)? == 0 {
            break;
        }

        linenr += 1;
        if config.verbose {
            eprint!(">> {}", line_buf);
        }
        let line = line_buf.trim();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }

        let mut args = getargs(line);
        if args.len() < 2 {
            eprintln!(
                "arp: format error on line {} of etherfile {} !",
                linenr, name
            );
        }
        if args[0].find(':').is_some() {
            /* We have a correct ethers file, switch hw address and hostname for arp */
            args.swap(0, 1);
        }
        let mut tmp_config = config.clone();
        tmp_config.set_entry_args = args;
        if let Err(e) = arp_set(&tmp_config) {
            let err_str = format!("{}", e);
            if !err_str.is_empty() {
                eprintln!("{}", err_str);
            }
            eprintln!(
                "arp: cannot set entry on line {} of etherfile {} !",
                linenr, name
            );
        }
    }

    Ok(())
}

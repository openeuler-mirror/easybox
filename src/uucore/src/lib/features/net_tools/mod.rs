//! This file is part of the easybox package.
//
// (c) Xu Biang <xubiang@foxmail.com>
// (c) Chen Yuchen <yuchen@isrc.iscas.ac.cn>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

mod features;

use features::*;

use libc::sockaddr_storage;
use once_cell::sync::Lazy;

/* pathnames of the procfs files used by NET. */
pub static _PATH_PROCNET_IGMP: &str = "/proc/net/igmp";
pub static _PATH_PROCNET_IGMP6: &str = "/proc/net/igmp6";
pub static _PATH_PROCNET_TCP: &str = "/proc/net/tcp";
pub static _PATH_PROCNET_TCP6: &str = "/proc/net/tcp6";
pub static _PATH_PROCNET_UDP: &str = "/proc/net/udp";
pub static _PATH_PROCNET_UDP6: &str = "/proc/net/udp6";
pub static _PATH_PROCNET_UDPLITE: &str = "/proc/net/udplite";
pub static _PATH_PROCNET_UDPLITE6: &str = "/proc/net/udplite6";
pub static _PATH_PROCNET_SCTPEPTS: &str = "/proc/net/sctp/eps";
pub static _PATH_PROCNET_SCTP6EPTS: &str = "/proc/net/sctp6/eps";
pub static _PATH_PROCNET_SCTPASSOCS: &str = "/proc/net/sctp/assocs";
pub static _PATH_PROCNET_SCTP6ASSOCS: &str = "/proc/net/sctp6/assocs";
pub static _PATH_PROCNET_RAW: &str = "/proc/net/raw";
pub static _PATH_PROCNET_RAW6: &str = "/proc/net/raw6";
pub static _PATH_PROCNET_UNIX: &str = "/proc/net/unix";
pub static _PATH_PROCNET_ROUTE: &str = "/proc/net/route";
pub static _PATH_PROCNET_ROUTE6: &str = "/proc/net/ipv6_route";
pub static _PATH_PROCNET_RTCACHE: &str = "/proc/net/rt_cache";
pub static _PATH_PROCNET_AX25_ROUTE: &str = "/proc/net/ax25_route";
pub static _PATH_PROCNET_NR: &str = "/proc/net/nr";
pub static _PATH_PROCNET_NR_NEIGH: &str = "/proc/net/nr_neigh";
pub static _PATH_PROCNET_NR_NODES: &str = "/proc/net/nr_nodes";
pub static _PATH_PROCNET_ARP: &str = "/proc/net/arp";
pub static _PATH_PROCNET_AX25: &str = "/proc/net/ax25";
pub static _PATH_PROCNET_IPX_SOCKET1: &str = "/proc/net/ipx/socket";
pub static _PATH_PROCNET_IPX_SOCKET2: &str = "/proc/net/ipx";
pub static _PATH_PROCNET_IPX_ROUTE1: &str = "/proc/net/ipx/route";
pub static _PATH_PROCNET_IPX_ROUTE2: &str = "/proc/net/ipx_route";
pub static _PATH_PROCNET_ATALK: &str = "/proc/net/appletalk";
pub static _PATH_PROCNET_IP_BLK: &str = "/proc/net/ip_block";
pub static _PATH_PROCNET_IP_FWD: &str = "/proc/net/ip_forward";
pub static _PATH_PROCNET_IP_ACC: &str = "/proc/net/ip_acct";
pub static _PATH_PROCNET_IP_MASQ: &str = "/proc/net/ip_masquerade";
pub static _PATH_PROCNET_NDISC: &str = "/proc/net/ndisc";
pub static _PATH_PROCNET_IFINET6: &str = "/proc/net/if_inet6";
pub static _PATH_PROCNET_DEV: &str = "/proc/net/dev";
pub static _PATH_PROCNET_RARP: &str = "/proc/net/rarp";
pub static _PATH_ETHERS: &str = "/etc/ethers";
pub static _PATH_PROCNET_ROSE: &str = "/proc/net/rose";
pub static _PATH_PROCNET_ROSE_NEIGH: &str = "/proc/net/rose_neigh";
pub static _PATH_PROCNET_ROSE_NODES: &str = "/proc/net/rose_nodes";
pub static _PATH_PROCNET_ROSE_ROUTE: &str = "/proc/net/rose_routes";
pub static _PATH_PROCNET_X25: &str = "/proc/net/x25";
pub static _PATH_PROCNET_X25_ROUTE: &str = "/proc/net/x25/route";
pub static _PATH_PROCNET_DEV_MCAST: &str = "/proc/net/dev_mcast";
pub static _PATH_PROCNET_ATALK_ROUTE: &str = "/proc/net/atalk_route";
pub static _PATH_SYS_BLUETOOTH_L2CAP: &str = "/sys/kernel/debug/bluetooth/l2cap";
pub static _PATH_SYS_BLUETOOTH_RFCOMM: &str = "/sys/kernel/debug/bluetooth/rfcomm";
/* pathname for the netlink device */
pub static _PATH_DEV_ROUTE: &str = "/dev/route";

pub static RTACTION_ADD: i32 = 1;
pub static RTACTION_DEL: i32 = 2;
pub static RTACTION_HELP: i32 = 3;
pub static RTACTION_FLUSH: i32 = 4;
pub static RTACTION_SHOW: i32 = 5;

pub static FLAG_EXT: i32 = 3; /* AND-Mask */
pub static FLAG_NUM_HOST: i32 = 4;
pub static FLAG_NUM_PORT: i32 = 8;
pub static FLAG_NUM_USER: i32 = 16;
pub static FLAG_NUM: i32 = FLAG_NUM_HOST | FLAG_NUM_PORT | FLAG_NUM_USER;
pub static FLAG_SYM: i32 = 32;
pub static FLAG_CACHE: i32 = 64;
pub static FLAG_FIB: i32 = 128;
pub static FLAG_VERBOSE: i32 = 256;

static AF_TYPES: Lazy<Vec<AFType>> = Lazy::new(|| {
    vec![
        #[cfg(feature = "AFUNIX")]
        unix::UNIX_AFTYPE,
        #[cfg(feature = "AFINET")]
        inet::INET_AFTYPE,
        #[cfg(feature = "AFINET6")]
        inet6::INET6_AFTYPE,
        #[cfg(feature = "AFAX25")]
        ax25::AX25_AFTYPE,
        #[cfg(feature = "AFNETROM")]
        netrom::NETROM_AFTYPE,
        #[cfg(feature = "AFIPX")]
        ipx::IPX_AFTYPE,
        #[cfg(feature = "AFATALK")]
        ddp::DDP_AFTYPE,
        #[cfg(feature = "AFECONET")]
        econet::EC_AFTYPE,
        #[cfg(feature = "AFX25")]
        x25::X25_AFTYPE,
        #[cfg(feature = "AFROSE")]
        rose::ROSE_AFTYPE,
        #[cfg(feature = "AFASH")]
        ash::ASH_AFTYPE,
        unix::UNSPEC_AFTYPE,
    ]
});

static HW_TYPES: Lazy<Vec<HWType>> = Lazy::new(|| {
    vec![
        loopback::LOOP_HWTYPE,
        #[cfg(feature = "HWSLIP")]
        slip::SLIP_HWTYPE,
        #[cfg(feature = "HWSLIP")]
        slip::CSLIP_HWTYPE,
        #[cfg(feature = "HWSLIP")]
        slip::SLIP6_HWTYPE,
        #[cfg(feature = "HWSLIP")]
        slip::CSLIP6_HWTYPE,
        #[cfg(feature = "HWSLIP")]
        slip::ADAPTIVE_HWTYPE,
        #[cfg(feature = "HWSTRIP")]
        strip::STRIP_HWTYPE,
        #[cfg(feature = "HWASH")]
        ash::ASH_HWTYPE,
        #[cfg(feature = "HWETHER")]
        ether::ETHER_HWTYPE,
        #[cfg(feature = "HWTR")]
        tr::TR_HWTYPE,
        #[cfg(all(feature = "HWTR", feature = "ARPHRD_IEEE802_TR"))]
        tr::TR_HWTYPE1,
        #[cfg(feature = "HWAX25")]
        ax25::AX25_HWTYPE,
        #[cfg(feature = "HWNETROM")]
        netrom::NETROM_HWTYPE,
        #[cfg(feature = "HWROSE")]
        rose::ROSE_HWTYPE,
        #[cfg(feature = "HWTUNNEL")]
        tunnel::TUNNEL_HWTYPE,
        #[cfg(feature = "HWPPP")]
        ppp::PPP_HWTYPE,
        #[cfg(feature = "HWHDLCLAPB")]
        hdlclapb::HDLC_HWTYPE,
        #[cfg(feature = "HWHDLCLAPB")]
        hdlclapb::LAPB_HWTYPE,
        #[cfg(feature = "HWARC")]
        arcnet::ARCNET_HWTYPE,
        #[cfg(feature = "HWFR")]
        frame::DLCI_HWTYPE,
        #[cfg(feature = "HWFR")]
        frame::FRAD_HWTYPE,
        #[cfg(feature = "HWSIT")]
        sit::SIT_HWTYPE,
        #[cfg(feature = "HWFDDI")]
        fddi::FDDI_HWTYPE,
        #[cfg(feature = "HWHIPPI")]
        hippi::HIPPI_HWTYPE,
        #[cfg(feature = "HWIRDA")]
        irda::IRDA_HWTYPE,
        #[cfg(feature = "HWEC")]
        ec_hw::EC_HWTYPE,
        #[cfg(feature = "HWX25")]
        x25::X25_HWTYPE,
        #[cfg(feature = "HWIB")]
        ib::IB_HWTYPE,
        #[cfg(feature = "HWEUI64")]
        eui64::EUI64_HWTYPE,
        loopback::UNSPEC_HWTYPE,
    ]
});

type AFPrintFn = fn(ptr: Vec<i8>) -> String;
type AFSprintFn = fn(sasp: &sockaddr_storage, numeric: i32) -> Option<String>;
type AFInputFn = fn(typ: i32, bufp: &str, sasp: &mut sockaddr_storage) -> Result<i32, String>;
type AFHerrorFn = fn(ptr: &str);
type AFRprintFn = fn(options: i32) -> i32;
type AFRinputFn = fn(typ: i32, ext: i32, argv: *mut *mut libc::c_char) -> i32;
type AFGetmaskFn =
    fn(src: *mut libc::c_char, mask: &sockaddr_storage, name: *mut libc::c_char) -> i32;

/* This structure defines protocol families and their handlers. */
#[derive(Clone)]
pub struct AFType {
    pub name: &'static str,
    pub title: &'static str,
    pub af: i32,
    pub alen: i32,
    pub print: Option<AFPrintFn>,
    pub sprint: Option<AFSprintFn>,
    pub input: Option<AFInputFn>,
    pub herror: Option<AFHerrorFn>,
    pub rprint: Option<AFRprintFn>,
    pub rinput: Option<AFRinputFn>,
    pub getmask: Option<AFGetmaskFn>,
    pub fd: i32,
    pub flag_file: Option<&'static str>,
}

impl clap::builder::ValueParserFactory for AFType {
    type Parser = AFTypeValueParser;

    fn value_parser() -> Self::Parser {
        AFTypeValueParser
    }
}

#[derive(Clone)]
pub struct AFTypeValueParser;

impl clap::builder::TypedValueParser for AFTypeValueParser {
    type Value = AFType;

    fn parse_ref(
        &self,
        cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        clap::builder::TypedValueParser::parse(self, cmd, arg, value.to_owned())
    }

    fn parse(
        &self,
        _cmd: &clap::Command,
        _arg: Option<&clap::Arg>,
        value: std::ffi::OsString,
    ) -> Result<Self::Value, clap::Error> {
        let af_type = get_aftype(&value.to_string_lossy()).ok_or_else(|| {
            clap::Error::raw(
                clap::error::ErrorKind::ValueValidation,
                format!("{}: unknown address family.", value.to_string_lossy()),
            )
        })?;

        Ok(af_type.clone())
    }
}

type HWPrintFn = fn(ptr: Vec<i8>) -> String;
type HWInputFn = fn(bufp: &str, sasp: &mut sockaddr_storage) -> Result<(), i32>;
type HWActivateFn = fn(fd: i32) -> i32;

/* This structure defines hardware protocols and their handlers. */
#[derive(Clone)]
pub struct HWType {
    pub name: &'static str,
    pub title: &'static str,
    pub typ: i32,
    pub alen: i32,
    pub print: Option<HWPrintFn>,
    pub input: Option<HWInputFn>,
    pub activate: Option<HWActivateFn>,
    pub suppress_null_addr: i32,
}

impl clap::builder::ValueParserFactory for HWType {
    type Parser = HWTypeValueParser;

    fn value_parser() -> Self::Parser {
        HWTypeValueParser
    }
}

#[derive(Clone)]
pub struct HWTypeValueParser;

impl clap::builder::TypedValueParser for HWTypeValueParser {
    type Value = HWType;

    fn parse_ref(
        &self,
        cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        clap::builder::TypedValueParser::parse(self, cmd, arg, value.to_owned())
    }

    fn parse(
        &self,
        _cmd: &clap::Command,
        _arg: Option<&clap::Arg>,
        value: std::ffi::OsString,
    ) -> Result<Self::Value, clap::Error> {
        let hw_type = get_hwtype(&value.to_string_lossy()).ok_or_else(|| {
            clap::Error::raw(
                clap::error::ErrorKind::ValueValidation,
                format!("{}: unknown hardware family.", value.to_string_lossy()),
            )
        })?;

        Ok(hw_type.clone())
    }
}

pub struct NTContext {
    pub aftypes: Vec<AFType>,
    pub hwtypes: Vec<HWType>,
}

pub fn get_af_list_str(typ: i32) -> String {
    let mut af_list_string = String::new();
    let mut i = 0;
    for af in AF_TYPES.iter() {
        if (typ == 1 && af.rprint.is_none()) || af.af == 0 {
            continue;
        }
        if i % 3 == 0 {
            af_list_string.push_str("\n    ");
        }
        let name = match af.name {
            "" => "..",
            _ => af.name,
        };
        af_list_string.push_str(format!("{} ({}) ", name, af.title).as_str());
        i += 1;
    }
    af_list_string
}

pub fn get_aftype<'a>(name: &str) -> Option<&'a AFType> {
    AF_TYPES.iter().find(|&v| v.name == name)
}

pub fn get_afntype<'a>(typ: i32) -> Option<&'a AFType> {
    AF_TYPES.iter().find(|&v| v.af == typ)
}

pub fn get_hw_list_str(typ: i32) -> String {
    let mut hw_list_string = String::new();
    let mut i = 0;
    for hw in HW_TYPES.iter() {
        if (typ == 1 && hw.alen == 0) || hw.typ == -1 {
            continue;
        }
        if i % 3 == 0 {
            hw_list_string.push_str("\n    ");
        }
        let name = match hw.name {
            "" => "..",
            _ => hw.name,
        };
        hw_list_string.push_str(format!("{} ({}) ", name, hw.title).as_str());
        i += 1;
    }
    hw_list_string
}

pub fn get_hwtype<'a>(name: &str) -> Option<&'a HWType> {
    HW_TYPES.iter().find(|&v| v.name == name)
}

pub fn get_hwntype<'a>(typ: i32) -> Option<&'a HWType> {
    HW_TYPES.iter().find(|&v| v.typ == typ)
}

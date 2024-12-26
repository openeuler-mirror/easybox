//! This file is part of the easybox package.
//
// (c) Xu Biang <xubiang@foxmail.com>
// (c) Chen Yuchen <yuchen@isrc.iscas.ac.cn>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

// Protocols.
#[cfg(any(feature = "AFASH", feature = "HWASH"))]
pub mod ash;
#[cfg(any(feature = "AFAX25", feature = "HWAX25"))]
pub mod ax25;
#[cfg(feature = "AFATALK")]
pub mod ddp;
#[cfg(feature = "AFECONET")]
pub mod econet;
#[cfg(feature = "AFINET")]
pub mod inet;
#[cfg(feature = "AFINET6")]
pub mod inet6;
#[cfg(feature = "AFIPX")]
pub mod ipx;
#[cfg(any(feature = "AFNETROM", feature = "HWNETROM"))]
pub mod netrom;
#[cfg(any(feature = "AFROSE", feature = "HWROSE"))]
pub mod rose;
pub mod unix;
#[cfg(any(feature = "AFX25", feature = "HWX25"))]
pub mod x25;

// Hardwares.
#[cfg(feature = "HWARC")]
pub mod arcnet;
#[cfg(feature = "HWEC")]
pub mod ec_hw;
#[cfg(feature = "HWETHER")]
pub mod ether;
#[cfg(feature = "HWEUI64")]
pub mod eui64;
#[cfg(feature = "HWFDDI")]
pub mod fddi;
#[cfg(feature = "HWFR")]
pub mod frame;
#[cfg(feature = "HWHDLCLAPB")]
pub mod hdlclapb;
#[cfg(feature = "HWHIPPI")]
pub mod hippi;
#[cfg(feature = "HWIB")]
pub mod ib;
#[cfg(feature = "HWIRDA")]
pub mod irda;
pub mod loopback;
#[cfg(feature = "HWPPP")]
pub mod ppp;
#[cfg(feature = "HWSIT")]
pub mod sit;
#[cfg(feature = "HWSLIP")]
pub mod slip;
#[cfg(feature = "HWSTRIP")]
pub mod strip;
#[cfg(feature = "HWTR")]
pub mod tr;
#[cfg(feature = "HWTUNNEL")]
pub mod tunnel;

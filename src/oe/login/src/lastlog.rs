//! This file is part of the easybox package.
//
// (c) Jiale Xiao <xiao-xjle@qq.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use std::{cmp::min, mem::size_of};

const UT_LINESIZE: usize = 32;
const UT_HOSTSIZE: usize = 256;

const LL_LINE_OFFSET: usize = size_of::<u32>();
const LL_HOST_OFFSET: usize = LL_LINE_OFFSET + UT_LINESIZE;

/// Lastlog data structure
#[derive(Debug, Clone)]
pub struct Lastlog {
    /// Login time
    pub ll_time: u32,
    /// Login TTY info
    pub ll_line: String,
    /// Remote host info
    pub ll_host: String,
}

impl From<[u8; Lastlog::size()]> for Lastlog {
    /// Build a Lastlog from byte array
    fn from(buf: [u8; Lastlog::size()]) -> Self {
        let mut time_buf: [u8; size_of::<u32>()] = Default::default();
        time_buf.copy_from_slice(&buf[0..size_of::<u32>()]);
        Self {
            ll_time: match cfg!(target_endian = "big") {
                true => u32::from_be_bytes(time_buf),
                false => u32::from_le_bytes(time_buf),
            },
            ll_line: String::from_utf8_lossy(&buf[LL_LINE_OFFSET..LL_HOST_OFFSET])
                .trim_end_matches('\0')
                .to_string(),
            ll_host: String::from_utf8_lossy(&buf[LL_HOST_OFFSET..])
                .trim_end_matches('\0')
                .to_string(),
        }
    }
}

impl Lastlog {
    /// Lastlog structure length
    pub const fn size() -> usize {
        return size_of::<u32>() + UT_HOSTSIZE + UT_LINESIZE;
    }

    /// Dump Lastlog into byte array
    pub fn as_bytes(self) -> [u8; Lastlog::size()] {
        let mut res = [0 as u8; Lastlog::size()];
        let tb = match cfg!(target_endian = "big") {
            true => self.ll_time.to_be_bytes(),
            false => self.ll_time.to_le_bytes(),
        };
        res[0..LL_LINE_OFFSET].copy_from_slice(&tb);

        let line_a = self.ll_line.as_bytes();
        let line_b = &line_a[0..min(UT_LINESIZE, line_a.len())];
        res[LL_LINE_OFFSET..LL_LINE_OFFSET + line_b.len()].copy_from_slice(line_b);

        let host_a = self.ll_host.as_bytes();
        let host_b = &host_a[0..min(UT_HOSTSIZE, host_a.len())];
        res[LL_HOST_OFFSET..LL_HOST_OFFSET + host_b.len()].copy_from_slice(host_b);

        res
    }
}

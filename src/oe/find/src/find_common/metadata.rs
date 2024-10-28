//! This file is part of the easybox package.
//
// (c) Xing Huang <navihx@foxmail.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use uucore::error::UResult;

use super::{get_gname_by_gid, get_uname_by_uid};

#[derive(Clone, Default)]

///
pub struct ForgeMetadata {
    ///
    pub mode: u32,

    ///
    pub atime: i64,

    ///
    pub mtime: i64,

    ///
    pub ctime: i64,

    ///
    pub len: u64,

    ///
    pub ino: u64,

    ///
    pub block: u64,

    ///
    pub blksize: u64,

    ///
    pub dev: u64,

    ///
    pub nlink: u64,

    ///
    pub uid: u32,

    ///
    pub uname: String,

    ///
    pub gid: u32,

    ///
    pub gname: String,
}

impl ForgeMetadata {
    ///
    pub fn new_forge() -> UResult<Self> {
        Ok(Self::default())
    }
}

impl FindMetadata for ForgeMetadata {
    fn st_mode(&self) -> u32 {
        self.mode
    }
    fn st_atime(&self) -> i64 {
        self.atime
    }
    fn st_mtime(&self) -> i64 {
        self.mtime
    }
    fn st_ctime(&self) -> i64 {
        self.ctime
    }
    fn st_len(&self) -> u64 {
        self.len
    }
    fn st_ino(&self) -> u64 {
        self.ino
    }
    fn st_block(&self) -> u64 {
        self.block
    }
    fn st_blksize(&self) -> u64 {
        self.blksize
    }
    fn st_dev(&self) -> u64 {
        self.dev
    }
    fn st_nlink(&self) -> u64 {
        self.nlink
    }
    fn st_uid(&self) -> u32 {
        self.uid
    }
    fn st_uname(&self) -> Option<String> {
        Some(self.uname.clone())
    }
    fn st_gid(&self) -> u32 {
        self.gid
    }
    fn st_gname(&self) -> Option<String> {
        Some(self.gname.clone())
    }
}

///
pub trait FindMetadata {
    ///
    fn st_mode(&self) -> u32;

    ///
    fn st_atime(&self) -> i64;

    ///
    fn st_mtime(&self) -> i64;

    ///
    fn st_ctime(&self) -> i64;

    ///
    fn st_len(&self) -> u64;

    ///
    fn st_ino(&self) -> u64;

    ///
    fn st_block(&self) -> u64;

    ///
    fn st_blksize(&self) -> u64;

    ///
    fn st_dev(&self) -> u64;

    ///
    fn st_nlink(&self) -> u64;

    ///
    fn st_uid(&self) -> u32;

    ///
    fn st_uname(&self) -> Option<String>;

    ///
    fn st_gid(&self) -> u32;

    ///
    fn st_gname(&self) -> Option<String>;
}

impl<M: std::os::linux::fs::MetadataExt> FindMetadata for M {
    fn st_mode(&self) -> u32 {
        self.st_mode()
    }

    fn st_atime(&self) -> i64 {
        self.st_atime()
    }

    fn st_mtime(&self) -> i64 {
        self.st_mtime()
    }

    fn st_ctime(&self) -> i64 {
        self.st_ctime()
    }

    fn st_len(&self) -> u64 {
        self.st_size()
    }

    fn st_ino(&self) -> u64 {
        self.st_ino()
    }

    fn st_block(&self) -> u64 {
        self.st_blocks()
    }

    fn st_blksize(&self) -> u64 {
        self.st_blksize()
    }

    fn st_dev(&self) -> u64 {
        self.st_dev()
    }

    fn st_nlink(&self) -> u64 {
        self.st_nlink()
    }

    fn st_uid(&self) -> u32 {
        self.st_uid()
    }

    fn st_uname(&self) -> Option<String> {
        get_uname_by_uid(self.st_uid())
    }

    fn st_gid(&self) -> u32 {
        self.st_gid()
    }

    fn st_gname(&self) -> Option<String> {
        get_gname_by_gid(self.st_gid())
    }
}

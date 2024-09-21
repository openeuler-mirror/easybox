//! This file is part of the easybox package.
//
// (c) Wentong Yang <ywt0821@163.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use libc::AT_FDCWD;
use nix::dir::Dir;
use nix::fcntl::{openat, readlink, AtFlags, OFlag};
use nix::sys::stat::{
    fchmodat, fstatat, mkdirat, mknodat, utimensat, FchmodatFlags, FileStat, Mode, SFlag,
    UtimensatFlags,
};
use nix::sys::time::TimeSpec;
use nix::unistd::{close, fchown, fchownat, linkat, symlinkat, FchownatFlags, LinkatFlags};
use nix::unistd::{Gid, Uid};
use std::collections::LinkedList;
use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::io::{FromRawFd, RawFd};
use uucore::error::{UResult, USimpleError};

/// LinkName structure
#[derive(Clone)]
pub struct LinkName {
    ///
    pub ln_dev: u64,
    ///
    pub ln_ino: u64,
    ///
    pub ln_count: u64,
    ///
    pub ln_name: String,
}

///
pub struct CopyContext {
    ///
    pub src_orig: Option<String>,
    ///
    pub dst_orig: Option<String>,
    ///
    pub links: LinkedList<LinkName>,
}

///
impl CopyContext {
    ///
    pub fn remove_link(&mut self, ln_dev: u64, ln_ino: u64) {
        let mut new_links = LinkedList::new();
        for link in self.links.iter() {
            if link.ln_dev != ln_dev || link.ln_ino != ln_ino {
                new_links.push_back(link.clone());
            }
        }
        self.links = new_links;
    }
}

/// PathInfo structure
pub struct PathInfo {
    ///
    full_path: String,
    ///
    dirfd: RawFd,
    ///
    name: String,
}

/// Copy a directory tree
pub fn copy_tree(
    src_root: &str,
    dst_root: &str,
    copy_root: bool,
    reset_selinux: bool,
    old_uid: Option<u32>,
    new_uid: Option<u32>,
    old_gid: Option<u32>,
    new_gid: Option<u32>,
) -> UResult<()> {
    let src = PathInfo {
        full_path: src_root.to_string(),
        dirfd: AT_FDCWD,
        name: src_root.to_string(),
    };
    let dst = PathInfo {
        full_path: dst_root.to_string(),
        dirfd: AT_FDCWD,
        name: dst_root.to_string(),
    };

    let mut context = CopyContext {
        src_orig: None,
        dst_orig: None,
        links: LinkedList::new(),
    };

    copy_tree_impl(
        &src,
        &dst,
        copy_root,
        reset_selinux,
        old_uid,
        new_uid,
        old_gid,
        new_gid,
        &mut context,
    )
}

/// Copy a directory tree
pub fn copy_tree_impl(
    src: &PathInfo,
    dst: &PathInfo,
    copy_root: bool,
    reset_selinux: bool,
    old_uid: Option<u32>,
    new_uid: Option<u32>,
    old_gid: Option<u32>,
    new_gid: Option<u32>,
    context: &mut CopyContext,
) -> UResult<()> {
    let mut set_orig = false;

    if copy_root {
        let _sb = fstatat(src.dirfd, src.name.as_str(), AtFlags::empty())
            .map_err(|e| USimpleError::new(-1, format!("Failed to stat source: {}", e)))?;

        let sb = fstatat(dst.dirfd, dst.name.as_str(), AtFlags::AT_SYMLINK_NOFOLLOW)
            .map_err(|e| USimpleError::new(-1, format!("Failed to stat destination: {}", e)))?;

        if !SFlag::from_bits_truncate(sb.st_mode).contains(SFlag::S_IFDIR) {
            return Err(
                USimpleError::new(-1, format!("{} is not a directory", src.full_path)).into(),
            );
        }

        return copy_entry(
            src,
            dst,
            reset_selinux,
            old_uid,
            new_uid,
            old_gid,
            new_gid,
            context,
        );
    }

    let src_fd = openat(
        src.dirfd,
        src.name.as_str(),
        OFlag::O_DIRECTORY | OFlag::O_RDONLY | OFlag::O_NOFOLLOW | OFlag::O_CLOEXEC,
        Mode::empty(),
    )
    .map_err(|e| USimpleError::new(-1, format!("Failed to open source directory: {}", e)))?;

    let dst_fd = openat(
        dst.dirfd,
        dst.name.as_str(),
        OFlag::O_DIRECTORY | OFlag::O_RDONLY | OFlag::O_NOFOLLOW | OFlag::O_CLOEXEC,
        Mode::empty(),
    )
    .map_err(|e| {
        close(src_fd).ok();
        USimpleError::new(-1, format!("Failed to open destination directory: {}", e))
    })?;

    let dir = Dir::from_fd(src_fd).map_err(|e| {
        close(src_fd).ok();
        close(dst_fd).ok();
        USimpleError::new(-1, format!("Failed to open directory stream: {}", e))
    })?;

    if context.src_orig.is_none() {
        context.src_orig = Some(src.full_path.clone());
        context.dst_orig = Some(dst.full_path.clone());
        set_orig = true;
    }

    for entry in dir {
        let entry = entry
            .map_err(|e| USimpleError::new(1, format!("Failed to read directory entry: {}", e)))?;

        let entry_name = entry.file_name().to_string_lossy();
        if entry_name == "." || entry_name == ".." {
            continue;
        }

        let src_name = format!("{}/{}", src.full_path, entry_name);
        let dst_name = format!("{}/{}", dst.full_path, entry_name);

        if src_name.is_empty() || dst_name.is_empty() {
            continue;
        }

        let src_entry = PathInfo {
            full_path: src_name.clone(),
            dirfd: src_fd,
            name: entry_name.to_string(),
        };
        let dst_entry = PathInfo {
            full_path: dst_name.clone(),
            dirfd: dst_fd,
            name: entry_name.to_string(),
        };

        copy_entry(
            &src_entry,
            &dst_entry,
            reset_selinux,
            old_uid,
            new_uid,
            old_gid,
            new_gid,
            context,
        )?;
    }

    close(src_fd).ok();
    close(dst_fd).ok();

    if set_orig {
        context.src_orig = None;
        context.dst_orig = None;
    }

    Ok(())
}

/// Copy a file or directory entry
pub fn copy_entry(
    src: &PathInfo,
    dst: &PathInfo,
    reset_selinux: bool,
    old_uid: Option<u32>,
    new_uid: Option<u32>,
    old_gid: Option<u32>,
    new_gid: Option<u32>,
    context: &mut CopyContext,
) -> UResult<()> {
    let sb = fstatat(src.dirfd, src.name.as_str(), AtFlags::AT_SYMLINK_NOFOLLOW)
        .map_err(|e| USimpleError::new(1, format!("Failed to stat source entry: {}", e)))?;

    let mt = [
        TimeSpec::new(sb.st_atime, sb.st_atime_nsec),
        TimeSpec::new(sb.st_mtime, sb.st_mtime_nsec),
    ];

    if SFlag::from_bits_truncate(sb.st_mode).contains(SFlag::S_IFDIR) {
        copy_dir(
            src,
            dst,
            reset_selinux,
            &sb,
            &mt,
            old_uid,
            new_uid,
            old_gid,
            new_gid,
            context,
        )?;
    } else {
        /*
         * If the destination already exists do nothing.
         * This is after the copy_dir above to still iterate into subdirectories.
         */
        if fstatat(dst.dirfd, dst.name.as_str(), AtFlags::AT_SYMLINK_NOFOLLOW).is_ok() {
            return Ok(());
        }

        if SFlag::from_bits_truncate(sb.st_mode).contains(SFlag::S_IFLNK) {
            copy_symlink(
                src,
                dst,
                reset_selinux,
                &sb,
                &mt,
                old_uid,
                new_uid,
                old_gid,
                new_gid,
                context,
            )?;
        } else if let Some(lp) = check_link(&src.full_path, &sb, context) {
            copy_hardlink(dst, reset_selinux, lp, context)?;
        } else if !SFlag::from_bits_truncate(sb.st_mode).contains(SFlag::S_IFREG) {
            copy_special(
                src,
                dst,
                reset_selinux,
                &sb,
                &mt,
                old_uid,
                new_uid,
                old_gid,
                new_gid,
            )?;
        } else {
            copy_file(
                src,
                dst,
                reset_selinux,
                &sb,
                &mt,
                old_uid,
                new_uid,
                old_gid,
                new_gid,
            )?;
        }
    }
    Ok(())
}

/// Copy a directory
pub fn copy_dir(
    src: &PathInfo,
    dst: &PathInfo,
    reset_selinux: bool,
    statp: &FileStat,
    mt: &[TimeSpec; 2],
    old_uid: Option<u32>,
    new_uid: Option<u32>,
    old_gid: Option<u32>,
    new_gid: Option<u32>,
    context: &mut CopyContext,
) -> UResult<()> {
    /*
     * If the destination is already a directory, don't change it
     * but copy into it (recursively).
     */
    if let Ok(dst_sb) = fstatat(dst.dirfd, dst.name.as_str(), AtFlags::AT_SYMLINK_NOFOLLOW) {
        if SFlag::from_bits_truncate(dst_sb.st_mode).contains(SFlag::S_IFDIR) {
            return copy_tree_impl(
                src,
                dst,
                false,
                reset_selinux,
                old_uid,
                new_uid,
                old_gid,
                new_gid,
                context,
            );
        }
    }

    mkdirat(
        dst.dirfd,
        dst.name.as_str(),
        Mode::from_bits_truncate(0o700),
    )
    .map_err(|e| USimpleError::new(-1, format!("Failed to create directory: {}", e)))?;

    chownat_if_needed(dst, statp, old_uid, new_uid, old_gid, new_gid)?;

    fchmodat(
        Some(dst.dirfd),
        dst.name.as_str(),
        Mode::from_bits_truncate(statp.st_mode & 0o7777),
        FchmodatFlags::NoFollowSymlink,
    )
    .map_err(|e| USimpleError::new(-1, format!("Failed to chmod directory: {}", e)))?;

    copy_tree_impl(
        src,
        dst,
        false,
        reset_selinux,
        old_uid,
        new_uid,
        old_gid,
        new_gid,
        context,
    )?;

    nix::sys::stat::utimensat(
        Some(dst.dirfd),
        dst.name.as_str(),
        &mt[0].into(),
        &mt[1].into(),
        nix::sys::stat::UtimensatFlags::NoFollowSymlink,
    )
    .map_err(|e| USimpleError::new(-1, format!("Failed to change timestamps: {}", e)))?;

    Ok(())
}

///
pub fn chownat_if_needed(
    dst: &PathInfo,
    statp: &FileStat,
    old_uid: Option<u32>,
    new_uid: Option<u32>,
    old_gid: Option<u32>,
    new_gid: Option<u32>,
) -> UResult<()> {
    let tmpuid = if old_uid.is_none() || old_uid == Some(statp.st_uid) {
        new_uid.unwrap_or(statp.st_uid)
    } else {
        statp.st_uid
    };

    let tmpgid = if old_gid.is_none() || old_gid == Some(statp.st_gid) {
        new_gid.unwrap_or(statp.st_gid)
    } else {
        statp.st_gid
    };
    fchownat(
        Some(dst.dirfd),
        dst.name.as_str(),
        Some(Uid::from_raw(tmpuid)),
        Some(Gid::from_raw(tmpgid)),
        FchownatFlags::NoFollowSymlink,
    )
    .map_err(|e| USimpleError::new(-1, format!("Failed to change ownership: {}", e)))?;

    Ok(())
}

///
pub fn copy_symlink(
    src: &PathInfo,
    dst: &PathInfo,
    _reset_selinux: bool,
    statp: &FileStat,
    mt: &[TimeSpec; 2],
    old_uid: Option<u32>,
    new_uid: Option<u32>,
    old_gid: Option<u32>,
    new_gid: Option<u32>,
    context: &mut CopyContext,
) -> UResult<()> {
    assert!(context.src_orig.is_some());
    assert!(context.dst_orig.is_some());

    let src_orig = context.src_orig.as_deref().unwrap();
    let dst_orig = context.dst_orig.as_deref().unwrap();

    let oldlink = readlink(src.full_path.as_str())
        .map_err(|e| USimpleError::new(-1, format!("Failed to readlink: {}", e)))?;

    let oldlink_str = oldlink.into_string().map_err(|e| {
        USimpleError::new(-1, format!("Failed to convert OsString to String: {:?}", e))
    })?;

    let adjusted_link = if oldlink_str.starts_with(src_orig) {
        let mut newlink = dst_orig.to_string();
        newlink.push_str(&oldlink_str[src_orig.len()..]);
        newlink
    } else {
        oldlink_str
    };

    symlinkat(adjusted_link.as_str(), Some(dst.dirfd), dst.name.as_str())
        .map_err(|e| USimpleError::new(-1, format!("Failed to create symlink: {}", e)))?;

    chownat_if_needed(dst, statp, old_uid, new_uid, old_gid, new_gid)?;

    nix::sys::stat::utimensat(
        Some(dst.dirfd),
        dst.name.as_str(),
        &mt[0].into(),
        &mt[1].into(),
        nix::sys::stat::UtimensatFlags::NoFollowSymlink,
    )
    .map_err(|e| USimpleError::new(-1, format!("Failed to change timestamps: {}", e)))?;

    Ok(())
}

///
pub fn check_link(name: &str, sb: &FileStat, context: &mut CopyContext) -> Option<LinkName> {
    assert!(context.src_orig.is_some());
    assert!(context.dst_orig.is_some());

    let src_orig = context.src_orig.as_deref().unwrap();
    let dst_orig = context.dst_orig.as_deref().unwrap();

    for lp in &context.links {
        if lp.ln_dev == sb.st_dev as u64 && lp.ln_ino == sb.st_ino as u64 {
            return Some(lp.clone());
        }
    }

    if sb.st_nlink == 1 {
        return None;
    }

    let ln_name = format!("{}{}", dst_orig, &name[src_orig.len()..]);
    let link = LinkName {
        ln_dev: sb.st_dev as u64,
        ln_ino: sb.st_ino as u64,
        ln_count: sb.st_nlink as u64,
        ln_name,
    };

    context.links.push_back(link.clone());

    None
}

///
pub fn copy_hardlink(
    dst: &PathInfo,
    _reset_selinux: bool,
    lp: LinkName,
    context: &mut CopyContext,
) -> UResult<()> {
    linkat(
        Some(AT_FDCWD),
        lp.ln_name.as_str(),
        Some(dst.dirfd),
        &dst.name,
        LinkatFlags::NoSymlinkFollow,
    )
    .map_err(|e| USimpleError::new(-1, format!("Failed to create hardlink: {}", e)))?;

    // If the file could be unlinked, decrement the links counter,
    // and forget about this link if it was the last reference
    if lp.ln_count > 1 {
        let new_lp = LinkName {
            ln_dev: lp.ln_dev,
            ln_ino: lp.ln_ino,
            ln_count: lp.ln_count - 1,
            ln_name: lp.ln_name.clone(),
        };

        for link in context.links.iter_mut() {
            if link.ln_dev == lp.ln_dev && link.ln_ino == lp.ln_ino {
                *link = new_lp.clone();
            }
        }

        if new_lp.ln_count == 0 {
            context.remove_link(lp.ln_dev, lp.ln_ino);
        }
    }

    Ok(())
}

///
pub fn copy_special(
    _src: &PathInfo,
    dst: &PathInfo,
    _reset_selinux: bool,
    statp: &FileStat,
    mt: &[TimeSpec; 2],
    old_uid: Option<u32>,
    new_uid: Option<u32>,
    old_gid: Option<u32>,
    new_gid: Option<u32>,
) -> UResult<()> {
    mknodat(
        dst.dirfd,
        dst.name.as_str(),
        SFlag::from_bits_truncate(statp.st_mode) & SFlag::S_IFMT,
        Mode::from_bits_truncate(statp.st_mode & 0o7777),
        statp.st_rdev,
    )
    .map_err(|e| USimpleError::new(-1, format!("Failed to create special file: {}", e)))?;

    chownat_if_needed(dst, statp, old_uid, new_uid, old_gid, new_gid)?;

    fchmodat(
        Some(dst.dirfd),
        dst.name.as_str(),
        Mode::from_bits_truncate(statp.st_mode & 0o7777),
        FchmodatFlags::NoFollowSymlink,
    )
    .map_err(|e| USimpleError::new(-1, format!("Failed to change file permissions: {}", e)))?;

    utimensat(
        Some(dst.dirfd),
        dst.name.as_str(),
        &mt[0].into(),
        &mt[1].into(),
        UtimensatFlags::NoFollowSymlink,
    )
    .map_err(|e| USimpleError::new(-1, format!("Failed to change timestamps: {}", e)))?;

    Ok(())
}

///
pub fn copy_file(
    src: &PathInfo,
    dst: &PathInfo,
    _reset_selinux: bool,
    statp: &FileStat,
    mt: &[TimeSpec; 2],
    _old_uid: Option<u32>,
    new_uid: Option<u32>,
    _old_gid: Option<u32>,
    new_gid: Option<u32>,
) -> UResult<()> {
    // Open source file
    let ifd = openat(
        src.dirfd,
        src.name.as_str(),
        OFlag::O_RDONLY | OFlag::O_NOFOLLOW | OFlag::O_CLOEXEC,
        Mode::empty(),
    )
    .map_err(|e| USimpleError::new(-1, format!("Failed to open source file: {}", e)))?;

    let mut src_file = unsafe { File::from_raw_fd(ifd) };

    let ofd = openat(
        dst.dirfd,
        dst.name.as_str(),
        OFlag::O_WRONLY
            | OFlag::O_CREAT
            | OFlag::O_EXCL
            | OFlag::O_TRUNC
            | OFlag::O_NOFOLLOW
            | OFlag::O_CLOEXEC,
        Mode::from_bits_truncate(0o600),
    )
    .map_err(|e| {
        close(ifd).ok();
        USimpleError::new(-1, format!("Failed to open destination file: {}", e))
    })?;

    let mut dst_file = unsafe { File::from_raw_fd(ofd) };

    if fchown(
        ofd,
        Some(Uid::from_raw(new_uid.unwrap_or(statp.st_uid))),
        Some(Gid::from_raw(new_gid.unwrap_or(statp.st_gid))),
    )
    .is_err()
        || fchmodat(
            Some(dst.dirfd),
            dst.name.as_str(),
            Mode::from_bits_truncate(statp.st_mode & 0o7777),
            FchmodatFlags::NoFollowSymlink,
        )
        .is_err()
    {
        close(ifd).ok();
        close(ofd).ok();
        return Err(USimpleError::new(-1, "Failed to set ownership or permissions").into());
    }

    let mut buffer = [0u8; 8192];
    loop {
        let count = src_file.read(&mut buffer).map_err(|e| {
            close(ifd).ok();
            close(ofd).ok();
            USimpleError::new(-1, format!("Failed to read file: {}", e))
        })?;
        if count == 0 {
            break;
        }
        dst_file.write_all(&buffer[..count]).map_err(|e| {
            close(ifd).ok();
            close(ofd).ok();
            USimpleError::new(-1, format!("Failed to write file: {}", e))
        })?;
    }

    dst_file
        .flush()
        .map_err(|e| USimpleError::new(-1, format!("Failed to flush destination file: {}", e)))?;

    drop(src_file);
    drop(dst_file);

    utimensat(
        Some(dst.dirfd),
        dst.name.as_str(),
        &mt[0].into(),
        &mt[1].into(),
        UtimensatFlags::NoFollowSymlink,
    )
    .map_err(|e| USimpleError::new(-1, format!("Failed to update file timestamps: {}", e)))?;

    Ok(())
}

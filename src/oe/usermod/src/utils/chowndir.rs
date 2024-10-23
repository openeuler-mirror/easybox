//! This file is part of the easybox package.
//
// (c) Wentong Yang <ywt0821@163.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use nix::dir::Dir;
use nix::fcntl::AtFlags;
use nix::fcntl::{openat, OFlag};
use nix::libc::AT_FDCWD;
use nix::sys::stat::{fstat, fstatat, Mode, SFlag};
use nix::unistd::{fchown, fchownat, FchownatFlags, Gid, Uid};
use std::os::unix::io::RawFd;

///
pub fn chown_tree_at(
    dir_fd: RawFd,
    path: &str,
    old_uid: Option<Uid>,
    new_uid: Option<Uid>,
    old_gid: Option<Gid>,
    new_gid: Option<Gid>,
) -> Result<(), nix::Error> {
    let dir_fd = openat(
        dir_fd,
        path,
        OFlag::O_RDONLY | OFlag::O_DIRECTORY | OFlag::O_NOFOLLOW | OFlag::O_CLOEXEC,
        Mode::empty(),
    )?;

    let mut dir = Dir::from_fd(dir_fd)?;

    for entry in dir.iter() {
        let entry = entry?;
        let entry_name = entry.file_name().to_str().unwrap();

        if entry_name == "." || entry_name == ".." {
            continue;
        }
        let entry_stat = fstatat(dir_fd, entry_name, AtFlags::AT_SYMLINK_NOFOLLOW)?;
        if SFlag::from_bits_truncate(entry_stat.st_mode).contains(SFlag::S_IFDIR) {
            chown_tree_at(dir_fd, entry_name, old_uid, new_uid, old_gid, new_gid)?;
        }

        let mut tmpuid = None;
        let mut tmpgid = None;
        if old_uid
            .map(|uid| uid == entry_stat.st_uid.into())
            .unwrap_or(true)
        {
            tmpuid = new_uid;
        }
        if old_gid
            .map(|gid| gid == entry_stat.st_gid.into())
            .unwrap_or(true)
        {
            tmpgid = new_gid;
        }

        if tmpuid.is_some() || tmpgid.is_some() {
            fchownat(
                Some(dir_fd),
                entry_name,
                tmpuid,
                tmpgid,
                FchownatFlags::NoFollowSymlink,
            )?;
        }
    }

    let dir_stat = fstat(dir_fd)?;
    let mut tmpuid = None;
    let mut tmpgid = None;
    if old_uid
        .map(|uid| uid == dir_stat.st_uid.into())
        .unwrap_or(true)
    {
        tmpuid = new_uid;
    }
    if old_gid
        .map(|gid| gid == dir_stat.st_gid.into())
        .unwrap_or(true)
    {
        tmpgid = new_gid;
    }

    if tmpuid.is_some() || tmpgid.is_some() {
        fchown(dir_fd, tmpuid, tmpgid)?;
    }

    Ok(())
}

///
pub fn chown_tree(
    root: &str,
    old_uid: Option<Uid>,
    new_uid: Option<Uid>,
    old_gid: Option<Gid>,
    new_gid: Option<Gid>,
) -> Result<(), nix::Error> {
    chown_tree_at(AT_FDCWD, root, old_uid, new_uid, old_gid, new_gid)
}

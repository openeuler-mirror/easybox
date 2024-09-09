use crate::error::{UResult, USimpleError};
use nix::errno::Errno;
use nix::mount::{umount, umount2, MntFlags};
use nix::unistd::Uid;
use std::path::Path;
///
pub fn umount_fs<P: AsRef<Path> + std::fmt::Display>(
    target: P,
    flags: MntFlags,
    internal_only: bool,
) -> nix::Result<()> {
    if !nix::unistd::geteuid().is_root() {
        eprintln!("umount: {}: must be superuser to unmount.", target);
        return Ok(());
    }
    let result = if flags == MntFlags::empty() {
        umount(target.as_ref())
    } else {
        umount2(target.as_ref(), flags)
    };
    if internal_only {
        result
    } else {
        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                eprintln!("Internal mount failed: {}. Attempting external mount...", e);
                internal_umount(&target, flags)
            }
        }
    }
}

fn internal_umount<P: AsRef<Path>>(target: &P, flags: MntFlags) -> nix::Result<()> {
    let mut cmd = std::process::Command::new("umount");

    // Add flags
    if flags.contains(MntFlags::MNT_FORCE) {
        cmd.arg("-f");
    }
    if flags.contains(MntFlags::MNT_DETACH) {
        cmd.arg("-l");
    }
    // More flag conversions can be added as needed

    cmd.arg(target.as_ref());

    match cmd.status() {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => Err(Errno::from_i32(status.code().unwrap_or(1))),
        Err(e) => Err(Errno::from_i32(e.raw_os_error().unwrap_or(1))),
    }
}
///
pub fn prepare_umount_target(_target: &str) -> UResult<String> {
    if !Uid::effective().is_root() {
        return Err(USimpleError::new(
            1,
            "Root privileges are required to unmount devices",
        ));
    } else {
        Ok("".to_string())
    }
}

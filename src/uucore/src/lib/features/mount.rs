//! This file is part of the easybox package.
//
// (c) Zhenghang <2113130664@qq.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use crate::error::{UResult, USimpleError};
use nix::errno::Errno;
use nix::mount::{mount, MsFlags};
use nix::unistd::Uid;
use regex::Regex;
use std::fs::File;
use std::io::{self};
use std::io::{BufRead, BufReader};
use std::os::unix::fs::FileTypeExt;
use std::path::Path;
///
pub fn mount_fs<P: AsRef<Path>>(
    source: Option<&P>,
    target: &P,
    fs_type: Option<&str>,
    flags: MsFlags,
    data: Option<&str>,
    internal_only: bool,
) -> nix::Result<()> {
    let result = mount(
        source.map(|s| s.as_ref()),
        target.as_ref(),
        fs_type,
        flags,
        data,
    );
    if internal_only {
        // If internal_only is specified, we only return the result of the kernel mount
        result
    } else {
        match result {
            Ok(_) => Ok(()),
            //Internal mount successful
            Err(e) => {
                eprintln!("Internal mount failed: {}. Attempting external mount...", e);
                // Attempt external mount
                external_mount(source, target, fs_type, flags, data)
            }
        }
    }
}
fn external_mount<P: AsRef<Path>>(
    source: Option<&P>,
    target: &P,
    fs_type: Option<&str>,
    flags: MsFlags,
    data: Option<&str>,
) -> nix::Result<()> {
    let mut cmd = std::process::Command::new("mount");

    if let Some(src) = source {
        cmd.arg(src.as_ref());
    }

    cmd.arg(target.as_ref());

    if let Some(fs) = fs_type {
        cmd.args(&["-t", fs]);
    }

    // Convert flags to command line options
    if flags.contains(MsFlags::MS_RDONLY) {
        cmd.arg("-r");
    }
    if flags.contains(MsFlags::MS_NOSUID) {
        cmd.arg("-o").arg("nosuid");
    }
    if flags.contains(MsFlags::MS_NODEV) {
        cmd.arg("-o").arg("nodev");
    }
    if flags.contains(MsFlags::MS_NOEXEC) {
        cmd.arg("-o").arg("noexec");
    }
    // Add more flag conversions as needed

    if let Some(d) = data {
        cmd.arg("-o").arg(d);
    }

    match cmd.status() {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => Err(Errno::from_i32(status.code().unwrap_or(1))),
        Err(e) => Err(Errno::from_i32(e.raw_os_error().unwrap_or(1))),
    }
}
///
pub fn prepare_mount_source(source: &str) -> UResult<String> {
    if !Uid::effective().is_root() {
        return Err(USimpleError::new(
            1,
            "Root privileges are required to mount devices",
        ));
    }
    let metadata = std::fs::metadata(source).map_err(|e| {
        USimpleError::new(1, format!("Unable to get source file information: {}", e))
    })?;
    if metadata.file_type().is_block_device() {
        // Return block device directly
        Ok(source.to_string())
    } else {
        // Create loop device for regular files
        let output = std::process::Command::new("losetup")
            .arg("-f")
            .arg("--show")
            .arg(source)
            .output()
            .map_err(|e| USimpleError::new(1, format!("Failed to create loop device: {}", e)))?;
        if !output.status.success() {
            Err(USimpleError::new(
                1,
                format!(
                    "Failed to create loop device: {}",
                    String::from_utf8_lossy(&output.stderr)
                )
                .to_string(),
            ))
        } else {
            String::from_utf8(output.stdout)
                .map_err(|e| {
                    USimpleError::new(1, format!("Failed to parse loop device path: {}", e))
                })
                .map(|s| s.trim().to_string())
        }
    }
}
///
pub fn is_already_mounted(target: &str) -> Result<bool, Box<dyn std::error::Error>> {
    /* Read /proc/mounts to get mounted device mount points, determine if already mounted */
    let file = File::open("/proc/mounts")?;
    let reader = BufReader::new(file);
    let re = Regex::new(r"^\S+\s+(\S+)")?;
    for line in reader.lines() {
        let line = line?;
        if let Some(caps) = re.captures(&line) {
            if let Some(mount_point) = caps.get(1) {
                if target == mount_point.as_str() {
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}
///
pub fn is_swapfile(fstype: &str) -> bool {
    fstype == "swap"
}
///
pub fn parse_mount_options(_options: &str) -> MsFlags {
    let flags = MsFlags::empty();
    // for option in options.split(',') {
    //     // match option {
    //     //     "noexec" => flags |= MsFlags::MS_NOEXEC,
    //     //     "nosuid" => flags |= MsFlags::MS_NOSUID,
    //     //     // Add other options...
    //     //     _ => {}
    //     // }
    // }
    flags
}
///
pub fn parse_fstab(path: &str) -> Result<Vec<Vec<String>>, Box<dyn std::error::Error>> {
    let path = Path::new(path);
    let file = File::open(path).map_err(|e| format!("Failed to open fstab file: {}", e))?;
    let reader = BufReader::new(file);
    let re = Regex::new(r"^(\S+)\s+(\S+)\s+(\S+)\s+(\S+)\s+(\d+)\s+(\d+)").unwrap();

    let mut fstab_vec = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        let line = line.map_err(|e| format!("Error reading line {}: {}", index + 1, e))?;
        let trimmed = line.trim();
        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue; // Skip comments and empty lines
        }
        if let Some(caps) = re.captures(trimmed) {
            let line_vec: Vec<String> = (1..=6).map(|i| caps[i].to_string()).collect();
            fstab_vec.push(line_vec);
        } else {
            eprintln!(
                "Warning: Line {} does not match expected format: {}",
                index + 1,
                trimmed
            );
        }
    }

    if fstab_vec.is_empty() {
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "No valid entries found in fstab file",
        )))
    } else {
        Ok(fstab_vec)
    }
}
///
pub fn find_device_by_label(label: &str) -> Result<String, Box<dyn std::error::Error>> {
    let output = std::process::Command::new("blkid")
        .arg("-L")
        .arg(label)
        .output()?;

    if output.status.success() {
        let device = String::from_utf8(output.stdout)?.trim().to_string();
        Ok(device)
    } else {
        Err(io::Error::new(io::ErrorKind::NotFound, "Not found device by label").into())
    }
}
///
pub fn find_device_by_uuid(uuid: &str) -> Result<String, Box<dyn std::error::Error>> {
    let output = std::process::Command::new("blkid")
        .arg("-U")
        .arg(uuid)
        .output()?;

    if output.status.success() {
        let device = String::from_utf8(output.stdout)?.trim().to_string();
        Ok(device)
    } else {
        Err(io::Error::new(io::ErrorKind::NotFound, "Not found device by uuid").into())
    }
}
///Check if the path is a mount point
pub fn is_mount_point(path: &str) -> bool {
    let file = match File::open("/proc/mounts") {
        Ok(f) => f,
        Err(_) => return false,
    };

    let reader = BufReader::new(file);
    for line in reader.lines() {
        if let Ok(line) = line {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() > 1 && parts[1] == path {
                return true;
            }
        }
    }
    false
}

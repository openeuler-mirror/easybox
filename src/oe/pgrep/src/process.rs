//! This file is part of the easybox package.
//
// (c) Krysztal Huang <suibing112233@outlook.com>
// (c) Xu Biang <xubiang@foxmail.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use nix::unistd::{sysconf, SysconfVar};
use std::{collections::HashMap, fs, io, os::linux::fs::MetadataExt, path::PathBuf, rc::Rc};
use walkdir::{DirEntry, WalkDir};

/// Process ID and its information used in pgrep.
#[derive(Debug, Clone, Default)]
pub struct ProcessInformation {
    /// Process id from `/proc/<pid>`.
    pub pid: usize,
    /// Unprocessed `/proc/self/status` file.
    inner_status: String,
    /// Unprocessed `/proc/self/stat` file.
    inner_stat: String,
    /// Processed `/proc/self/status` file.
    cached_status: Option<Rc<HashMap<String, String>>>,
    /// Processed `/proc/self/stat` file.
    cached_stat: Option<Rc<Vec<String>>>,
}

impl ProcessInformation {
    /// Try new with pid path such as `/proc/self`
    ///
    /// # Error
    ///
    /// If the files in path cannot be parsed into [ProcessInformation],
    /// it almost caused by wrong filesystem structure.
    ///
    /// - [The /proc Filesystem](https://docs.kernel.org/filesystems/proc.html#process-specific-subdirectories)
    pub fn try_new(value: PathBuf) -> Result<Self, io::Error> {
        let dir_append = |mut path: PathBuf, str: String| {
            path.push(str);
            path
        };

        let value = if value.is_symlink() {
            fs::read_link(value)?
        } else {
            value
        };

        let pid = value
            .iter()
            .last()
            .ok_or(io::ErrorKind::Other)?
            .to_str()
            .ok_or(io::ErrorKind::InvalidData)?
            .parse::<usize>()
            .map_err(|_| io::ErrorKind::InvalidData)?;

        Ok(Self {
            pid,
            inner_status: fs::read_to_string(dir_append(value.clone(), "status".into()))?,
            inner_stat: fs::read_to_string(dir_append(value, "stat".into()))?,
            ..Default::default()
        })
    }

    /// Collect information from `/proc/<pid>/status` file
    pub fn status(&mut self) -> Rc<HashMap<String, String>> {
        if let Some(c) = &self.cached_status {
            return Rc::clone(c);
        }

        let result = self
            .inner_status
            .lines()
            .filter_map(|it| it.split_once(':'))
            .map(|it| (it.0.to_string(), it.1.trim_start().to_string()))
            .collect::<HashMap<_, _>>();

        let result = Rc::new(result);
        self.cached_status = Some(Rc::clone(&result));
        Rc::clone(&result)
    }

    /// Collect information from `/proc/<pid>/stat` file
    fn stat(&mut self) -> Rc<Vec<String>> {
        if let Some(c) = &self.cached_stat {
            return Rc::clone(c);
        }

        let result: Vec<_> = stat_split(&self.inner_stat);

        let result = Rc::new(result);
        self.cached_stat = Some(Rc::clone(&result));
        Rc::clone(&result)
    }

    ///
    fn stat_get_usize(&mut self, index: usize) -> Result<usize, io::Error> {
        let result = self
            .stat()
            .get(index)
            .ok_or(io::ErrorKind::InvalidData)?
            .parse::<usize>()
            .map_err(|_| io::ErrorKind::InvalidData)?;

        Ok(result)
    }

    ///
    fn stat_get_string(&mut self, index: usize) -> Result<String, io::Error> {
        let stat = self.stat();
        let result = stat.get(index).ok_or(io::ErrorKind::InvalidData)?;

        Ok(result.clone())
    }

    /// Process id of the parent process from ppid in `/proc/<pid>/stat` or PPid in `/proc/<pid>/status` (favor `stat`).
    pub fn ppid(&mut self) -> Result<usize, io::Error> {
        let ppid = self.stat_get_usize(3)?;
        Ok(ppid)
    }

    /// pgrp of the process from pgrp in `/proc/<pid>/stat`.
    pub fn pgrp(&mut self) -> Result<usize, io::Error> {
        let pgrp = self.stat_get_usize(4)?;
        Ok(pgrp)
    }

    /// euid of the process from the file attributes of `/proc/<pid>/stat`.
    pub fn euid(&mut self) -> Result<usize, io::Error> {
        let euid = fs::metadata(format!("/proc/{}/stat", self.pid))?.st_uid() as usize;
        Ok(euid)
    }

    /// Real, effective, saved set, and file system UIDs from Uid in `/proc/<pid>/status`.
    pub fn ruid(&mut self) -> Result<usize, io::Error> {
        let ruid = self
            .status()
            .get("Uid")
            .ok_or(io::ErrorKind::InvalidData)?
            .as_str()
            .split_whitespace()
            .next()
            .ok_or(io::ErrorKind::InvalidData)?
            .parse::<usize>()
            .map_err(|_| io::ErrorKind::InvalidData)?;
        Ok(ruid)
    }

    /// Real, effective, saved set, and file system GIDs from Gid in `/proc/<pid>/status`.
    pub fn rgid(&mut self) -> Result<usize, io::Error> {
        let rgid = self
            .status()
            .get("Gid")
            .ok_or(io::ErrorKind::InvalidData)?
            .as_str()
            .split_whitespace()
            .next()
            .ok_or(io::ErrorKind::InvalidData)?
            .parse::<usize>()
            .map_err(|_| io::ErrorKind::InvalidData)?;
        Ok(rgid)
    }

    /// Session id from sid in `/proc/<pid>/stat`.
    pub fn session(&mut self) -> Result<usize, io::Error> {
        let session: usize = self.stat_get_usize(5)?;
        Ok(session)
    }

    /// Thread group ID from Tgid in `/proc/<pid>/status`.
    pub fn tgid(&mut self) -> Result<usize, io::Error> {
        let tgid = self
            .status()
            .get("Tgid")
            .ok_or(io::ErrorKind::InvalidData)?
            .as_str()
            .trim()
            .parse::<usize>()
            .map_err(|_| io::ErrorKind::InvalidData)?;
        Ok(tgid)
    }

    /// Time the process started after system boot (seconds) from start_time in `/proc/<pid>/stat`.
    /// (`starttime = start_time / hertz`)
    pub fn starttime(&mut self) -> Result<f64, io::Error> {
        let time = self.stat_get_usize(21)?;
        let hertz = sysconf(SysconfVar::CLK_TCK)
            .map_err(|_| io::ErrorKind::InvalidData)?
            .ok_or(io::ErrorKind::InvalidData)?;

        Ok(time as f64 / hertz as f64)
    }

    /// Name of tty the process uses from tty_nr in `/proc/<pid>/stat`.
    ///
    /// - [devices.txt](https://www.kernel.org/doc/Documentation/admin-guide/devices.txt)
    pub fn ttyname(&mut self) -> String {
        let tty_nr = self.stat_get_usize(6).unwrap_or(0) as u32;

        let major = (tty_nr >> 8) & 0xFFF;
        let minor = tty_nr & 0xFF;
        let unknown = "?".to_string();

        // TODO: more TTY types
        match major {
            4 => match minor {
                0..=63 => format!("tty{}", minor),
                64..=255 => format!("ttyS{}", minor),
                _ => unknown,
            },
            136..=143 => format!("pts/{}", minor),
            _ => unknown,
        }
    }

    /// Filename of the executable from comm in `/proc/<pid>/stat` or Name in `/proc/<pid>/status` (favor `stat`).
    pub fn cmd(&mut self) -> Result<String, io::Error> {
        let cmd = self.stat_get_string(1)?;
        Ok(cmd)
    }

    /// Complete command line from `/proc/<pid>/cmdline`.
    pub fn cmdline(&mut self) -> Result<String, io::Error> {
        let cmdline = fs::read_to_string(format!("/proc/{}/cmdline", self.pid))?
            .replace('\0', " ")
            .trim_end()
            .to_owned();
        Ok(cmdline)
    }

    /// State from state in `/proc/<pid>/stat` or State in `/proc/<pid>/status` (favor `stat`).
    pub fn sta(&mut self) -> Result<String, io::Error> {
        let sta = self.stat_get_string(2)?;
        Ok(sta)
    }

    /// Elapsed time (seconds) from start_time in `/proc/<pid>/stat`.
    /// (`elapsed = (/proc/uptime - start_time) / hertz`) ???
    pub fn elapsed(&mut self) -> Result<f64, io::Error> {
        let uptime = fs::read_to_string("/proc/uptime")?
            .split_whitespace()
            .next()
            .ok_or(io::ErrorKind::InvalidData)?
            .parse::<f64>()
            .map_err(|_| io::ErrorKind::InvalidData)?;
        let start_time = self.starttime()?;

        Ok(uptime - start_time)
    }

    /// cgroup from `/proc/<pid>/cgroup`.
    pub fn cgroup(&mut self) -> Result<String, io::Error> {
        let cgroup = fs::read_to_string(format!("/proc/{}/cgroup", self.pid))?
            .replace('\n', " ")
            .trim_end()
            .to_owned();
        Ok(cgroup)
    }

    /// Bitmap of caught signals from SigCgt in `/proc/<pid>/status`.
    pub fn sigcatch(&mut self) -> Result<String, io::Error> {
        let sigcatch = self
            .status()
            .get("SigCgt")
            .ok_or(io::ErrorKind::InvalidData)?
            .to_string();
        Ok(sigcatch)
    }

    /// Values of environment variables in `/proc/<pid>/environ`.
    pub fn environ(&mut self) -> Result<String, io::Error> {
        let environ = fs::read_to_string(format!("/proc/{}/environ", self.pid))?
            .replace('\0', " ")
            .trim_end()
            .to_owned();
        Ok(environ)
    }
}

impl TryFrom<DirEntry> for ProcessInformation {
    type Error = io::Error;

    fn try_from(value: DirEntry) -> Result<Self, Self::Error> {
        let value = value.into_path();

        ProcessInformation::try_new(value)
    }
}

/// Parsing `/proc/<pid>/stat` file.
fn stat_split(stat: &str) -> Vec<String> {
    let stat = String::from(stat);

    if let (Some(left), Some(right)) = (stat.find('('), stat.rfind(')')) {
        let mut split_stat = vec![];

        split_stat.push(stat[..left - 1].to_string());
        split_stat.push(stat[left + 1..right].to_string());
        split_stat.extend(stat[right + 2..].split_whitespace().map(String::from));

        split_stat
    } else {
        stat.split_whitespace().map(String::from).collect()
    }
}

/// Iterating pid in current system
pub fn walk_process(with_thread: bool) -> impl Iterator<Item = ProcessInformation> {
    fn walk_dir(root_path: &str) -> impl Iterator<Item = ProcessInformation> {
        WalkDir::new(root_path)
            .max_depth(1)
            .follow_links(false)
            .into_iter()
            .flatten()
            .filter(|it| it.path().is_dir())
            .flat_map(ProcessInformation::try_from)
    }

    let process_iter = walk_dir("/proc");

    if with_thread {
        process_iter
            .flat_map(|process| walk_dir(&format!("/proc/{}/task/", process.pid)))
            .collect::<Vec<_>>()
            .into_iter()
    } else {
        process_iter.collect::<Vec<_>>().into_iter()
    }
}

//! This file is part of the easybox package.
//
// (c) Zhenghang <2113130664@qq.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use clap::{crate_version, Arg, Command};
use nix::mount::{MntFlags, MsFlags};
use nix::sched::{setns, CloneFlags};
use once_cell::sync::Lazy;
use std::collections::HashSet;
use std::ffi::OsString;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::os::unix::io::AsRawFd;
use std::path::Path;
use std::sync::Mutex;
use uucore::error::{UResult, USimpleError};
use uucore::format_usage;
use uucore::mount::is_mount_point;
use uucore::umount::umount_fs;
///
pub static BASE_CMD_PARSE_ERROR: i32 = 1;

#[derive(Debug, Default)]
///
pub struct Config {
    pub all: bool,
    pub all_targets: bool,
    pub no_canonicalize: bool,
    pub detach_loop: bool,
    pub fake: bool,
    pub force: bool,
    pub internal_only: bool,
    pub no_mtab: bool,
    pub lazy: bool,
    pub recursive: bool,
    pub read_only: bool,
    pub verbose: bool,
    pub quiet: bool,
    pub help: bool,
    pub version: bool,

    pub test_opts: Option<OsString>,
    pub types: Option<OsString>,
    pub namespace: Option<OsString>,

    pub target: Option<OsString>,
}
///
pub mod options {
    ///
    pub static ALL: &str = "all";
    ///
    pub static ALL_TARGETS: &str = "all-targets";
    ///
    pub static NO_CANONICALIZE: &str = "no-canonicalize";
    ///
    pub static DETACH_LOOP: &str = "detach-loop";
    ///
    pub static FAKE: &str = "fake";
    ///
    pub static FORCE: &str = "force";
    ///
    pub static INTERNAL_ONLY: &str = "internal-only";
    ///
    pub static NO_MTAB: &str = "no-mtab";
    ///
    pub static LAZY: &str = "lazy";
    ///
    pub static TEST_OPTS: &str = "test-opts";
    ///
    pub static RECURSIVE: &str = "recursive";
    ///
    pub static READ_ONLY: &str = "read-only";
    ///
    pub static TYPES: &str = "types";
    ///
    pub static VERBOSE: &str = "verbose";
    ///
    pub static QUIET: &str = "quiet";
    ///
    pub static NAMESPACE: &str = "namespace";
    ///
    pub static HELP: &str = "help";
    ///
    pub static VERSION: &str = "version";
}

impl Config {
    ///
    pub fn from(options: &clap::ArgMatches) -> UResult<Self> {
        Ok(Self {
            all: options.is_present(options::ALL),
            all_targets: options.is_present(options::ALL_TARGETS),
            no_canonicalize: options.is_present(options::NO_CANONICALIZE),
            detach_loop: options.is_present(options::DETACH_LOOP),
            fake: options.is_present(options::FAKE),
            force: options.is_present(options::FORCE),
            internal_only: options.is_present(options::INTERNAL_ONLY),
            no_mtab: options.is_present(options::NO_MTAB),
            lazy: options.is_present(options::LAZY),
            recursive: options.is_present(options::RECURSIVE),
            read_only: options.is_present(options::READ_ONLY),
            verbose: options.is_present(options::VERBOSE),
            quiet: options.is_present(options::QUIET),
            help: options.is_present(options::HELP),
            version: options.is_present(options::VERSION),

            test_opts: options.value_of_os(options::TEST_OPTS).map(OsString::from),
            types: options.value_of_os(options::TYPES).map(OsString::from),
            namespace: options.value_of_os(options::NAMESPACE).map(OsString::from),

            target: options.value_of_os("target").map(OsString::from),
        })
    }
}
///
pub fn parse_umount_cmd_args(args: impl uucore::Args, about: &str, usage: &str) -> UResult<Config> {
    let command = umount_app(about, usage);
    let args_list = args.collect_lossy();
    match command.try_get_matches_from(args_list) {
        Ok(matches) => Config::from(&matches),
        Err(e) => Err(USimpleError::new(BASE_CMD_PARSE_ERROR, e.to_string())),
    }
}
///
pub fn umount_app<'a>(about: &'a str, usage: &'a str) -> Command<'a> {
    let mut cmd = Command::new(uucore::util_name())
        .version(crate_version!())
        .about(about)
        .override_usage(format_usage(usage))
        .infer_long_args(true);

    cmd = cmd.arg(
        Arg::new("target")
            .help("Specify the target to unmount")
            .index(1)
            .allow_invalid_utf8(true),
    );

    for (name, short, help) in &[
        (options::ALL, Some('a'), "Unmount all filesystems"),
        (
            options::ALL_TARGETS,
            Some('A'),
            "Unmount all mount points for the specified device in the current namespace",
        ),
        (
            options::NO_CANONICALIZE,
            Some('c'),
            "Don't canonicalize paths",
        ),
        (
            options::DETACH_LOOP,
            Some('d'),
            "If mounted loop device, also free this loop device",
        ),
        (
            options::FAKE,
            None,
            "Dry run; skip the umount(2) system call",
        ),
        (
            options::FORCE,
            Some('f'),
            "Force unmount (in case of an unreachable NFS system)",
        ),
        (
            options::INTERNAL_ONLY,
            Some('i'),
            "Don't call the umount.<type> helper program",
        ),
        (options::NO_MTAB, Some('n'), "Don't write to /etc/mtab"),
        (
            options::LAZY,
            Some('l'),
            "Detach the filesystem now, clean up things later",
        ),
        (
            options::RECURSIVE,
            Some('R'),
            "Recursively unmount a target with all its children",
        ),
        (
            options::READ_ONLY,
            Some('r'),
            "In case unmounting fails, try to remount read-only",
        ),
        (options::VERBOSE, Some('v'), "Print current action"),
        (
            options::QUIET,
            Some('q'),
            "suppress 'not mounted' error messages",
        ),
        (options::HELP, Some('h'), "display this help"),
        (options::VERSION, Some('V'), "display version"),
    ] {
        let arg = Arg::new(*name).long(*name).help(*help);
        cmd = cmd.arg(if let Some(s) = short {
            arg.short(*s)
        } else {
            arg
        });
    }

    for (name, short, help, value_name) in &[
        (
            options::TEST_OPTS,
            Some('O'),
            "Limit the set of filesystems (use with -a)",
            "list",
        ),
        (
            options::TYPES,
            Some('t'),
            "Limit the set of filesystem types",
            "list",
        ),
        (
            options::NAMESPACE,
            Some('N'),
            "perform umount in another namespace",
            "ns",
        ),
    ] {
        let arg = Arg::new(*name)
            .long(*name)
            .help(*help)
            .value_name(*value_name)
            .takes_value(true)
            .allow_invalid_utf8(true);
        cmd = cmd.arg(if let Some(s) = short {
            arg.short(*s)
        } else {
            arg
        });
    }

    cmd
}
///
pub struct UmountHandler {
    config: Config,
}
static MTAB_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

impl UmountHandler {
    ///
    pub fn new(config: Config) -> UmountHandler {
        Self { config }
    }
    ///
    pub fn process(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.handle_namespace()?;
        self.handle_basic_options()?;
        if self.config.all_targets {
            return Ok(());
        }
        self.handle_target()?;
        Ok(())
    }

    fn handle_namespace(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ns) = &self.config.namespace {
            self.verbose_print(&format!("Using namespace: {:?}", ns));
            // Implement namespace switching logic here
            self.enter_namespace()?;
        }
        Ok(())
    }

    fn handle_basic_options(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.config.all {
            self.umount_all_filesystems()?;
        } else if self.config.all_targets {
            self.umount_all_targets()?;
        }
        Ok(())
    }

    fn handle_target(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(target) = &self.config.target {
            let target_path = if self.config.no_canonicalize {
                target.to_string_lossy().into_owned()
            } else {
                self.canonicalize_path(target.to_str().unwrap())?
            };
            if self.config.recursive {
                self.umount_recursive(&target_path)?;
            } else {
                self.umount_single_target(&target_path)?;
            }
        }
        Ok(())
    }

    fn umount_all_filesystems(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.verbose_print("Unmounting all filesystems");
        // Implement logic to unmount all filesystems
        if !self.config.fake {
            let mounts = fs::read_to_string("/proc/mounts")?;
            let mut mounted = Vec::new();

            for line in mounts.lines().rev() {
                let fields: Vec<&str> = line.split_whitespace().collect();
                if fields.len() >= 3 {
                    let mount_point = fields[1];
                    let fs_type = fields[2];

                    if self.should_umount(mount_point, fs_type) {
                        mounted.push(mount_point);
                    }
                }
            }

            for mount_point in mounted {
                let mut is_skiped = false;
                for i in ["/dev/pts", "/sys", "/proc", "/dev/console"] {
                    if mount_point.trim() == i {
                        is_skiped = true;
                        break;
                    }
                }
                if is_skiped {
                    continue;
                }
                self.umount_single_target(mount_point)?;
            }
        }
        Ok(())
    }
    fn should_umount(&self, mount_point: &str, fs_type: &str) -> bool {
        if let Some(types) = &self.config.types {
            let types_str = types.to_str().unwrap_or_else(|| {
                log::warn!("Unable to convert filesystem types to string, using empty string");
                ""
            });
            let allowed_types: HashSet<_> = types_str.split(',').collect();
            if !allowed_types.contains(fs_type) {
                return false;
            }
        }

        if let Some(test_opts) = &self.config.test_opts {
            let test_opts_str = test_opts.to_str().unwrap_or_else(|| {
                log::warn!("Unable to convert test options to string, using empty string");
                ""
            });
            let mount_opts = self.get_mount_options(mount_point);
            let required_opts: HashSet<_> = test_opts_str.split(',').collect::<HashSet<_>>();
            if !required_opts.iter().all(|opt| mount_opts.contains(*opt)) {
                return false;
            }
        }
        true
    }

    fn get_mount_options(&self, mount_point: &str) -> HashSet<String> {
        let mut options = HashSet::new();
        if let Ok(file) = File::open("/proc/mounts") {
            let reader = BufReader::new(file);
            for line in reader.lines() {
                if let Ok(line) = line {
                    let fields: Vec<&str> = line.split_whitespace().collect();
                    if fields.len() >= 4 && fields[1] == mount_point {
                        options = fields[3].split(',').map(String::from).collect();
                        break;
                    }
                }
            }
        }
        options
    }
    fn umount_all_targets(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.verbose_print("Unmounting all targets for the specified device");
        // Implement logic to unmount all targets for a device
        // Read /proc/mounts file to get all mount point information
        let mounts = fs::read_to_string("/proc/mounts")?;
        let device_to_unmount = self
            .config
            .target
            .as_ref()
            .ok_or("No device specified for unmounting all targets")?;

        // Iterate through all mount points, find matching devices and unmount
        for line in mounts.lines() {
            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.len() >= 2 && fields[0] == device_to_unmount {
                let mount_point = fields[1];
                if !self.config.fake {
                    self.umount_single_target(mount_point)?;
                }
            }
        }

        Ok(())
    }

    fn umount_single_target(&self, target: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Implement logic to unmount a single target
        let loop_device = self.get_loop_device(target);
        if !self.config.fake {
            // Use the umount_fs function to actually perform the unmount operation
            let result = if self.config.force || self.config.lazy {
                let mut flags = MntFlags::empty();
                if self.config.force {
                    flags |= MntFlags::MNT_FORCE;
                }
                if self.config.lazy {
                    flags |= MntFlags::MNT_DETACH;
                }
                umount_fs::<&str>(target.as_ref(), flags, self.config.internal_only)
            } else {
                let flags = MntFlags::empty();
                umount_fs::<&str>(target.as_ref(), flags, self.config.internal_only)
            };
            match result {
                Ok(_) => {
                    self.verbose_print(&format!(
                        "{} ({}) unmounted",
                        target,
                        loop_device.as_deref().unwrap_or_default()
                    ));
                    if self.config.detach_loop {
                        if let Ok(device) = loop_device {
                            match self.detach_loop_device(&device) {
                                Ok(_) => self.verbose_print(&format!(
                                    "Successfully detached loop device {}",
                                    device
                                )),
                                Err(e) => self.verbose_print(&format!(
                                    "Failed to detach loop device {}: {}",
                                    device, e
                                )),
                            }
                        } else {
                            self.verbose_print("No loop device found to detach");
                        }
                    }
                    if !self.config.no_mtab {
                        self.update_mtab(target)?;
                    }
                }
                Err(e) => {
                    if self.config.read_only {
                        self.verbose_print(&format!(
                            "Unmount failed, attempting read-only remount for {}",
                            target
                        ));
                        self.remount_read_only(target)?;
                    } else if !self.config.quiet {
                        eprintln!("Failed to unmount {}: {}", target, e);
                        return Err(Box::new(e));
                    }
                }
            }
        }
        Ok(())
    }

    fn verbose_print(&self, message: &str) {
        if self.config.verbose && !self.config.quiet {
            eprintln!("umount: {}", message);
        }
    }

    fn enter_namespace(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ns) = &self.config.namespace {
            self.verbose_print(&format!("Entering namespace: {:?}", ns));
            let res = File::open(ns);
            if res.is_err() {
                return Ok(());
            }
            let ns_file = res.unwrap();
            let _guard = scopeguard::guard(ns_file, |f| drop(f));
            setns(_guard.as_raw_fd(), CloneFlags::CLONE_NEWNS)
                .map_err(|e| format!("Failed to enter namespace: {}", e))?;
            self.verbose_print("Successfully entered the specified namespace");
        }
        Ok(())
    }
    fn detach_loop_device(&self, device: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.verbose_print(&format!("Attempting to detach loop device for {}", device));
        // Open the device file
        let file = File::open(&device)?;
        let fd = file.as_raw_fd();

        // LOOP_CLR_FD ioctl request code
        const LOOP_CLR_FD: nix::libc::c_ulong = 0x4C01;

        // Execute ioctl call
        unsafe {
            if nix::libc::ioctl(fd, LOOP_CLR_FD.try_into().unwrap(), 0) == -1 {
                return Err(Box::new(std::io::Error::last_os_error()));
            }
        }
        Ok(())
    }
    fn get_loop_device(&self, target: &str) -> Result<String, Box<dyn std::error::Error>> {
        let target_path = Path::new(target).canonicalize()?;
        // Method 1: Check /proc/mounts
        let mounts = fs::read_to_string("/proc/mounts")?;
        for line in mounts.lines() {
            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.len() > 1 {
                let mount_point = Path::new(fields[1])
                    .canonicalize()
                    .unwrap_or_else(|_| Path::new(fields[1]).to_path_buf());
                if mount_point == target_path && fields[0].starts_with("/dev/loop") {
                    return Ok(fields[0].to_string());
                }
            }
        }

        // Method 2: Use losetup command
        let output = std::process::Command::new("losetup").arg("-a").output()?;
        let output_str = String::from_utf8_lossy(&output.stdout);
        for line in output_str.lines() {
            let parts: Vec<&str> = line.splitn(2, ": ").collect();
            if parts.len() == 2 {
                let device = parts[0];
                let file_path = parts[1].trim_start_matches('(').trim_end_matches(')');
                if Path::new(file_path).canonicalize()? == target_path {
                    self.verbose_print(&format!("Found loop device using losetup: {}", device));
                    return Ok(device.to_string());
                }
            }
        }
        self.verbose_print("No loop device found for the given target");
        Err("No loop device found for the given target".into())
    }
    fn remount_read_only(&self, target: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.verbose_print(&format!("Remounting {} as read-only", target));
        let path = Path::new(target);
        nix::mount::mount(
            None::<&str>,
            path,
            None::<&str>,
            MsFlags::MS_REMOUNT | MsFlags::MS_RDONLY,
            None::<&str>,
        )?;
        Ok(())
    }
    fn canonicalize_path(&self, path: &str) -> Result<String, Box<dyn std::error::Error>> {
        let canonical_path = fs::canonicalize(path)?;
        Ok(canonical_path.to_string_lossy().into_owned())
    }
    fn umount_recursive(&self, target: &str) -> Result<(), Box<dyn std::error::Error>> {
        let path = Path::new(target);
        if path.is_dir() {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    if let Err(e) = self.umount_recursive(path.to_str().ok_or("Invalid path")?) {
                        log::warn!(
                            "Error during recursive unmount of {}: {}",
                            path.display(),
                            e
                        );
                    }
                }
            }
        }
        if is_mount_point(target) {
            return self.umount_single_target(target);
        }
        Ok(())
    }

    fn update_mtab(&self, target: &str) -> Result<(), Box<dyn std::error::Error>> {
        if fs::symlink_metadata("/etc/mtab")?.file_type().is_symlink() {
            return Ok(());
        }

        let _lock = MTAB_LOCK.lock().unwrap();
        let content = fs::read_to_string("/etc/mtab")?;
        let updated_content: String = content
            .lines()
            .filter(|line| {
                !line
                    .split_whitespace()
                    .nth(1)
                    .map_or(false, |mp| mp == target)
            })
            .collect::<Vec<&str>>()
            .join("\n");
        fs::write("/etc/mtab", updated_content)?;
        Ok(())
    }
}

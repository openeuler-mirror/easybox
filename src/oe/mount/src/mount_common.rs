//! This file is part of the easybox package.
//
// (c) Zhenghang <2113130664@qq.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use clap::{crate_version, Arg, ArgGroup, Command};
use nix::mount::MsFlags;
use nix::sched::{setns, CloneFlags};
use nix::unistd::{fork, ForkResult};

use std::collections::HashSet;
use std::ffi::OsString;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::os::unix::io::AsRawFd;
use std::path::Path;
use std::process::exit;
use std::{fs, io};
use uucore::error::{UResult, USimpleError};
use uucore::format_usage;
use uucore::mount::{
    find_device_by_label, find_device_by_uuid, is_already_mounted, is_mount_point, is_swapfile,
    mount_fs, parse_fstab, prepare_mount_source,
};
///
pub static BASE_CMD_PARSE_ERROR: i32 = 1;

#[derive(Debug, Default)]
///
pub struct Config {
    // Basic options
    ///
    pub all: bool, // Mount all filesystems mentioned in /etc/fstab
    ///
    pub no_canonicalize: bool, // Don't canonicalize paths
    ///
    pub fake: bool, // Simulate mounting, don't actually call mount system call
    ///
    pub fork: bool, // Fork for each device (used with -a)
    ///
    pub fstab: Option<OsString>, // Specify alternative file to /etc/fstab
    ///
    pub internal_only: bool, // Don't call mount.<type> helper program
    ///
    pub show_labels: bool, // Show filesystem labels
    ///
    pub no_mtab: bool, // Don't write to /etc/mtab file
    ///
    pub verbose: bool, // Display detailed operation information
    ///
    pub help: bool, // Display help information
    ///
    pub version: bool, // Display version information
    /// Mount options
    pub options: MountOptions,
    /// Source and target
    pub source: Option<Source>, // Explicitly specify source (path, label, UUID)
    ///
    pub target: Option<OsString>, // Explicitly specify mount point
    ///
    pub target_prefix: Option<OsString>, // Specify path prefix for all mount points
    /// Namespace
    pub namespace: Option<OsString>, // Execute mount in another namespace
    /// Operation
    pub operation: Operation,
}

#[derive(Debug, Default)]
///
pub struct MountOptions {
    ///
    pub mode: Option<OsString>, // Specify how to handle options loaded from fstab
    ///
    pub source: Option<OsString>, // Specify source of mount options
    ///
    pub source_force: bool, // Force use of options from fstab/mtab
    ///
    pub options: Option<OsString>, // Specify comma-separated list of mount options
    ///
    pub test_opts: Option<OsString>, // Limit set of filesystems (used with -a)
    ///
    pub read_only: bool, // Mount filesystem read-only
    ///
    pub read_write: bool, // Mount filesystem read-write (default)
    ///
    pub types: Option<OsString>, // Limit filesystem types
}
#[derive(Debug)]
///
pub enum Source {
    ///
    Device(OsString), // Specify by device path
    ///
    Label(OsString), // Specify device by filesystem label
    ///
    UUID(OsString), // Specify device by filesystem UUID
}

#[derive(Debug, Default, PartialEq)]
///
pub enum Operation {
    #[default]
    ///
    Normal,
    ///
    Bind, // Mount a subtree to another location
    ///
    Move, // Move a subtree to another location
    ///
    RBind, // Mount a subtree and all its submounts to another location
    ///
    MakeShared, // Mark a subtree as shared
    ///
    MakeSlave, // Mark a subtree as slave
    ///
    MakePrivate, // Mark a subtree as private
    ///
    MakeUnbindable, // Mark a subtree as unbindable
    ///
    MakeRShared, // Recursively mark an entire subtree as shared
    ///
    MakeRSlave, // Recursively mark an entire subtree as slave
    ///
    MakeRPrivate, // Recursively mark an entire subtree as private
    ///
    MakeRUnbindable, // Recursively mark an entire subtree as unbindable
}
///
pub mod options {
    ///
    pub static ALL: &str = "all";
    ///
    pub static NO_CANONICALIZE: &str = "no-canonicalize";
    ///
    pub static FAKE: &str = "fake";
    ///
    pub static FORK: &str = "fork";
    ///
    pub static FSTAB: &str = "fstab";
    ///
    pub static INTERNAL_ONLY: &str = "internal-only";
    ///
    pub static SHOW_LABELS: &str = "show-labels";
    ///
    pub static NO_MTAB: &str = "no-mtab";
    ///
    pub static OPTIONS_MODE: &str = "options-mode";
    ///
    pub static OPTIONS_SOURCE: &str = "options-source";
    ///
    pub static OPTIONS_SOURCE_FORCE: &str = "options-source-force";
    ///
    pub static OPTIONS: &str = "options";
    ///
    pub static TEST_OPTS: &str = "test-opts";
    ///
    pub static READ_ONLY: &str = "read-only";
    ///
    pub static TYPES: &str = "types";
    ///
    pub static SOURCE: &str = "source";
    ///
    pub static TARGET: &str = "target";
    ///
    pub static TARGET_PREFIX: &str = "target-prefix";
    ///
    pub static VERBOSE: &str = "verbose";
    ///
    pub static READ_WRITE: &str = "read-write";
    ///
    pub static NAMESPACE: &str = "namespace";
    ///
    pub static HELP: &str = "help";
    ///
    pub static VERSION: &str = "version";

    // Source
    ///
    pub static LABEL: &str = "label";
    ///
    pub static UUID: &str = "uuid";
    ///
    pub static DEVICE: &str = "device";

    // operations
    ///
    pub static BIND: &str = "bind";
    ///
    pub static MOVE: &str = "move";
    ///
    pub static RBIND: &str = "rbind";
    ///
    pub static MAKE_SHARED: &str = "make-shared";
    ///
    pub static MAKE_SLAVE: &str = "make-slave";
    ///
    pub static MAKE_PRIVATE: &str = "make-private";
    ///
    pub static MAKE_UNBINDABLE: &str = "make-unbindable";
    ///
    pub static MAKE_RSHARED: &str = "make-rshared";
    ///
    pub static MAKE_RSLAVE: &str = "make-rslave";
    ///
    pub static MAKE_RPRIVATE: &str = "make-rprivate";
    ///
    pub static MAKE_RUNBINDABLE: &str = "make-runbindable";
}

impl Config {
    ///
    pub fn from(options: &clap::ArgMatches) -> UResult<Self> {
        let operation = Self::parse_operation(options);
        let no_canonicalize = options.is_present(options::NO_CANONICALIZE);

        let (canonicalized_source, canonicalized_target) = if !no_canonicalize {
            let source = if operation == Operation::Move {
                options
                    .value_of_os(options::DEVICE)
                    .map(|s| Source::Device(s.to_owned()))
            } else if operation == Operation::MakeShared
                || operation == Operation::MakeSlave
                || operation == Operation::MakePrivate
                || operation == Operation::MakeUnbindable
                || operation == Operation::MakeRShared
                || operation == Operation::MakeRSlave
                || operation == Operation::MakeRPrivate
                || operation == Operation::MakeRUnbindable
            {
                None
            } else {
                Self::parse_source(options)
            };

            let target = if operation == Operation::Move {
                options.value_of_os("target_positional")
            } else if operation == Operation::MakeShared
                || operation == Operation::MakeSlave
                || operation == Operation::MakePrivate
                || operation == Operation::MakeUnbindable
                || operation == Operation::MakeRShared
                || operation == Operation::MakeRSlave
                || operation == Operation::MakeRPrivate
                || operation == Operation::MakeRUnbindable
            {
                options.value_of_os(options::DEVICE)
            } else {
                options
                    .value_of_os(options::TARGET)
                    .or_else(|| options.value_of_os("target_positional"))
            }
            .map(OsString::from);
            (
                source.and_then(|s| match s {
                    Source::Device(dev) => match fs::canonicalize(&dev) {
                        Ok(path) => Some(Source::Device(path.into_os_string())),
                        Err(e) => {
                            eprintln!(
                                "Warning: Unable to canonicalize device path {:?}: {}",
                                dev, e
                            );
                            Some(Source::Device(dev))
                        }
                    },
                    Source::Label(label) => Some(Source::Label(label)),
                    Source::UUID(uuid) => Some(Source::UUID(uuid)),
                }),
                target.and_then(|t| match fs::canonicalize(&t) {
                    Ok(path) => Some(path.into_os_string()),
                    Err(e) => {
                        eprintln!("Warning: Unable to canonicalize device path {:?}: {}", t, e);
                        Some(t)
                    }
                }),
            )
        } else {
            // If no canonicalization is specified, use the original paths
            let source = if operation == Operation::Move {
                options
                    .value_of_os(options::DEVICE)
                    .map(|s| Source::Device(s.to_owned()))
            } else {
                Self::parse_source(options)
            };

            let target = if operation == Operation::Move {
                options.value_of_os("target_positional")
            } else {
                options
                    .value_of_os(options::TARGET)
                    .or_else(|| options.value_of_os("target_positional"))
            }
            .map(OsString::from);

            (source, target)
        };

        Ok(Self {
            all: options.is_present(options::ALL),
            no_canonicalize: options.is_present(options::NO_CANONICALIZE),
            fake: options.is_present(options::FAKE),
            fork: options.is_present(options::FORK),
            fstab: options.value_of_os(options::FSTAB).map(OsString::from),
            internal_only: options.is_present(options::INTERNAL_ONLY),
            show_labels: options.is_present(options::SHOW_LABELS),
            no_mtab: options.is_present(options::NO_MTAB),
            verbose: options.is_present(options::VERBOSE),
            help: options.is_present(options::HELP),
            version: options.is_present(options::VERSION),

            options: MountOptions {
                mode: options
                    .value_of_os(options::OPTIONS_MODE)
                    .map(OsString::from),
                source: options
                    .value_of_os(options::OPTIONS_SOURCE)
                    .map(OsString::from),
                source_force: options.is_present(options::OPTIONS_SOURCE_FORCE),
                options: options.value_of_os(options::OPTIONS).map(OsString::from),
                test_opts: options.value_of_os(options::TEST_OPTS).map(OsString::from),
                read_only: options.is_present(options::READ_ONLY),
                read_write: options.is_present(options::READ_WRITE),
                types: options.value_of_os(options::TYPES).map(OsString::from),
            },

            source: canonicalized_source,
            target: canonicalized_target,
            target_prefix: options
                .value_of_os(options::TARGET_PREFIX)
                .map(OsString::from),

            namespace: options.value_of_os(options::NAMESPACE).map(OsString::from),

            operation: Self::parse_operation(options),
        })
    }

    fn parse_source(options: &clap::ArgMatches) -> Option<Source> {
        options
            .value_of_os(options::DEVICE)
            .or_else(|| options.value_of_os(options::SOURCE))
            .map(|device| {
                if options.contains_id(options::LABEL) {
                    Source::Label(device.to_owned())
                } else if options.contains_id(options::UUID) {
                    Source::UUID(device.to_owned())
                } else {
                    Source::Device(device.to_owned())
                }
            })
    }

    fn parse_operation(options: &clap::ArgMatches) -> Operation {
        if options.is_present(options::BIND) {
            Operation::Bind
        } else if options.is_present(options::MOVE) {
            Operation::Move
        } else if options.is_present(options::RBIND) {
            Operation::RBind
        } else if options.is_present(options::MAKE_SHARED) {
            Operation::MakeShared
        } else if options.is_present(options::MAKE_SLAVE) {
            Operation::MakeSlave
        } else if options.is_present(options::MAKE_PRIVATE) {
            Operation::MakePrivate
        } else if options.is_present(options::MAKE_UNBINDABLE) {
            Operation::MakeUnbindable
        } else if options.is_present(options::MAKE_RSHARED) {
            Operation::MakeRShared
        } else if options.is_present(options::MAKE_RSLAVE) {
            Operation::MakeRSlave
        } else if options.is_present(options::MAKE_RPRIVATE) {
            Operation::MakeRPrivate
        } else if options.is_present(options::MAKE_RUNBINDABLE) {
            Operation::MakeRUnbindable
        } else {
            Operation::Normal
        }
    }
}
/// Parse arguments and populate Config struct
pub fn parse_mount_cmd_args(args: impl uucore::Args, about: &str, usage: &str) -> UResult<Config> {
    let command = mount_app(about, usage);
    let args_list = args.collect_lossy();
    match command.try_get_matches_from(args_list) {
        Ok(matches) => Config::from(&matches),
        Err(e) => Err(USimpleError::new(BASE_CMD_PARSE_ERROR, e.to_string())),
    }
}
/// Define command line application structure and arguments, using uucore to simplify code
pub fn mount_app<'a>(about: &'a str, usage: &'a str) -> Command<'a> {
    let mut cmd = Command::new(uucore::util_name())
        .version(crate_version!())
        .about(about)
        .override_usage(format_usage(usage))
        .infer_long_args(true);

    // Add positional arguments
    cmd = cmd
        .arg(
            Arg::new(options::DEVICE)
                .takes_value(true)
                .help("Specify device by path")
                .index(1)
                .allow_invalid_utf8(true),
        )
        .arg(
            Arg::new("target_positional")
                .takes_value(true)
                .help("Specify mount point")
                .index(2)
                .allow_invalid_utf8(true),
        );

    // Add boolean flags
    for (name, short, help) in &[
        (
            options::ALL,
            Some('a'),
            "Mount all filesystems mentioned in fstab",
        ),
        (
            options::NO_CANONICALIZE,
            Some('c'),
            "Don't canonicalize paths",
        ),
        (
            options::FAKE,
            Some('f'),
            "Dry run; skip the mount(2) system call",
        ),
        (
            options::FORK,
            Some('F'),
            "Fork for each device (use with -a option)",
        ),
        (
            options::INTERNAL_ONLY,
            Some('i'),
            "Don't call the mount.<type> helper program",
        ),
        (
            options::SHOW_LABELS,
            Some('l'),
            "Also show filesystem labels",
        ),
        (options::NO_MTAB, Some('n'), "Don't write to /etc/mtab"),
        (
            options::OPTIONS_SOURCE_FORCE,
            Some('\0'),
            "Force use of options from fstab/mtab",
        ),
        (
            options::READ_ONLY,
            Some('r'),
            "Mount filesystem read-only (same as -o ro)",
        ),
        (options::VERBOSE, Some('v'), "Print current operations"),
        (
            options::READ_WRITE,
            Some('w'),
            "Mount filesystem read-write (default)",
        ),
        (options::HELP, Some('h'), "Display this help"),
        (options::VERSION, Some('V'), "Display version"),
    ] {
        let arg = Arg::new(*name).long(*name).help(*help).global(true);
        cmd = cmd.arg(if let Some(s) = short {
            arg.short(*s)
        } else {
            arg
        });
    }
    for (name, short, help) in &[
        (options::FSTAB, Some('T'), "Alternative file to /etc/fstab"),
        (
            options::OPTIONS_MODE,
            None,
            "How to handle options loaded from fstab",
        ),
        (options::OPTIONS_SOURCE, None, "Mount options source"),
        (
            options::OPTIONS,
            Some('o'),
            "Comma-separated list of mount options",
        ),
        (
            options::TEST_OPTS,
            Some('O'),
            "Limit set of filesystems (use with -a option)",
        ),
        (
            options::TYPES,
            Some('t'),
            "Limit the set of filesystem types",
        ),
        (options::SOURCE, None, "Specify source (path, label, uuid)"),
        (options::TARGET, None, "Specify mount point"),
        (
            options::TARGET_PREFIX,
            None,
            "Specify path used for all mountpoints",
        ),
        (
            options::NAMESPACE,
            Some('N'),
            "Perform mount in another namespace",
        ),
    ] {
        let arg = Arg::new(*name)
            .long(*name)
            .help(*help)
            .takes_value(true)
            .allow_invalid_utf8(true);
        cmd = cmd.arg(if let Some(s) = short {
            arg.short(*s)
        } else {
            arg
        });
    }
    for (name, short, help) in &[
        (
            options::BIND,
            Some('B'),
            "Mount a subtree somewhere else (same as -o bind)",
        ),
        (
            options::MOVE,
            Some('M'),
            "Move a subtree to some other place",
        ),
        (
            options::RBIND,
            Some('R'),
            "Mount a subtree and all submounts somewhere else",
        ),
        (options::MAKE_SHARED, None, "Mark a subtree as shared"),
        (options::MAKE_SLAVE, None, "Mark a subtree as slave"),
        (options::MAKE_PRIVATE, None, "Mark a subtree as private"),
        (
            options::MAKE_UNBINDABLE,
            None,
            "Mark a subtree as unbindable",
        ),
        (
            options::MAKE_RSHARED,
            None,
            "Recursively mark an entire subtree as shared",
        ),
        (
            options::MAKE_RSLAVE,
            None,
            "Recursively mark an entire subtree as slave",
        ),
        (
            options::MAKE_RPRIVATE,
            None,
            "Recursively mark an entire subtree as private",
        ),
        (
            options::MAKE_RUNBINDABLE,
            None,
            "Recursively mark an entire subtree as unbindable",
        ),
        (options::LABEL, Some('L'), "Synonym for LABEL=<label>"),
        (options::UUID, Some('U'), "Synonym for UUID=<uuid>"),
    ] {
        let arg = Arg::new(*name).long(*name).help(*help);
        cmd = cmd.arg(if let Some(s) = short {
            arg.short(*s)
        } else {
            arg
        });
    }
    cmd = cmd
        .group(
            ArgGroup::new("operation")
                .args(&[
                    options::BIND,
                    options::MOVE,
                    options::RBIND,
                    options::MAKE_SHARED,
                    options::MAKE_SLAVE,
                    options::MAKE_PRIVATE,
                    options::MAKE_UNBINDABLE,
                    options::MAKE_RSHARED,
                    options::MAKE_RSLAVE,
                    options::MAKE_RPRIVATE,
                    options::MAKE_RUNBINDABLE,
                ])
                .required(false),
        )
        .group(
            ArgGroup::new("source_operation")
                .args(&[options::DEVICE, options::SOURCE])
                .required(false),
        )
        .group(
            ArgGroup::new("read_write_mode")
                .args(&[options::READ_ONLY, options::READ_WRITE])
                .required(false),
        )
        .group(
            ArgGroup::new("options_source")
                .args(&[options::OPTIONS_SOURCE, options::OPTIONS_SOURCE_FORCE])
                .required(false),
        )
        .group(
            ArgGroup::new("target_options")
                .args(&[options::TARGET, "target_positional"])
                .required(false),
        );
    cmd.trailing_var_arg(true)
}
///
pub struct ConfigHandler {
    config: Config,
}
impl ConfigHandler {
    ///
    pub fn new(config: Config) -> ConfigHandler {
        Self { config }
    }
    ///
    pub fn process(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.handle_namespace()?;
        self.handle_basic_options()?;
        self.handle_operation()?;
        Ok(())
    }
    fn handle_basic_options(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.config.all {
            self.mount_all_filesystems()?;
        }
        if let Some(fstab) = &self.config.fstab {
            self.use_alternative_fstab(fstab)?;
        }
        Ok(())
    }

    fn handle_namespace(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.config.namespace.is_some() {
            self.enter_namespace()?;
        }
        Ok(())
    }

    fn handle_operation(&self) -> Result<(), Box<dyn std::error::Error>> {
        match &self.config.operation {
            Operation::Normal => self.perform_normal_mount()?,
            Operation::Bind => self.perform_bind_mount()?,
            Operation::Move => self.perform_move_mount()?,
            // Operation::RBind => self.perform_rbind_mount()?,
            Operation::RBind => self.perform_bind_mount()?,
            Operation::MakeShared => self.make_mount_shared()?,
            Operation::MakeSlave => self.make_mount_slave()?,
            Operation::MakePrivate => self.make_mount_private()?,
            Operation::MakeUnbindable => self.make_mount_unbindable()?,
            Operation::MakeRShared => self.make_mount_rshared()?,
            Operation::MakeRSlave => self.make_mount_rslave()?,
            Operation::MakeRPrivate => self.make_mount_rprivate()?,
            Operation::MakeRUnbindable => self.make_mount_runbindable()?,
        }
        Ok(())
    }
    // Ancillary methods
    fn verbose_print(&self, message: &str) {
        if self.config.verbose {
            println!("mount: {}", message);
        }
    }
    fn mount_all_filesystems(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.verbose_print("Mounting all filesystems from /etc/fstab");
        let fstab_path = "/etc/fstab";
        let parse_fstab_res = parse_fstab(fstab_path);
        if parse_fstab_res.is_err() {
            return Ok(());
        }
        let fstab_file = parse_fstab_res.unwrap();
        // Implement logic to mount all filesystems
        for line_vec in fstab_file {
            let source = &line_vec[0];
            let prepare_mount_source_res = prepare_mount_source(&source);
            if prepare_mount_source_res.is_err() {
                continue;
            }
            let _mount_source = Some(prepare_mount_source_res.unwrap());
            let target = &line_vec[1];
            let fstype = line_vec[2].as_str().clone();
            let _flags = MsFlags::MS_NOEXEC | MsFlags::MS_NOSUID;
            let fstab_options = &line_vec[3];
            if let Some(test_opts) = &self.config.options.test_opts {
                if !self.match_test_opts(fstab_options, test_opts) {
                    self.verbose_print(&format!("Skipping mount of {} due to test_opts", target));
                    continue;
                }
            }
            if self.should_fork() {
                match unsafe { fork() } {
                    Ok(ForkResult::Parent { child }) => {
                        // Parent process
                        println!("Forked child with PID: {}", child);
                    }
                    Ok(ForkResult::Child) => {
                        // Child process
                        if let Err(e) = self.mount_single_filesystem(source, target, fstype) {
                            eprintln!("Failed to mount {}: {}", source, e);
                            exit(1);
                        }
                        exit(0);
                    }
                    Err(e) => return Err(Box::new(e)),
                }
            } else {
                if let Err(e) = self.mount_single_filesystem(source, target, fstype) {
                    eprintln!("Failed to mount {}: {}", source, e);
                }
            }
        }
        if self.should_fork() {
            // Wait for all child processes to complete
            use nix::sys::wait::{waitpid, WaitStatus};
            use nix::unistd::Pid;

            loop {
                match waitpid(Pid::from_raw(-1), None) {
                    Ok(WaitStatus::Exited(_, _)) => {}
                    Ok(WaitStatus::Signaled(_, _, _)) => {}
                    Ok(_) => continue,
                    Err(nix::errno::Errno::ECHILD) => break,
                    Err(e) => return Err(Box::new(e)),
                }
            }
        }
        Ok(())
    }
    fn match_test_opts(&self, fstab_opts: &str, test_opts: &OsString) -> bool {
        let fstab_opts_set: HashSet<&str> = fstab_opts.split(',').collect();
        let test_opts_set: HashSet<&str> = test_opts.to_str().unwrap_or("").split(',').collect();

        test_opts_set.is_subset(&fstab_opts_set)
    }
    fn mount_single_filesystem(
        &self,
        source: &str,
        target: &str,
        fstype: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mount_source = Some(prepare_mount_source(source).unwrap());
        let flags = MsFlags::MS_NOEXEC | MsFlags::MS_NOSUID;
        let data = None;
        let interal_only = self.use_internal_only();
        if is_already_mounted(target).unwrap() {
            println!("Filesystem path: {} is already mounted! Skipping!", target);
            return Ok(());
        }
        if is_swapfile(fstype) {
            println!(
                "Skipping mounting swap file!: {}, please use swapon to mount swap files!",
                source
            );
            return Ok(());
        }

        if !self.is_fake_mode() {
            mount_fs(
                mount_source.as_ref(),
                &target.to_string(),
                Some(fstype),
                flags,
                data,
                interal_only,
            )?;
        }

        self.verbose_print(&format!("{} mounted on {}.", source, target));
        Ok(())
    }
    fn use_alternative_fstab(&self, fstab: &OsString) -> Result<(), Box<dyn std::error::Error>> {
        let fstab_path = fstab.to_str().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid fstab path".to_string(),
            )
        })?;
        let path = Path::new(fstab_path);
        if path.is_dir() {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("{:?} is a directory. Please specify a file.", path),
            )));
        }

        // Read and parse the alternative fstab file
        match parse_fstab(fstab_path) {
            Ok(fstab_entries) => {
                for entry in fstab_entries {
                    //  Perform mount operation for each fstab entry
                    let source = &entry[0];
                    let mount_source = prepare_mount_source(source)
                        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
                    let target = &entry[1];
                    let fstype = &entry[2];
                    self.mount_single_filesystem(&mount_source, target, fstype)?;
                }
                Ok(())
            }
            Err(e) => Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                e.to_string(),
            ))),
        }
    }
    fn perform_normal_mount(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Implement the logic of a normal mount
        if self.config.source.is_none() && !self.config.all {
            if self.config.target.is_none() {
                self.print_all()?;
                return Ok(());
            }
            return Err(Box::new(io::Error::new(
                io::ErrorKind::InvalidInput,
                "bad usage",
            )));
        }
        let mount_source = match &self.config.source {
            Some(Source::Device(dev)) => dev
                .to_str()
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid device path"))?
                .to_string(),
            Some(Source::Label(label)) => {
                let label_str = label
                    .to_str()
                    .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid label"))?;
                let dev = find_device_by_label(label_str)?;
                dev
            }
            Some(Source::UUID(uuid)) => {
                let uuid_str = uuid
                    .to_str()
                    .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid UUID"))?;
                let dev = find_device_by_uuid(uuid_str)?;
                dev
            }
            None => String::default(),
        };
        if self.config.target.is_none() {
            return Ok(());
        }
        let target = &self
            .config
            .target
            .as_ref()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "No target specified!"))?
            .to_str()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid target path!"))
            .unwrap();
        let (flags, options) = self.parse_options()?;
        let fstype = if let Some(t) = self.config.options.types.as_ref().and_then(|t| t.to_str()) {
            Some(t.to_string())
        } else {
            let output = std::process::Command::new("blkid")
                .arg("-o")
                .arg("value")
                .arg("-s")
                .arg("TYPE")
                .arg(&mount_source)
                .output()?;
            let fs_type = String::from_utf8(output.stdout)?.trim().to_string();
            if fs_type.is_empty() {
                None
            } else {
                Some(fs_type)
            }
        };
        let data = None;
        let interal_only = self.use_internal_only();
        if self.is_fake_mode() {
            self.verbose_print(&format!(
                "FAKE: Would mount {} on {} with type {:?}, flags {:?}, and options {:?}",
                mount_source,
                target,
                fstype.unwrap(),
                flags,
                options
            ));
        } else {
            if !is_already_mounted(*target).unwrap() {
                let source = prepare_mount_source(&mount_source).unwrap();
                if self.config.show_labels {
                    if let Some(label) = self.get_filesystem_label(&source)? {
                        println!("Mounting filesystem, label:  {}", label);
                    }
                }
                mount_fs(Some(&source), &target.to_string(), Some(fstype.clone().unwrap().as_str()), flags, data,interal_only).map_err(|e| {
                    eprintln!("Mount failed:{:?}", e);
                    eprintln!("Source: {:?}, Target: {}, Filesystem type: {:?}, Flags: {:?}, Options: {:?}",
                              source, target, fstype, flags, options);
                    e
                })?;
                self.verbose_print(&format!("{} mounted on {}.", source, target));
            } else {
                println!("Already mounted!");
            }
        }
        Ok(())
    }
    #[allow(dead_code)]
    fn convert_uresult<T>(result: UResult<T>) -> Result<T, Box<dyn std::error::Error>> {
        result.map_err(|e| {
            Box::new(io::Error::new(io::ErrorKind::Other, e.to_string()))
                as Box<dyn std::error::Error>
        })
    }
    fn get_filesystem_label(
        &self,
        device: &str,
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let output = std::process::Command::new("blkid")
            .arg("-s")
            .arg("LABEL")
            .arg("-o")
            .arg("value")
            .arg(device)
            .output()?;

        if output.status.success() {
            let label = String::from_utf8(output.stdout)?.trim().to_string();
            if !label.is_empty() {
                Ok(Some(label))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    fn use_internal_only(&self) -> bool {
        self.config.internal_only
    }
    fn is_fake_mode(&self) -> bool {
        self.config.fake
    }
    fn should_fork(&self) -> bool {
        self.config.fork && self.config.all
    }
    fn parse_options(&self) -> Result<(MsFlags, Option<String>), Box<dyn std::error::Error>> {
        let mut flags = MsFlags::empty();
        let mut data = Vec::new();

        if self.config.options.read_only {
            flags |= MsFlags::MS_RDONLY;
        }

        if let Some(options) = &self.config.options.options {
            for option in options
                .to_str()
                .ok_or("Invalid UTF-8 in options")?
                .split(',')
            {
                match option {
                    "noexec" => flags |= MsFlags::MS_NOEXEC,
                    "nosuid" => flags |= MsFlags::MS_NOSUID,
                    "nodev" => flags |= MsFlags::MS_NODEV,
                    "sync" => flags |= MsFlags::MS_SYNCHRONOUS,
                    "dirsync" => flags |= MsFlags::MS_DIRSYNC,
                    "noatime" => flags |= MsFlags::MS_NOATIME,
                    "nodiratime" => flags |= MsFlags::MS_NODIRATIME,
                    "relatime" => flags |= MsFlags::MS_RELATIME,
                    "strictatime" => flags |= MsFlags::MS_STRICTATIME,
                    "lazytime" => flags |= MsFlags::MS_LAZYTIME,
                    "ro" => flags |= MsFlags::MS_RDONLY,
                    _ => data.push(option.to_string()),
                }
            }
        }

        let data_string = if data.is_empty() {
            None
        } else {
            Some(data.join(","))
        };

        Ok((flags, data_string))
    }
    fn enter_namespace(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ns) = &self.config.namespace {
            self.verbose_print(&format!("Entering namespace: {:?}", ns));

            let res = File::open(ns);
            if res.is_err() {
                return Ok(());
            }
            let ns_file = res.unwrap();

            // Use scopeguard to ensure the file descriptor is properly closed
            let _guard = scopeguard::guard(ns_file, |f| drop(f));
            setns(_guard.as_raw_fd(), CloneFlags::CLONE_NEWNS).map_err(|e| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to enter namespace: {}", e),
                )
            })?;
            self.verbose_print("Successfully entered the specified namespace");
        }
        Ok(())
    }
    fn perform_bind_mount(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.verbose_print("Performing bind mount");
        // Implement bind mount logic
        // Get source path
        let source = match &self.config.source {
            Some(Source::Device(dev)) => dev
                .to_str()
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid source path"))?,
            _ => {
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Bind mount requires a source path",
                )))
            }
        };

        // Get target path
        let target = self
            .config
            .target
            .as_ref()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "No target specified"))?
            .to_str()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid target path"))?;

        // Check if source and target paths exist
        if !Path::new(source).exists() {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Source path does not exist: {}", source),
            )));
        }
        if !Path::new(target).exists() {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Target path does not exist: {}", target),
            )));
        }

        // Set bind mount flags
        let mut flags = MsFlags::MS_BIND;

        // If recursive bind mount (rbind) is needed, add MS_REC flag
        if self.config.operation == Operation::RBind {
            flags |= MsFlags::MS_REC;
        }

        // Perform bind mount
        if self.is_fake_mode() {
            self.verbose_print(&format!("FAKE: Would bind mount {} to {}", source, target));
        } else {
            mount_fs(
                Some(&source.to_string()),
                &target.to_string(),
                None, // Bind mount doesn't require filesystem type
                flags,
                None, // Bind mount doesn't need extra data
                self.use_internal_only(),
            )?;
            self.verbose_print(&format!(
                "Successfully bind mounted {} to {}",
                source, target
            ));
        }
        Ok(())
    }

    fn perform_move_mount(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.verbose_print("Performing move mount operation");
        // Get source path
        let source = match &self.config.source {
            Some(Source::Device(dev)) => dev
                .to_str()
                .ok_or("Source device path contains invalid UTF-8 characters")?,
            Some(Source::Label(_)) | Some(Source::UUID(_)) => {
                return Err("Move operation doesn't support using labels or UUIDs".into())
            }
            None => return Err("Move operation requires specifying a source mount point".into()),
        };

        // Get target path
        let target = self
            .config
            .target
            .as_ref()
            .ok_or("Move operation requires specifying a target mount point")?
            .to_str()
            .ok_or("Target path contains invalid UTF-8 characters")?;

        // Check if source and target paths exist
        if !Path::new(source).exists() {
            return Err(format!("Source path does not exist: {}", source).into());
        }
        if !Path::new(target).exists() {
            return Err(format!("Target path does not exist: {}", target).into());
        }

        // Check if the source path is a mount point
        if !is_mount_point(source) {
            return Err(format!("Source path is not a mount point: {}", source).into());
        }
        let interal_only = self.config.internal_only;
        // Perform move mount operation
        match mount_fs(
            Some(&source.to_string()),
            &target.to_string(),
            None,
            MsFlags::MS_MOVE,
            None,
            interal_only,
        ) {
            Ok(_) => {
                self.verbose_print(&format!(
                    "Successfully moved mount point from {} to {}",
                    source, target
                ));
                Ok(())
            }
            Err(e) => {
                Err(format!("Move mount failed: {} -> {}, Error: {}", source, target, e).into())
            }
        }
    }
    #[allow(dead_code)]
    fn perform_rbind_mount(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.verbose_print("Performing rbind mount");
        // Implement the logic of recursive binding mounting
        //Implemented in rbind
        Ok(())
    }

    fn make_mount_shared(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.verbose_print("Making mount shared");
        // Implement logic for setting shared mount
        self.change_mount_propagation(MsFlags::MS_SHARED, false, "shared")
    }

    fn make_mount_slave(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.verbose_print("Making mount slave");
        // Implement logic for setting slave mount
        self.change_mount_propagation(MsFlags::MS_SLAVE, false, "slave")
    }

    fn make_mount_private(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.verbose_print("Making mount private");
        // Implement logic for setting private mount
        self.change_mount_propagation(MsFlags::MS_PRIVATE, false, "private")
    }

    fn make_mount_unbindable(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.verbose_print("Making mount unbindable");
        // Implement logic for setting unbindable mount
        self.change_mount_propagation(MsFlags::MS_UNBINDABLE, false, "unbindable")
    }

    fn make_mount_rshared(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.verbose_print("Making mount recursively shared");
        // Implement logic for setting recursive shared mount
        self.change_mount_propagation(MsFlags::MS_SHARED, true, "recursively shared")
    }

    fn make_mount_rslave(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.verbose_print("Making mount recursively slave");
        // Implement logic for setting recursive slave mount
        self.change_mount_propagation(MsFlags::MS_SLAVE, true, "recursively slave")
    }

    fn make_mount_rprivate(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.verbose_print("Making mount recursively private");
        // Implement logic for setting recursive private mount
        self.change_mount_propagation(MsFlags::MS_PRIVATE, true, "recursively private")
    }

    fn make_mount_runbindable(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.verbose_print("Making mount recursively unbindable");
        // Implement logic for setting recursive unbindable mount
        self.change_mount_propagation(MsFlags::MS_UNBINDABLE, true, "recursively unbindable")
    }

    fn change_mount_propagation(
        &self,
        flag: MsFlags,
        recursive: bool,
        prop_type: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let target = self
            .config
            .target
            .as_ref()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "No target specified"))?
            .to_str()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid target path"))?;

        if !Path::new(target).exists() {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Target path does not exist: {}", target),
            )));
        }

        let mut flags = flag;
        if recursive {
            flags |= MsFlags::MS_REC;
        }

        if self.is_fake_mode() {
            self.verbose_print(&format!(
                "FAKE: Would change mount propagation of {} to {}",
                target, prop_type
            ));
        } else {
            mount_fs(
                None,
                &target.to_string(),
                None,
                flags,
                None,
                self.use_internal_only(),
            )?;
            self.verbose_print(&format!(
                "Successfully changed mount propagation of {} to {}",
                target, prop_type
            ));
        }

        Ok(())
    }
    fn print_all(&self) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open("/etc/mtab")?;
        let reader = BufReader::new(file);
        for lineres in reader.lines() {
            if let Ok(line) = lineres {
                let mut ls = line.split(' ');
                let mut src = ls.next().unwrap_or_default().to_string();
                let targ = ls.next().unwrap_or_default();
                let types = ls.next();
                let optstr = ls.next();
                if types.is_some()
                    && self.config.options.types.is_some()
                    && types.as_deref().unwrap().to_ascii_lowercase()
                        != self
                            .config
                            .options
                            .types
                            .as_deref()
                            .unwrap()
                            .to_ascii_lowercase()
                            .to_string_lossy()
                {
                    continue;
                }
                if src.starts_with("/dev/loop") {
                    let loopname = src.rsplit_once('/').unwrap().1;
                    if loopname.starts_with("loop") {
                        if let Ok(mut file) =
                            File::open(format!("/sys/block/{}/loop/autoclear", loopname))
                        {
                            let mut buf = [0 as u8; 1];
                            file.read(&mut buf).ok();
                            if buf[0] == '1' as u8 {
                                let mut file = File::open(format!(
                                    "/sys/block/{}/loop/backing_file",
                                    loopname
                                ))?;
                                src = String::default();
                                file.read_to_string(&mut src).ok();
                                src.pop();
                            }
                        }
                    }
                }
                print!("{} on {}", src, targ);
                if let Some(typev) = types {
                    print!(" type {}", typev);
                }
                if let Some(optstrv) = optstr {
                    print!(" ({})", optstrv);
                }
                print!("\n");
            }
        }
        Ok(())
    }
}

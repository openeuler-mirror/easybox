//! This file is part of the easybox package.
//
// (c) Wentong Yang <ywt0821@163.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.
use clap::{crate_version, Arg, Command};
use nix::{
    errno::Errno,
    fcntl::{flock, open, FlockArg, OFlag},
    sys::stat::{fstat, stat, Mode, SFlag},
    unistd::{
        access, close, fchown, linkat, sysconf, unlinkat, AccessFlags, Gid, Group, LinkatFlags,
        SysconfVar, Uid, UnlinkatFlags, User,
    },
};
use std::{
    ffi::CString,
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, Error, Read, Seek, SeekFrom, Write},
    os::unix::{ffi::OsStrExt, fs::PermissionsExt, io::AsRawFd},
    path::{Path, PathBuf},
};
use uucore::{
    error::{UResult, USimpleError},
    format_usage,
};

use crate::utils::{
    chkname::is_valid_user_name,
    chowndir::chown_tree,
    constants::*,
    copydir::copy_tree,
    get_uid::get_uid,
    getdef::{getdef_bool, getdef_num, getdef_str},
    gettime::*,
    gshadow::Sgrp,
    list::{add_list, del_list, is_on_list},
    prefix_flag::{
        prefix_getgr_nam_gid, prefix_getpwnam, prefix_getpwuid, prefix_getspnam, read_all_groups,
        read_all_sgrp,
    },
    shadow::Spwd,
    strtoday::strtoday,
    subordinateio::{sub_gid_add, sub_gid_remove, sub_uid_add, sub_uid_remove},
    user_busy::user_busy,
};
use crate::utils::{prefix_flag::process_prefix_flag, root_flag::process_root_flag};

/// success
const E_SUCCESS: i32 = 0;
/// can't update password file
const E_PW_UPDATE: i32 = 1;
/// invalid command syntax
const E_USAGE: i32 = 2;
/// invalid argument to option
const E_BAD_ARG: i32 = 3;
/// UID already in use (and no -o)
const E_UID_IN_USE: i32 = 4;
/// passwd file contains errors (not used)
const _E_BAD_PWFILE: i32 = 5;
/// specified user/group doesn't exist
const E_NOTFOUND: i32 = 6;
/// user to modify is logged in
const E_USER_BUSY: i32 = 8;
/// username or group name already in use
const E_NAME_IN_USE: i32 = 9;
/// can't update group file
const E_GRP_UPDATE: i32 = 10;
/// insufficient space to move home dir (not used)
const _E_NOSPACE: i32 = 11;
/// unable to complete home dir move
const E_HOMEDIR: i32 = 12;
/// can't update SELinux user mapping
const _E_SE_UPDATE: i32 = 13;
/// can't update the subordinate uid file
const E_SUB_UID_UPDATE: i32 = 16;
/// can't update the subordinate gid file
const E_SUB_GID_UPDATE: i32 = 18;

/// If the shadow file does not exist, it won't be created
const SHADOW_PASSWD_STRING: &str = "x";

/// Command Options
pub mod options {
    /// -R, --root CHROOT_DIR
    pub static ROOT: &str = "root";
    ///
    pub static USER_NAME: &str = "USER_NAME";
    ///
    pub static COMMENT: &str = "comment";
    ///
    pub static PREFIX: &str = "prefix";
    ///
    pub static HOME: &str = "home";
    ///
    pub const EXPIRE_DATE: &str = "expiredate";
    ///
    pub const INACTIVE: &str = "inactive";
    ///
    pub const SHELL: &str = "shell";
    ///
    pub const LOGIN: &str = "login";
    ///
    pub const BADNAME: &str = "badname";
    ///
    pub const PASSWORD: &str = "password";
    /// -a, --append
    pub const APPEND: &str = "append";
    /// -g, --gid GROUP
    pub const GID: &str = "gid";
    /// -G, --groups GROUPS
    pub const GROUPS: &str = "groups";
    /// -L, --lock
    pub const LOCK: &str = "lock";
    /// -m --move-home
    pub const MOVE_HOME: &str = "move-home";
    /// -o, --non-unique
    pub const NON_UNIQUE: &str = "non-unique";
    /// -r, --remove
    pub const REMOVE: &str = "remove";
    /// -u, --uid UID
    pub const UID: &str = "uid";
    /// -U, --unlock
    pub const UNLOCK: &str = "unlock";
    /// -v, --add-subuids FIRST-LAST
    pub const ADD_SUBUIDS: &str = "add-subuids";
    /// -V, --del-subuids FIRST-LAST
    pub const DEL_SUBUIDS: &str = "del-subuids";
    /// -w, --add-subgids FIRST-LAST
    pub const ADD_SUBGIDS: &str = "add-subgids";
    /// -W, --del-subgids FIRST-LAST
    pub const DEL_SUBGIDS: &str = "del-subgids";
}

/// Configuration structure for usermod
pub struct Config {
    /// Username to modify
    pub user_name: String,
    /// New username
    pub user_newname: Option<String>,
    /// New encrypted password
    pub user_pass: Option<String>,
    /// User ID
    pub user_id: Option<Uid>,
    /// New User ID
    pub user_newid: Option<Uid>,
    /// Group ID
    pub user_gid: Option<Gid>,
    /// Group name string
    pub grname_string: Option<String>,
    /// New Group ID
    pub user_newgid: Option<Gid>,
    /// User comment (GECOS field)
    pub user_comment: Option<CString>,
    /// New user comment
    pub user_newcomment: Option<CString>,
    /// User home directory
    pub user_home: Option<PathBuf>,
    /// New user home directory
    pub user_newhome: Option<PathBuf>,
    /// User shell
    pub user_shell: Option<PathBuf>,
    /// New user shell
    pub user_newshell: Option<PathBuf>,
    /// Account expiration date
    pub user_expire: Option<i64>,
    /// New account expiration date
    pub user_newexpire: Option<i64>,
    /// Password inactive after expiration
    pub user_inactive: Option<i64>,
    /// New password inactive after expiration
    pub user_newinactive: Option<i64>,
    /// List of user groups
    pub user_groups: Vec<String>,
    /// User groups as a comma-separated string
    pub user_groups_string: Option<String>,
    /// Prefix directory
    pub prefix: Option<String>,
    /// User home directory with prefix
    pub prefix_user_home: Option<PathBuf>,
    /// New user home directory with prefix
    pub prefix_user_newhome: Option<PathBuf>,
    /// Ranges to add for subordinate UIDs
    pub add_subuids_range_vec: Option<Vec<String>>,
    /// Ranges to delete for subordinate UIDs
    pub del_subuids_range_vec: Option<Vec<String>>,
    /// Ranges to add for subordinate GIDs
    pub add_subgids_range_vec: Option<Vec<String>>,
    /// Ranges to delete for subordinate GIDs
    pub del_subgids_range_vec: Option<Vec<String>>,
    /// Linked list of ranges to add for subordinate UIDs
    pub add_sub_uids: Option<Box<UlongRangeListEntry>>,
    /// Linked list of ranges to delete for subordinate UIDs
    pub del_sub_uids: Option<Box<UlongRangeListEntry>>,
    /// Linked list of ranges to add for subordinate GIDs
    pub add_sub_gids: Option<Box<UlongRangeListEntry>>,
    /// Linked list of ranges to delete for subordinate GIDs
    pub del_sub_gids: Option<Box<UlongRangeListEntry>>,
    /// List of command-line arguments
    pub arg_list: Vec<String>,
    /// Usage string
    pub usage: String,
    /// Flags
    pub flags: Flags,
    /// Paths to configuration files
    pub paths: Paths,
    /// System settings
    pub settings: SystemSettings,
}

/// Lock status for various files
pub struct LockStatus {
    ///
    pw_locked: Option<File>,
    ///
    spw_locked: Option<File>,
    ///
    gr_locked: Option<File>,
    ///
    sgr_locked: Option<File>,
    ///
    sub_uid_locked: Option<File>,
    ///
    sub_gid_locked: Option<File>,
}

impl LockStatus {
    fn new() -> Self {
        LockStatus {
            pw_locked: None,
            spw_locked: None,
            gr_locked: None,
            sgr_locked: None,
            sub_uid_locked: None,
            sub_gid_locked: None,
        }
    }
}

///
#[derive(Debug)]
#[allow(non_snake_case)]
pub struct Flags {
    /// Append to existing secondary group set
    aflg: bool,
    /// Allow bad names
    bflg: bool,
    /// New comment (GECOS) field
    cflg: bool,
    /// New home directory
    dflg: bool,
    /// Account expiration date
    eflg: bool,
    /// Password inactive after expiration
    fflg: bool,
    /// New primary group ID
    gflg: bool,
    /// New secondary group set
    Gflg: bool,
    /// Lock the password
    Lflg: bool,
    /// New user name
    lflg: bool,
    /// Move home directory if it doesn't exist
    mflg: bool,
    /// Allow duplicate (non-unique) UID
    oflg: bool,
    /// New encrypted password
    pflg: bool,
    /// Prefix directory
    Pflg: bool,
    /// Remove a user from a single group
    rflg: bool,
    /// New shell program
    sflg: bool,
    /// Specify new user ID
    uflg: bool,
    /// Unlock the password
    Uflg: bool,
    /// New SELinux user
    Zflg: bool,
    // Add flags for SUBIDS if needed
    vflg: bool,
    Vflg: bool,
    wflg: bool,
    Wflg: bool,
}

/// System settings
pub struct SystemSettings {
    ///
    pub is_shadow_pwd: bool,
    ///
    pub is_shadow_grp: bool,
    ///
    pub is_sub_uid: bool,
    ///
    pub is_sub_gid: bool,
    ///
    pub sys_ngroups: i32,
}

/// Paths structure to manage file paths
#[derive(Default, Debug)]
pub struct Paths {
    ///
    pub passwd_db_file: String,
    ///
    pub shadow_db_file: String,
    ///
    pub group_db_file: String,
    ///
    pub gshadow_db_file: String,
    ///
    pub default_db_file: String,
    ///
    pub login_defs_db_file: String,
    ///
    pub subuid_db_file: Option<String>,
    ///
    pub subgid_db_file: Option<String>,
}

/// Represents a range of unsigned long integers
#[derive(Debug, Clone, Copy)]
pub struct UlongRange {
    ///
    pub first: u32,
    ///
    pub last: u32,
}

///
impl UlongRange {
    ///
    fn new() -> Self {
        Self {
            first: u32::MAX,
            last: 0,
        }
    }
}

/// Entry in a linked list of UlongRange structures
#[derive(Debug)]
pub struct UlongRangeListEntry {
    ///
    pub next: Option<Box<UlongRangeListEntry>>,
    ///
    pub range: UlongRange,
}

///
impl UlongRangeListEntry {
    ///
    fn new(range: UlongRange) -> Self {
        Self { next: None, range }
    }
}

///
impl Config {
    /// Create a Config from command-line options
    pub fn from(options: &clap::ArgMatches, arg_list: Vec<String>, usage: String) -> UResult<Self> {
        let flags = Flags::new().update(options);

        let mut user_newcomment = None;
        if let Some(comment) = options.value_of(options::COMMENT).map(|s| s.to_string()) {
            if !Self::is_valid(&comment) {
                return Err(USimpleError::new(E_BAD_ARG, "Invalid comment".to_string()));
            }
            match CString::new(comment) {
                Ok(cstring) => {
                    user_newcomment = Some(cstring);
                }
                Err(_) => {
                    return Err(USimpleError::new(
                        E_BAD_ARG,
                        "Invalid comment with null bytes".to_string(),
                    ));
                }
            }
        }

        let user_pass = options.value_of(options::PASSWORD).map(|s| s.to_string());

        let mut user_newid: Option<Uid> = None;
        if let Some(user_newid_str) = options.value_of(options::UID) {
            let res = get_uid(user_newid_str);
            if let Some(res) = res {
                user_newid = Some(res);
            } else {
                return Err(USimpleError::new(
                    E_BAD_ARG,
                    format!("invalid user ID '{}'", user_newid_str),
                ));
            }
        }

        let prefix = options.value_of(options::PREFIX).map(|s| s.to_string());

        let grname_string = options.value_of(options::GID).map(|s| s.to_string());

        let mut user_newhome: Option<PathBuf> = None;
        if let Some(home) = options.value_of(options::HOME).map(|s| PathBuf::from(s)) {
            if !Self::is_valid_path(&home) {
                return Err(USimpleError::new(
                    E_BAD_ARG,
                    format!("invalid field '{}'", home.display()),
                ));
            }
            if !home.is_absolute() {
                return Err(USimpleError::new(
                    E_BAD_ARG,
                    "homedir must be an absolute path",
                ));
            }
            user_newhome = Some(home);
        }

        let mut user_newexpire = None;
        if let Some(new_expire) = options.value_of(options::EXPIRE_DATE) {
            let res = strtoday(new_expire)?;
            if res < -1 {
                return Err(
                    USimpleError::new(E_BAD_ARG, format!("invalid date '{}'", new_expire)).into(),
                );
            }
            user_newexpire = Some(res);
        }

        let mut user_newinactive = None;
        if let Some(inactive) = options.value_of(options::INACTIVE) {
            match inactive.parse::<i64>() {
                Ok(value) if value >= -1 => {
                    user_newinactive = Some(value);
                }
                _ => {
                    return Err(USimpleError::new(
                        E_BAD_ARG,
                        format!("invalid numeric argument '{}'", inactive),
                    ));
                }
            }
        }

        let user_groups_string = options.value_of(options::GROUPS).map(|s| s.to_string());

        let mut user_newname = None;
        if let Some(new_name) = options.value_of(options::LOGIN).map(|s| s.to_string()) {
            if !is_valid_user_name(&new_name, options.is_present(options::BADNAME)) {
                return Err(USimpleError::new(
                    E_BAD_ARG,
                    format!("invalid user name '{}': use --badname to ignore", new_name),
                ));
            }
            user_newname = Some(new_name);
        }

        let mut user_newshell: Option<PathBuf> = None;
        if let Some(shell) = options.value_of(options::SHELL).map(PathBuf::from) {
            if !Self::is_valid_shell(&shell) {
                return Err(USimpleError::new(
                    E_BAD_ARG,
                    format!("invalid shell '{}'", shell.display()),
                ));
            }

            let shell_bytes = shell.as_os_str().as_bytes();
            if !shell_bytes.is_empty()
                && shell_bytes[0] != b'\0'
                && shell_bytes[0] != b'*'
                && shell != PathBuf::from("/sbin/nologin")
            {
                match stat(&shell) {
                    Ok(file_stat) => {
                        if SFlag::from_bits_truncate(file_stat.st_mode).contains(SFlag::S_IFDIR)
                            || !shell.exists()
                            || shell
                                .metadata()
                                .map(|m| m.permissions().mode() & 0o111 == 0)
                                .unwrap_or(true)
                        {
                            eprintln!(
                                "{}: Warning: missing or non-executable shell '{}'",
                                "usermod",
                                shell.display()
                            );
                        }
                    }
                    Err(_) => eprintln!(
                        "{}: Warning: missing or non-executable shell '{}'",
                        "usermod",
                        shell.display()
                    ),
                }
            }

            user_newshell = Some(shell);
        }

        let add_subuids_range_vec: Option<Vec<String>> = options
            .values_of(options::ADD_SUBUIDS)
            .map(|vals| vals.map(String::from).collect());
        let del_subuids_range_vec: Option<Vec<String>> = options
            .values_of(options::DEL_SUBUIDS)
            .map(|vals| vals.map(String::from).collect());

        let add_subgids_range_vec: Option<Vec<String>> = options
            .values_of(options::ADD_SUBGIDS)
            .map(|vals| vals.map(String::from).collect());
        let del_subgids_range_vec: Option<Vec<String>> = options
            .values_of(options::DEL_SUBGIDS)
            .map(|vals| vals.map(String::from).collect());

        let mut paths = Paths::new();
        paths.update_path(prefix.clone());

        Ok(Self {
            user_name: options.value_of(options::USER_NAME).unwrap().to_string(),
            user_newname,
            user_pass,
            user_id: None,
            user_newid,
            user_gid: None,
            grname_string,
            user_newgid: None,
            user_comment: None,
            user_newcomment,
            user_home: None,
            user_newhome,
            user_shell: None,
            user_newshell,
            user_expire: None,
            user_newexpire,
            user_inactive: None,
            user_newinactive,
            user_groups: Vec::new(),
            user_groups_string,
            prefix,
            prefix_user_home: None,
            prefix_user_newhome: None,
            add_subuids_range_vec,
            del_subuids_range_vec,
            add_subgids_range_vec,
            del_subgids_range_vec,
            add_sub_uids: None,
            del_sub_uids: None,
            add_sub_gids: None,
            del_sub_gids: None,
            arg_list,
            usage,
            flags,
            paths,
            settings: SystemSettings::new(),
        })
    }

    fn is_valid(comment: &str) -> bool {
        comment.chars().all(|c| c != ':' && c != '\n')
    }

    fn is_valid_path(path: &PathBuf) -> bool {
        let os_str = path.as_os_str();
        let bytes = os_str.as_bytes();
        !bytes.contains(&b':') && !bytes.contains(&b'\n')
    }

    fn is_valid_shell(shell: &PathBuf) -> bool {
        let shell_bytes = shell.as_os_str().as_bytes();
        !shell_bytes.contains(&b':')
            && !shell_bytes.contains(&b'\n')
            && (shell_bytes.is_empty() || shell_bytes[0] == b'/' || shell_bytes[0] == b'*')
    }
}

///
impl Paths {
    /// Create a new Paths structure
    pub fn new() -> Self {
        Self {
            passwd_db_file: DEFAULT_PASSWD_DB_FILE.to_string(),
            shadow_db_file: DEFAULT_SHADOW_DB_FILE.to_string(),
            group_db_file: DEFAULT_GROUP_DB_FILE.to_string(),
            gshadow_db_file: DEFAULT_GSHADOW_DB_FILE.to_string(),
            default_db_file: DEFAULT_USERADD_DB_FILE.to_string(),
            login_defs_db_file: DEFAULT_LOGIN_DEFS_DB_FILE.to_string(),
            subuid_db_file: if Path::new(DEFAULT_SUBUID_DB_FILE).exists() {
                Some(DEFAULT_SUBUID_DB_FILE.to_string())
            } else {
                None
            },
            subgid_db_file: if Path::new(DEFAULT_SUBGID_DB_FILE).exists() {
                Some(DEFAULT_SUBGID_DB_FILE.to_string())
            } else {
                None
            },
        }
    }

    /// Construct the base path with the given prefix and file name
    fn base_path(prefix_dir: Option<&String>, file_name: &str) -> String {
        if let Some(prefix) = prefix_dir {
            format!(
                "{}/{}",
                prefix.trim_end_matches('/'),
                file_name.trim_start_matches('/')
            )
        } else {
            file_name.to_string()
        }
    }

    /// Update the paths with the given prefix
    pub fn update_path(&mut self, prefix: Option<String>) {
        let prefix_ref = prefix.as_ref();

        self.passwd_db_file = Self::base_path(prefix_ref, "/etc/passwd");
        self.group_db_file = Self::base_path(prefix_ref, "/etc/group");
        self.shadow_db_file = Self::base_path(prefix_ref, "/etc/shadow");
        self.gshadow_db_file = Self::base_path(prefix_ref, "/etc/gshadow");
        self.default_db_file = Self::base_path(prefix_ref, "/etc/default/useradd");
        self.login_defs_db_file = Self::base_path(prefix_ref, "/etc/login.defs");

        if let Some(subuid_db) = &mut self.subuid_db_file {
            *subuid_db = Self::base_path(prefix_ref, "/etc/subuid");
        }

        if let Some(subgid_db) = &mut self.subgid_db_file {
            *subgid_db = Self::base_path(prefix_ref, "/etc/subgid");
        }
    }
}

impl SystemSettings {
    /// Create a new SystemSettings
    pub fn new() -> Self {
        Self {
            is_shadow_pwd: false,
            is_shadow_grp: false,
            is_sub_uid: false,
            is_sub_gid: false,
            sys_ngroups: 65536,
        }
    }

    /// Check if the shadow password file is present
    pub fn spw_file_present(shadow_db_file_path: &str) -> bool {
        if getdef_bool("FORCE_SHADOW") {
            return true;
        }

        Self::commonio_present(shadow_db_file_path)
    }

    /// Check if the shadow group file is present
    pub fn sgr_file_present(gshadow_db_file_path: &str) -> bool {
        if getdef_bool("FORCE_SHADOW") {
            return true;
        }

        Self::commonio_present(gshadow_db_file_path)
    }

    /// Check if the subordinate UID file is present
    pub fn sub_uid_file_present(subuid_db_file_path: &str) -> bool {
        Self::commonio_present(subuid_db_file_path)
    }

    /// Check if the subordinate GID file is present
    pub fn sub_gid_file_present(subgid_db_file_path: &str) -> bool {
        Self::commonio_present(subgid_db_file_path)
    }

    /// Check if a file is present
    pub fn commonio_present(file_path: &str) -> bool {
        Path::new(file_path).exists()
    }

    /// Get the system's maximum number of groups
    pub fn get_sys_ngroups() -> Option<i32> {
        match sysconf(SysconfVar::NGROUPS_MAX) {
            Ok(Some(value)) => Some(value as i32),
            Ok(None) => None,
            Err(_) => None,
        }
    }
}
///
impl Flags {
    /// Init a new Flags
    fn new() -> Self {
        Self {
            aflg: false,
            bflg: false,
            cflg: false,
            dflg: false,
            eflg: false,
            fflg: false,
            gflg: false,
            Gflg: false,
            Lflg: false,
            lflg: false,
            mflg: false,
            oflg: false,
            pflg: false,
            Pflg: false,
            rflg: false,
            sflg: false,
            uflg: false,
            Uflg: false,
            Zflg: false,
            vflg: false,
            Vflg: false,
            wflg: false,
            Wflg: false,
        }
    }

    /// Update the flags
    fn update(mut self, options: &clap::ArgMatches) -> Flags {
        self.cflg = options.is_present(options::COMMENT);
        self.eflg = options.is_present(options::EXPIRE_DATE);
        self.dflg = options.is_present(options::HOME);
        self.fflg = options.is_present(options::INACTIVE);
        self.lflg = options.is_present(options::LOGIN);
        self.Pflg = options.is_present(options::PREFIX);
        self.sflg = options.is_present(options::SHELL);
        self.pflg = options.is_present(options::PASSWORD);
        self.bflg = options.is_present(options::BADNAME);
        self.Lflg = options.is_present(options::LOCK);
        self.Uflg = options.is_present(options::UNLOCK);
        self.gflg = options.is_present(options::GID);
        self.uflg = options.is_present(options::UID);
        self.mflg = options.is_present(options::MOVE_HOME);
        self.Gflg = options.is_present(options::GROUPS);
        self.oflg = options.is_present(options::NON_UNIQUE);
        self.aflg = options.is_present(options::APPEND);
        self.rflg = options.is_present(options::REMOVE);
        self.vflg = options.is_present(options::ADD_SUBUIDS);
        self.Vflg = options.is_present(options::DEL_SUBUIDS);
        self.wflg = options.is_present(options::ADD_SUBGIDS);
        self.Wflg = options.is_present(options::DEL_SUBGIDS);
        self
    }
}

/// Parse the cmd args
pub fn parse_usermod_cmd_args(
    args: impl uucore::Args,
    about: &str,
    usage: &str,
) -> UResult<Config> {
    let arg_list = args.collect_lossy();
    let mut command = usermod_app(about, usage);
    let mut usage_doc = Vec::new();
    command.write_help(&mut usage_doc).unwrap();

    let usage = String::from_utf8_lossy(&usage_doc).into_owned();

    let matches = command.get_matches_from(
        &arg_list
            .iter()
            .map(std::convert::AsRef::as_ref)
            .collect::<Vec<&str>>(),
    );

    Config::from(&matches, arg_list, usage)
}

/// Create the command-line application for usermod
pub fn usermod_app<'a>(about: &'a str, usage: &'a str) -> Command<'a> {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about(about)
        .override_usage(format_usage(usage))
        .infer_long_args(true)
        .arg(
            Arg::new(options::USER_NAME)
                .value_name("USER_NAME")
                .required(true)
                .help("Specify the username to modify"),
        )
        .arg(
            Arg::new(options::ROOT)
                .short('R')
                .long("root")
                .value_name("CHROOT_DIR")
                .takes_value(true)
                .multiple_occurrences(true)
                .help("Set the root directory"),
        )
        .arg(
            Arg::new(options::COMMENT)
                .short('c')
                .long(options::COMMENT)
                .value_name("COMMENT")
                .takes_value(true)
                .help("new value of the GECOS field"),
        )
        .arg(
            Arg::new(options::PREFIX)
                .short('P')
                .long(options::PREFIX)
                .value_name("PREFIX_DIR")
                .help("prefix directory where are located the /etc/* files")
                .takes_value(true),
        )
        .arg(
            Arg::new(options::HOME)
                .short('d')
                .long(options::HOME)
                .value_name("HOME_DIR")
                .help("new home directory for the user account")
                .takes_value(true),
        )
        .arg(
            Arg::new(options::EXPIRE_DATE)
                .short('e')
                .long(options::EXPIRE_DATE)
                .value_name("EXPIRE_DATE")
                .help("set account expiration date to EXPIRE_DATE")
                .takes_value(true),
        )
        .arg(
            Arg::new(options::INACTIVE)
                .short('f')
                .long(options::INACTIVE)
                .value_name("INACTIVE")
                .help("set password inactive after expiration to INACTIVE")
                .takes_value(true)
                .allow_hyphen_values(true),
        )
        .arg(
            Arg::new(options::SHELL)
                .short('s')
                .long(options::SHELL)
                .value_name("SHELL")
                .help("new login shell for the user account")
                .takes_value(true),
        )
        .arg(
            Arg::new(options::LOGIN)
                .short('l')
                .long(options::LOGIN)
                .value_name("NEW_LOGIN")
                .help("new value of the login name")
                .takes_value(true),
        )
        .arg(
            Arg::new(options::BADNAME)
                .short('b')
                .long(options::BADNAME)
                .help("allow bad names")
                .takes_value(true),
        )
        .arg(
            Arg::new(options::PASSWORD)
                .short('p')
                .long(options::PASSWORD)
                .help("use encrypted password for the new password")
                .takes_value(true),
        )
        .arg(
            Arg::new(options::LOCK)
                .short('L')
                .long(options::LOCK)
                .help("lock the user account"),
        )
        .arg(
            Arg::new(options::UNLOCK)
                .short('U')
                .long(options::UNLOCK)
                .help("unlock the user account"),
        )
        .arg(
            Arg::new(options::GID)
                .short('g')
                .long(options::GID)
                .help("force use GROUP as new primary group")
                .takes_value(true),
        )
        .arg(
            Arg::new(options::UID)
                .short('u')
                .long(options::UID)
                .help("new UID for the user account")
                .takes_value(true),
        )
        .arg(
            Arg::new(options::MOVE_HOME)
                .short('m')
                .long(options::MOVE_HOME)
                .help("move contents of the home directory to the new location (use only with -d)"),
        )
        .arg(
            Arg::new(options::GROUPS)
                .short('G')
                .long(options::GROUPS)
                .help("new list of supplementary GROUPS")
                .takes_value(true),
        )
        .arg(
            Arg::new(options::NON_UNIQUE)
                .short('o')
                .long(options::NON_UNIQUE)
                .help("allow using duplicate (non-unique) UID"),
        )
        .arg(
            Arg::new(options::APPEND)
                .short('a')
                .long(options::APPEND)
                .help("append the user to the supplemental GROUPS mentioned by the -G option without removing the user from other groups"),
        )
        .arg(
            Arg::new(options::REMOVE)
                .short('r')
                .long(options::REMOVE)
                .help("remove the user from only the supplemental GROUPS mentioned by the -G option without removing the user from other groups"),
        )
        .arg(
            Arg::new(options::ADD_SUBUIDS)
                .short('v')
                .long(options::ADD_SUBUIDS)
                .value_name("FIRST-LAST")
                .help("add range of subordinate uids")
                .multiple_occurrences(true)
                .takes_value(true),
        )
        .arg(
            Arg::new(options::DEL_SUBUIDS)
                .short('V')
                .long(options::DEL_SUBUIDS)
                .value_name("FIRST-LAST")
                .help("remove range of subordinate uids")
                .multiple_occurrences(true)
                .takes_value(true),
        )
        .arg(
            Arg::new(options::ADD_SUBGIDS)
                .short('w')
                .long(options::ADD_SUBGIDS)
                .value_name("FIRST-LAST")
                .help("add range of subordinate gids")
                .multiple_occurrences(true)
                .takes_value(true),
        )
        .arg(
            Arg::new(options::DEL_SUBGIDS)
                .short('W')
                .long(options::DEL_SUBGIDS)
                .value_name("FIRST-LAST")
                .help("remove range of subordinate gids")
                .multiple_occurrences(true)
                .takes_value(true),
        )
}

fn new_pw_passwd(mut pw_pass: String, config: &Config) -> UResult<String> {
    if config.flags.Lflg && !pw_pass.starts_with('!') {
        pw_pass = format!("!{}", pw_pass);
    } else if config.flags.Uflg && pw_pass.starts_with('!') {
        if pw_pass.len() == 1 {
            return Err(USimpleError::new(
                1,
                format!(
                    "{}: unlocking the user's password would result in a passwordless account.\nYou should set a password with usermod -p to unlock this user's password.\n",
                    "usermod"
                ),
            ));
        }
        pw_pass = pw_pass[1..].to_string();
    } else if config.flags.pflg {
        if let Some(ref new_password) = config.user_pass {
            pw_pass = new_password.clone();
        } else {
            return Err(USimpleError::new(
                1,
                "No password provided with -p option.".to_string(),
            ));
        }
    }

    Ok(pw_pass)
}

fn get_groups(config: &mut Config) -> bool {
    let mut groups: Vec<String> = Vec::new();
    let mut ngroups = 0;
    let mut errors = 0;
    let user_groups_string = config.user_groups_string.clone().unwrap();
    if user_groups_string.is_empty() {
        return true;
    }

    let list_parts: Vec<&str> = user_groups_string.split(',').collect();
    for list_part in list_parts {
        let grp = prefix_getgr_nam_gid(
            list_part,
            config.prefix.clone(),
            config.paths.group_db_file.clone(),
        );
        if grp.is_none() {
            eprintln!("{}: group '{}' does not exist", "usermod", list_part);
            errors += 1;
            continue;
        }

        if ngroups == config.settings.sys_ngroups {
            eprintln!("{}: too many groups specified (max {}).", "Prog", ngroups);
            break;
        }
        groups.push(grp.unwrap().name);
        ngroups += 1;
    }

    if errors != 0 {
        return false;
    }
    config.user_groups = groups;
    true
}

fn new_pwent(mut pwent: User, config: &Config) -> UResult<User> {
    //modify the name
    if config.flags.lflg {
        if let Some(newname) = &config.user_newname {
            let existing_user = prefix_getpwnam(
                newname.as_str(),
                config.prefix.clone(),
                config.paths.passwd_db_file.clone(),
            );
            if existing_user.is_some() {
                return Err(USimpleError::new(
                    E_NAME_IN_USE,
                    format!("user '{}' already exists", newname),
                ));
            }

            pwent.name = newname.clone();
        }
    }

    if config.settings.is_shadow_pwd
        || pwent.passwd.to_string_lossy().into_owned() != SHADOW_PASSWD_STRING
    {
        pwent.passwd = CString::new(new_pw_passwd(
            pwent.passwd.to_string_lossy().into_owned(),
            config,
        )?)
        .map_err(|e| {
            USimpleError::new(
                E_PW_UPDATE,
                format!("Failed to convert password to CString: {}", e),
            )
        })?;
    }

    if config.flags.uflg {
        if let Some(user_newid) = &config.user_newid {
            pwent.uid = *user_newid;
        }
    }

    if config.flags.gflg {
        if let Some(user_newgid) = &config.user_newgid {
            pwent.gid = *user_newgid;
        }
    }

    // modify the GECOS
    if config.flags.cflg {
        if let Some(new_gecos) = &config.user_newcomment {
            pwent.gecos = new_gecos.clone();
        }
    }

    if config.flags.dflg {
        let mut new_home: PathBuf = config.user_newhome.as_ref().unwrap().clone();
        if new_home.as_os_str().as_bytes().len() > 1
            && new_home.as_os_str().as_bytes().last() == Some(&b'/')
        {
            new_home.pop();
        }

        pwent.dir = new_home;
    }

    if config.flags.sflg {
        pwent.shell = config.user_newshell.as_ref().unwrap().clone();
    }

    Ok(pwent)
}

fn new_spent(mut spent: Spwd, config: &Config, file: &mut File) -> UResult<Spwd> {
    if config.flags.lflg {
        if let Some(newname) = &config.user_newname {
            let existing_user = spw_locate(file, newname)?;
            if existing_user.is_some() {
                return Err(USimpleError::new(
                    E_NAME_IN_USE,
                    format!("user '{}' already exists", newname),
                ));
            }

            spent.sp_namp = newname.clone();
        }
    }

    if config.flags.fflg {
        spent.sp_inact = config.user_newinactive;
    }

    if config.flags.eflg {
        spent.sp_expire = config.user_newexpire;
    }

    spent.sp_pwdp = new_pw_passwd(spent.sp_pwdp, config)?;

    if config.flags.pflg {
        let current_time = gettime()?;
        let lstchg = current_time / DAY;
        if lstchg != 0 {
            spent.sp_lstchg = Some(lstchg);
        } else {
            spent.sp_lstchg = None;
        }
    }

    Ok(spent)
}

/// Update the group file
pub fn update_group(lock_status: &mut LockStatus, config: &mut Config) -> UResult<()> {
    let mut changed = false;
    let groups = read_all_groups(&config.paths.group_db_file)?;
    for grp in groups.iter() {
        let was_member = grp.mem.contains(&config.user_name);
        let mut is_member = config.flags.Gflg
            && ((was_member && config.flags.aflg) || config.user_groups.contains(&grp.name));
        if !was_member && !is_member {
            continue;
        }
        if config.flags.Gflg && config.flags.rflg {
            is_member = !is_member;
        }
        let mut ngrp = grp.clone();
        if was_member {
            if !config.flags.Gflg || is_member {
                if config.flags.lflg {
                    del_list(&mut ngrp.mem, &config.user_name);
                    add_list(&mut ngrp.mem, &config.user_newname.as_ref().unwrap());
                    changed = true;
                }
            } else {
                del_list(&mut ngrp.mem, &config.user_name);
                changed = true;
            }
        } else if is_member {
            add_list(&mut ngrp.mem, &config.user_newname.as_ref().unwrap());
            changed = true;
        }
        if !changed {
            continue;
        }
        changed = false;

        if let Some(ref mut gr_lock) = lock_status.gr_locked {
            if gr_update(gr_lock, &ngrp).is_err() {
                return Err(USimpleError::new(
                    E_GRP_UPDATE,
                    format!(
                        "failed to prepare the new {} entry '{}'",
                        config.paths.group_db_file, ngrp.name
                    ),
                ));
            }
        } else {
            return Err(USimpleError::new(
                E_GRP_UPDATE,
                "Group file is not locked or opened.".to_string(),
            ));
        }
    }

    Ok(())
}

/// Update the shadow password file
pub fn update_gshadow(lock_status: &mut LockStatus, config: &mut Config) -> UResult<()> {
    let mut changed = false;
    let gshadow_groups = read_all_sgrp(&config.paths.gshadow_db_file)?;
    for sgrp in gshadow_groups.iter() {
        let was_member = is_on_list(&sgrp.sg_adm, &config.user_name);
        let was_admin = is_on_list(&sgrp.sg_adm, &config.user_name);
        let mut is_member = config.flags.Gflg
            && ((was_member && config.flags.aflg)
                || is_on_list(&config.user_groups, &sgrp.sg_name));

        if !was_member && !was_admin && !is_member {
            continue;
        }

        if config.flags.Gflg && config.flags.rflg {
            is_member = !is_member;
        }

        let mut nsgrp = sgrp.clone();

        if was_admin && config.flags.lflg {
            del_list(&mut nsgrp.sg_adm, &config.user_name);
            add_list(&mut nsgrp.sg_adm, &config.user_newname.as_ref().unwrap());
            changed = true;
        }

        if was_member {
            if !config.flags.Gflg || is_member {
                if config.flags.lflg {
                    del_list(&mut nsgrp.sg_mem, &config.user_name);
                    add_list(&mut nsgrp.sg_mem, &config.user_newname.as_ref().unwrap());
                    changed = true;
                }
            } else {
                del_list(&mut nsgrp.sg_mem, &config.user_name);
                changed = true;
            }
        } else if is_member {
            add_list(&mut nsgrp.sg_mem, &config.user_newname.as_ref().unwrap());
            changed = true;
        }

        if !changed {
            continue;
        }

        changed = false;

        if let Some(ref mut sgr_lock) = lock_status.sgr_locked {
            if sgr_update(sgr_lock, &nsgrp).is_err() {
                return Err(USimpleError::new(
                    E_GRP_UPDATE,
                    format!(
                        "failed to prepare the new {} entry '{}'",
                        config.paths.gshadow_db_file, nsgrp.sg_name
                    ),
                ));
            }
        } else {
            return Err(USimpleError::new(
                E_GRP_UPDATE,
                "Shadow group file is not locked or opened.".to_string(),
            ));
        }
    }

    Ok(())
}

fn format_user_entry(user: &User) -> String {
    format!(
        "{}:{}:{}:{}:{}:{}:{}",
        user.name,
        user.passwd.to_string_lossy(),
        user.uid,
        user.gid,
        user.gecos.to_string_lossy(),
        user.dir.display(),
        user.shell.display()
    )
}

fn format_spwd_entry(spwd: &Spwd) -> String {
    format!(
        "{}:{}:{}:{}:{}:{}:{}:{}:{}",
        spwd.sp_namp,
        spwd.sp_pwdp,
        Spwd::format_optional(spwd.sp_lstchg),
        Spwd::format_optional(spwd.sp_min),
        Spwd::format_optional(spwd.sp_max),
        Spwd::format_optional(spwd.sp_warn),
        Spwd::format_optional(spwd.sp_inact),
        Spwd::format_optional(spwd.sp_expire),
        Spwd::format_optional(spwd.sp_flag),
    )
}

fn format_group_entry(group: &Group) -> String {
    let members = group
        .mem
        .iter()
        .filter(|s| !s.is_empty())
        .cloned()
        .collect::<Vec<String>>()
        .join(",");
    format!(
        "{}:{}:{}:{}",
        group.name,
        group.passwd.to_string_lossy(),
        group.gid.as_raw(),
        members
    )
}

///
fn format_sgrp_entry(sgrp: &Sgrp) -> String {
    format!(
        "{}:{}:{}:{}",
        sgrp.sg_name,
        sgrp.sg_passwd,
        sgrp.sg_adm.join(","),
        sgrp.sg_mem.join(",")
    )
}

///
fn update_file_entry(file: &mut File, user_name: &str, updated_entry: &str) -> Result<(), Error> {
    let mut file_content = String::new();
    file.seek(SeekFrom::Start(0))?;
    file.read_to_string(&mut file_content)?;

    let mut lines: Vec<String> = file_content.lines().map(|line| line.to_string()).collect();
    let mut user_found = false;

    for line in &mut lines {
        if line.starts_with(&format!("{}:", user_name)) {
            *line = updated_entry.to_string();
            user_found = true;
            break;
        }
    }

    if !user_found {
        lines.push(updated_entry.to_string());
    }

    file.seek(SeekFrom::Start(0))?;
    file.set_len(0)?;
    file.write_all(lines.join("\n").as_bytes())?;
    file.write_all(b"\n")?;
    file.sync_all()?;

    Ok(())
}

///
fn pw_update(file: &mut File, updated_pwent: &User) -> Result<(), std::io::Error> {
    let updated_entry = format_user_entry(&updated_pwent);
    update_file_entry(file, &updated_pwent.name, &updated_entry)
}

///
fn spw_update(spw_file: &mut File, updated_spwd: &Spwd) -> Result<(), Error> {
    let updated_entry = format_spwd_entry(updated_spwd);
    update_file_entry(spw_file, &updated_spwd.sp_namp, &updated_entry)
}

///
fn gr_update(gr_file: &mut File, update_group: &Group) -> Result<(), Error> {
    let updated_entry = format_group_entry(update_group);
    update_file_entry(gr_file, &update_group.name, &updated_entry)
}

///
fn sgr_update(file: &mut File, updated_sgrp: &Sgrp) -> Result<(), Error> {
    let updated_entry = format_sgrp_entry(updated_sgrp);
    update_file_entry(file, &updated_sgrp.sg_name, &updated_entry)
}

/// Finds the shadow password entry for the specified username
fn spw_locate(spw_file: &mut File, user_name: &str) -> Result<Option<Spwd>, Error> {
    let reader = BufReader::new(spw_file);

    for line in reader.lines() {
        let line = line?;
        if let Some(spwd) = parse_spwd_line(&line) {
            if spwd.sp_namp == user_name {
                return Ok(Some(spwd));
            }
        }
    }

    Ok(None)
}

/// Parse a line from a shadow password file into a `Spwd` structure
fn parse_spwd_line(line: &str) -> Option<Spwd> {
    let parts: Vec<&str> = line.split(':').collect();
    if parts.len() < 9 {
        return None;
    }

    Some(Spwd {
        sp_namp: parts[0].to_string(),
        sp_pwdp: parts[1].to_string(),
        sp_lstchg: parts[2].parse::<i64>().ok(),
        sp_min: parts[3].parse::<i64>().ok(),
        sp_max: parts[4].parse::<i64>().ok(),
        sp_warn: parts[5].parse::<i64>().ok(),
        sp_inact: parts[6].parse::<i64>().ok(),
        sp_expire: parts[7].parse::<i64>().ok(),
        sp_flag: parts[8].parse::<i64>().ok(),
    })
}

///
fn spw_remove(spw_file: &mut File, user_name: &str) -> Result<(), Error> {
    let mut file_content = String::new();
    spw_file.seek(SeekFrom::Start(0))?;
    spw_file.read_to_string(&mut file_content)?;

    let mut lines: Vec<String> = file_content.lines().map(|line| line.to_string()).collect();
    lines.retain(|line| !line.starts_with(&format!("{}:", user_name)));

    spw_file.seek(SeekFrom::Start(0))?;
    spw_file.set_len(0)?;
    spw_file.write_all(lines.join("\n").as_bytes())?;
    spw_file.write_all(b"\n")?;
    spw_file.sync_all()?;

    Ok(())
}

///
fn pw_remove(pw_file: &mut File, user_name: &str) -> Result<(), Error> {
    let mut file_content = String::new();
    pw_file.seek(SeekFrom::Start(0))?;
    pw_file.read_to_string(&mut file_content)?;

    let mut lines: Vec<String> = file_content.lines().map(|line| line.to_string()).collect();
    lines.retain(|line| !line.starts_with(&format!("{}:", user_name)));

    pw_file.seek(SeekFrom::Start(0))?;
    pw_file.set_len(0)?;
    pw_file.write_all(lines.join("\n").as_bytes())?;
    pw_file.write_all(b"\n")?;
    pw_file.sync_all()?;

    Ok(())
}

///
fn open_files(config: &mut Config) -> UResult<LockStatus> {
    let mut lock_status = LockStatus::new();
    let pw_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&config.paths.passwd_db_file)
        .map_err(|e| {
            USimpleError::new(
                E_PW_UPDATE,
                format!("Failed to open {}: {}", &config.paths.passwd_db_file, e),
            )
        })?;

    let pw_fd = pw_file.as_raw_fd();
    flock(pw_fd, FlockArg::LockExclusive).map_err(|_| {
        USimpleError::new(E_GRP_UPDATE, "Cannot lock password file; try again later.")
    })?;
    lock_status.pw_locked = Some(pw_file);

    if config.settings.is_shadow_pwd {
        let spw_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&config.paths.shadow_db_file)
            .map_err(|e| {
                USimpleError::new(
                    E_PW_UPDATE,
                    format!("Failed to open {}: {}", &config.paths.shadow_db_file, e),
                )
            })?;
        let spw_fd = spw_file.as_raw_fd();
        flock(spw_fd, FlockArg::LockExclusive).map_err(|_| {
            USimpleError::new(
                E_GRP_UPDATE,
                "Cannot lock shadow group file; try again later.",
            )
        })?;

        lock_status.spw_locked = Some(spw_file);
    }

    if config.flags.Gflg || config.flags.lflg {
        let gr_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&config.paths.group_db_file)
            .map_err(|e| {
                USimpleError::new(
                    E_GRP_UPDATE,
                    format!("Failed to open {}: {}", &config.paths.group_db_file, e),
                )
            })?;
        let gr_fd = gr_file.as_raw_fd();
        flock(gr_fd, FlockArg::LockExclusive).map_err(|_| {
            USimpleError::new(E_GRP_UPDATE, "Cannot lock group file; try again later.")
        })?;
        lock_status.gr_locked = Some(gr_file);

        if config.settings.is_shadow_grp {
            let sgr_file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(&config.paths.gshadow_db_file)
                .map_err(|e| {
                    USimpleError::new(
                        E_GRP_UPDATE,
                        format!("Failed to open {}: {}", &config.paths.gshadow_db_file, e),
                    )
                })?;
            let sgr_fd = sgr_file.as_raw_fd();
            flock(sgr_fd, FlockArg::LockExclusive).map_err(|_| {
                USimpleError::new(
                    E_GRP_UPDATE,
                    "Cannot lock gshadow group file; try again later.",
                )
            })?;

            lock_status.sgr_locked = Some(sgr_file);
        }
    }

    if (config.flags.vflg || config.flags.Vflg) && config.settings.is_sub_uid {
        let sub_uid_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&config.paths.subuid_db_file.as_ref().unwrap())
            .map_err(|e| {
                USimpleError::new(
                    E_SUB_UID_UPDATE,
                    format!(
                        "Failed to open {}: {}",
                        config.paths.subuid_db_file.as_ref().unwrap(),
                        e
                    ),
                )
            })?;
        let sub_uid_fd = sub_uid_file.as_raw_fd();
        flock(sub_uid_fd, FlockArg::LockExclusive).map_err(|_| {
            USimpleError::new(E_GRP_UPDATE, "Cannot lock subuid file; try again later.")
        })?;

        lock_status.sub_uid_locked = Some(sub_uid_file);
    }

    if config.flags.wflg || config.flags.Wflg {
        let sub_gid_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&config.paths.subgid_db_file.as_ref().unwrap())
            .map_err(|e| {
                USimpleError::new(
                    E_SUB_GID_UPDATE,
                    format!(
                        "Failed to open {}: {}",
                        config.paths.subgid_db_file.as_ref().unwrap(),
                        e
                    ),
                )
            })?;
        let sub_gid_fd = sub_gid_file.as_raw_fd();
        flock(sub_gid_fd, FlockArg::LockExclusive).map_err(|_| {
            USimpleError::new(E_GRP_UPDATE, "Cannot lock subgid file; try again later.")
        })?;

        lock_status.sub_gid_locked = Some(sub_gid_file);
    }

    Ok(lock_status)
}

///
fn usr_update(lock_status: &mut LockStatus, config: &mut Config) -> UResult<()> {
    let mut spwd: Option<Spwd> = None;
    let pwd = prefix_getpwnam(
        config.user_name.as_str(),
        config.prefix.clone(),
        config.paths.passwd_db_file.clone(),
    );
    if pwd.is_none() {
        return Err(USimpleError::new(
            E_NOTFOUND,
            format!(
                "user '{}' does not exist in {}",
                config.user_name,
                config.paths.passwd_db_file.clone()
            ),
        ));
    }

    let pwent = pwd.unwrap();
    let mut updated_pwent = new_pwent(pwent, config)?;

    let mut spent = Spwd::new();

    if config.settings.is_shadow_pwd {
        if let Some(ref mut spw_lock) = lock_status.spw_locked {
            spwd = spw_locate(spw_lock, config.user_name.as_str())?;

            if let Some(existing_spwd) = spwd {
                spwd = Some(new_spent(existing_spwd, config, spw_lock)?);
            } else if config.flags.pflg
                && updated_pwent.passwd.to_str().unwrap_or("") == SHADOW_PASSWD_STRING
                || config.flags.eflg
                || config.flags.fflg
            {
                let mut new_spwd = Spwd::new();
                new_spwd.sp_namp = config.user_name.clone();
                new_spwd.sp_pwdp = updated_pwent.passwd.to_str().unwrap_or("").to_string();
                updated_pwent.passwd = CString::new(SHADOW_PASSWD_STRING).unwrap();

                let current_time = gettime()?;
                let lstchg = current_time / DAY;
                if lstchg != 0 {
                    spent.sp_lstchg = Some(lstchg);
                } else {
                    spent.sp_lstchg = None;
                }
                new_spwd.sp_min = Some(getdef_num("PASS_MIN_DAYS", -1));
                new_spwd.sp_max = Some(getdef_num("PASS_MAX_DAYS", -1));
                new_spwd.sp_warn = Some(getdef_num("PASS_WARN_AGE", -1));
                new_spwd.sp_inact = None;
                new_spwd.sp_expire = None;
                new_spwd.sp_flag = None;
                let new_spwd = new_spent(new_spwd, config, spw_lock)?;
                spwd = Some(new_spwd);
            }
        }
    }

    if config.flags.lflg
        || config.flags.uflg
        || config.flags.gflg
        || config.flags.cflg
        || config.flags.dflg
        || config.flags.sflg
        || config.flags.pflg
        || config.flags.Lflg
        || config.flags.Uflg
    {
        if let Some(ref mut pw_lock) = lock_status.pw_locked {
            if pw_update(pw_lock, &updated_pwent).is_err() {
                return Err(USimpleError::new(
                    E_PW_UPDATE,
                    format!(
                        "failed to prepare the new {} entry '{}'",
                        config.paths.passwd_db_file, updated_pwent.name
                    ),
                ));
            }
            if config.flags.lflg && pw_remove(pw_lock, config.user_name.as_str()).is_err() {
                return Err(USimpleError::new(
                    E_PW_UPDATE,
                    format!(
                        "cannot remove entry '{}' from {}",
                        config.user_name, config.paths.passwd_db_file
                    ),
                ));
            }
        } else {
            return Err(USimpleError::new(
                E_PW_UPDATE,
                "Password file is not locked or opened.".to_string(),
            ));
        }
    }

    if let Some(updated_spwd) = spwd {
        if config.flags.lflg
            || config.flags.eflg
            || config.flags.fflg
            || config.flags.pflg
            || config.flags.Lflg
            || config.flags.Uflg
        {
            if let Some(ref mut spw_lock) = lock_status.spw_locked {
                if spw_update(spw_lock, &updated_spwd).is_err() {
                    return Err(USimpleError::new(
                        E_PW_UPDATE,
                        format!(
                            "Failed to update the shadow password entry for user '{}'",
                            config.user_name
                        ),
                    ));
                }
            }
            if let Some(ref mut spw_lock) = lock_status.spw_locked {
                if config.flags.lflg && spw_remove(spw_lock, config.user_name.as_str()).is_err() {
                    return Err(USimpleError::new(
                        E_PW_UPDATE,
                        format!(
                            "cannot remove entry '{}' from {}",
                            config.user_name, config.paths.shadow_db_file
                        ),
                    ));
                }
            }
        }
    }
    Ok(())
}

///
pub fn grp_update(lock_status: &mut LockStatus, config: &mut Config) -> UResult<()> {
    update_group(lock_status, config)?;
    if config.settings.is_shadow_grp {
        update_gshadow(lock_status, config)?;
    }
    Ok(())
}

///
pub fn process_flags(config: &mut Config) -> UResult<()> {
    // get the set the user information
    let pwd = prefix_getpwnam(
        config.user_name.as_str(),
        config.prefix.clone(),
        config.paths.passwd_db_file.clone(),
    );

    if let Some(pwd) = pwd {
        config.user_id = Some(pwd.uid);
        config.user_gid = Some(pwd.gid);
        config.user_comment = Some(pwd.gecos);
        config.user_home = Some(pwd.dir);
        config.user_shell = Some(pwd.shell);
    } else {
        return Err(USimpleError::new(
            E_NOTFOUND,
            format!(
                "user '{}' does not exist in {}",
                config.user_name,
                config.paths.passwd_db_file.clone()
            ),
        ));
    }

    /* user_newname, user_newid, user_newgid can be used even when the
     * options where not specified. */
    if !config.flags.lflg {
        config.user_newname = Some(config.user_name.clone());
    }

    if !config.flags.uflg {
        config.user_newid = config.user_id;
    }

    if !config.flags.gflg {
        config.user_newgid = config.user_gid;
    }

    if let Some(prefix) = &config.prefix {
        let join_path = |base: &str, path: &PathBuf| -> PathBuf {
            let mut result = PathBuf::from(base);
            if path.is_absolute() {
                if let Ok(stripped) = path.strip_prefix("/") {
                    result.push(stripped);
                }
            } else {
                result.push(path);
            }
            result
        };

        if let Some(user_home) = &config.user_home {
            config.prefix_user_home = Some(join_path(prefix, user_home));
        }

        if let Some(user_newhome) = &config.user_newhome {
            config.prefix_user_newhome = Some(join_path(prefix, user_newhome));
        }
    } else {
        config.prefix_user_home = config.user_home.clone();
        config.prefix_user_newhome = config.user_newhome.clone();
    }

    if config.settings.is_shadow_pwd {
        if let Some(spwd) = prefix_getspnam(
            config.user_name.as_str(),
            config.prefix.clone(),
            config.paths.shadow_db_file.clone(),
        ) {
            config.user_expire = spwd.sp_expire;
            config.user_inactive = spwd.sp_inact;
        }
    }

    if config.flags.aflg && !config.flags.Gflg {
        return Err(USimpleError::new(
            E_USAGE,
            format!(
                "{} flag is only allowed with the {} flag\n{}",
                "-a", "-G", config.usage
            ),
        ));
    }

    if config.flags.rflg && !config.flags.Gflg {
        return Err(USimpleError::new(
            E_USAGE,
            format!(
                "{} flag is only allowed with the {} flag\n{}",
                "-r", "-G", config.usage
            ),
        ));
    }

    if config.flags.rflg && config.flags.aflg {
        return Err(USimpleError::new(
            E_USAGE,
            format!(
                "{} and {} flags are mutually exclusive\n{}",
                "-r", "-a", config.usage
            ),
        ));
    }

    if (config.flags.Lflg && (config.flags.pflg || config.flags.Uflg))
        || (config.flags.pflg && config.flags.Uflg)
    {
        return Err(USimpleError::new(
            E_USAGE,
            format!(
                "The {}, {}, and {} flags are exclusive\n{}",
                "-L", "-p", "-U", config.usage
            ),
        ));
    }

    if config.flags.oflg && !config.flags.uflg {
        return Err(USimpleError::new(
            E_USAGE,
            format!(
                "{} flag is only allowed with the {} flag\n{}",
                "-o", "-u", config.usage
            ),
        ));
    }

    if config.flags.mflg && !config.flags.dflg {
        return Err(USimpleError::new(
            E_USAGE,
            format!(
                "{} flag is only allowed with the {} flag\n{}",
                "-m", "-d", config.usage
            ),
        ));
    }

    if config.user_newid == config.user_id {
        config.flags.uflg = false;
        config.flags.oflg = false;
    }

    if config.user_newgid == config.user_gid {
        config.flags.gflg = false;
    }

    if config.user_newshell.is_some() {
        if let (Some(user_shell), Some(user_newshell)) = (&config.user_shell, &config.user_newshell)
        {
            if user_shell == user_newshell {
                config.flags.sflg = false;
            }
        }
    }

    if let (Some(user_newname), user_name) = (&config.user_newname, &config.user_name) {
        if user_newname == user_name {
            config.flags.lflg = false;
        }
    }

    if let (Some(user_newinactive), Some(user_inactive)) =
        (&config.user_newinactive, &config.user_inactive)
    {
        if user_newinactive == user_inactive {
            config.flags.fflg = false;
        }
    }

    if let (Some(user_newexpire), Some(user_expire)) = (&config.user_newexpire, &config.user_expire)
    {
        if user_newexpire == user_expire {
            config.flags.eflg = false;
        }
    }

    if let (Some(new_home), Some(home)) = (&config.user_newhome, &config.user_home) {
        if new_home == home {
            config.flags.dflg = false;
            config.flags.mflg = false;
        }
    }

    if let (Some(ref new_comment), Some(ref old_comment)) =
        (&config.user_newcomment, &config.user_comment)
    {
        if new_comment.as_c_str() == old_comment.as_c_str() {
            config.flags.cflg = false;
        }
    }

    if !(config.flags.Uflg
        || config.flags.uflg
        || config.flags.sflg
        || config.flags.pflg
        || config.flags.mflg
        || config.flags.Lflg
        || config.flags.lflg
        || config.flags.Gflg
        || config.flags.gflg
        || config.flags.fflg
        || config.flags.eflg
        || config.flags.dflg
        || config.flags.cflg
        || ((config.flags.vflg || config.flags.Vflg) && config.settings.is_sub_uid)
        || ((config.flags.wflg || config.flags.Wflg) && config.settings.is_sub_gid)
        || config.flags.Zflg)
    {
        return Err(USimpleError::new(E_SUCCESS, "no changes"));
    }

    if !config.settings.is_shadow_pwd && (config.flags.eflg || config.flags.fflg) {
        return Err(USimpleError::new(
            E_USAGE,
            "shadow passwords required for -e and -f".to_string(),
        ));
    }

    /* local, no need for xgetpwnam */
    if config.flags.lflg {
        if let Some(newname) = &config.user_newname {
            let user = prefix_getpwnam(
                newname.as_str(),
                config.prefix.clone(),
                config.paths.passwd_db_file.clone(),
            );
            if user.is_some() {
                return Err(USimpleError::new(
                    E_NAME_IN_USE,
                    format!("user '{}' already exists", newname),
                ));
            }
        }
    }

    /* local, no need for xgetpwuid */
    if config.flags.uflg && !config.flags.oflg {
        if let Some(newid) = config.user_newid {
            let user = prefix_getpwuid(
                newid.into(),
                config.prefix.clone(),
                config.paths.passwd_db_file.clone(),
            );
            if user.is_some() {
                return Err(USimpleError::new(
                    E_UID_IN_USE,
                    format!("UID '{}' already exists", newid),
                ));
            }
        }
    }

    Ok(())
}

fn move_mailbox(config: &Config) -> UResult<()> {
    let maildir = getdef_str("MAIL_DIR");
    if maildir.is_none() {
        return Ok(());
    }

    let maildir = maildir.unwrap();
    let mut mailfile = match &config.prefix {
        Some(prefix) => {
            let mut path = PathBuf::from(prefix);
            if Path::new(maildir.as_str()).is_absolute() {
                path.push(maildir.strip_prefix("/").unwrap());
            } else {
                path.push(maildir.clone());
            }
            path
        }
        None => PathBuf::from(maildir.clone()),
    };
    mailfile.push(&config.user_name);

    let fd = open(
        &mailfile,
        OFlag::O_RDONLY | OFlag::O_NONBLOCK,
        Mode::empty(),
    )
    .map_err(|e| USimpleError::new(E_PW_UPDATE, format!("Failed to open mail file: {}", e)))?;

    let st = fstat(fd).map_err(|e| {
        close(fd).ok();
        USimpleError::new(E_PW_UPDATE, format!("Failed to stat mail file: {}", e))
    })?;

    if st.st_uid != config.user_id.unwrap().as_raw() {
        eprintln!(
            "warning: {} not owned by {}",
            mailfile.as_path().display(),
            config.user_name
        );
        close(fd).ok();
        return Ok(());
    }

    if config.flags.uflg {
        if let Some(new_uid) = config.user_newid {
            fchown(fd, Some(new_uid), None).map_err(|e| {
                close(fd).ok();
                USimpleError::new(
                    E_PW_UPDATE,
                    format!("Failed to change mail file owner: {}", e),
                )
            })?;
        }
    }

    close(fd).ok();

    if config.flags.lflg {
        if let Some(newname) = &config.user_newname {
            let mut newmailfile = match &config.prefix {
                Some(prefix) => {
                    let mut path = PathBuf::from(prefix);
                    if Path::new(maildir.as_str()).is_absolute() {
                        path.push(maildir.strip_prefix("/").unwrap());
                    } else {
                        path.push(maildir);
                    }
                    path
                }
                None => PathBuf::from(maildir),
            };
            newmailfile.push(newname);

            linkat(
                None,
                &mailfile,
                None,
                &newmailfile,
                LinkatFlags::NoSymlinkFollow,
            )
            .map_err(|e| {
                USimpleError::new(E_PW_UPDATE, format!("Failed to link mail file: {}", e))
            })?;
            unlinkat(None, &mailfile, UnlinkatFlags::NoRemoveDir).map_err(|e| {
                USimpleError::new(E_PW_UPDATE, format!("Failed to unlink mail file: {}", e))
            })?;
        }
    }

    Ok(())
}

///
fn change_home_directory_owner(config: &Config) -> UResult<()> {
    if !config.flags.mflg && (config.flags.uflg || config.flags.gflg) {
        let home_path = if config.flags.dflg {
            config.prefix_user_newhome.as_ref()
        } else {
            config.prefix_user_home.as_ref()
        };

        if let Some(home) = home_path {
            if let Ok(sb) = stat(home) {
                let user_newid = config.user_newid.unwrap();
                let user_id = config.user_id.unwrap();

                if (config.flags.uflg && sb.st_uid == user_newid.as_raw())
                    || sb.st_uid == user_id.as_raw()
                {
                    if let Err(err) = chown_tree(
                        home.to_str().unwrap_or_default(),
                        config.user_id,
                        if config.flags.uflg {
                            Some(user_newid)
                        } else {
                            None
                        },
                        config.user_gid,
                        if config.flags.gflg {
                            config.user_newgid
                        } else {
                            None
                        },
                    ) {
                        return Err(USimpleError::new(
                            E_HOMEDIR,
                            format!("Failed to change ownership of the home directory: {}", err),
                        ));
                    }
                }
            }
        }
    }

    Ok(())
}

fn move_home(config: &mut Config) -> UResult<()> {
    let user_home = config
        .prefix_user_home
        .as_ref()
        .ok_or_else(|| USimpleError::new(E_HOMEDIR, "Old home directory is not specified"))?;
    let user_newhome = config
        .prefix_user_newhome
        .as_ref()
        .ok_or_else(|| USimpleError::new(E_HOMEDIR, "New home directory is not specified"))?;

    let old_uid = config
        .user_id
        .ok_or_else(|| USimpleError::new(E_HOMEDIR, "Old user ID is not specified"))?;
    let new_uid = config
        .user_newid
        .ok_or_else(|| USimpleError::new(E_HOMEDIR, "New user ID is not specified"))?;
    let old_gid = config
        .user_gid
        .ok_or_else(|| USimpleError::new(E_HOMEDIR, "Old group ID is not specified"))?;
    let new_gid = config
        .user_newgid
        .ok_or_else(|| USimpleError::new(E_HOMEDIR, "New group ID is not specified"))?;

    if access(user_newhome, AccessFlags::F_OK).is_ok() {
        return Err(USimpleError::new(
            E_PW_UPDATE,
            format!(
                "cannot move home directory '{}' to '{}' because it is not empty",
                config.user_home.as_ref().unwrap().display(),
                config.user_newhome.as_ref().unwrap().display()
            ),
        ));
    }

    let old_home_stat = stat(user_home);
    if let Ok(stat) = old_home_stat {
        if !SFlag::from_bits_truncate(stat.st_mode).contains(SFlag::S_IFDIR) {
            return Err(USimpleError::new(
                E_PW_UPDATE,
                format!(
                    "The previous home directory ({}) was not a directory",
                    user_home.display()
                ),
            ));
        }

        match fs::rename(user_home, user_newhome) {
            Ok(_) => {
                chown_tree(
                    user_newhome.to_str().unwrap(),
                    config.user_id,
                    if config.flags.uflg {
                        Some(new_uid)
                    } else {
                        None
                    },
                    config.user_gid,
                    if config.flags.gflg {
                        Some(new_gid)
                    } else {
                        None
                    },
                )
                .map_err(|e| {
                    USimpleError::new(
                        E_HOMEDIR,
                        format!("Failed to change ownership of the home directory: {}", e),
                    )
                })?;
                return Ok(());
            }
            Err(err) => {
                if err.raw_os_error() == Some(Errno::EXDEV as i32) {
                    copy_tree(
                        user_home.to_str().unwrap(),
                        user_newhome.to_str().unwrap(),
                        true,
                        true,
                        Some(old_uid.as_raw()),
                        if config.flags.uflg {
                            Some(new_uid.as_raw())
                        } else {
                            None
                        },
                        Some(old_gid.as_raw()),
                        if config.flags.gflg {
                            Some(new_gid.as_raw())
                        } else {
                            None
                        },
                    )
                    .map_err(|e| {
                        USimpleError::new(
                            E_HOMEDIR,
                            format!("Failed to copy home directory: {}", e),
                        )
                    })?;
                    fs::remove_dir_all(user_home).map_err(|e| {
                        USimpleError::new(
                            E_HOMEDIR,
                            format!("Failed to remove old home directory: {}", e),
                        )
                    })?;
                    return Ok(());
                } else {
                    return Err(USimpleError::new(
                        E_HOMEDIR,
                        format!("Failed to rename home directory: {}", err),
                    ));
                }
            }
        }
    } else {
        return Err(USimpleError::new(
            E_HOMEDIR,
            format!(
                "Failed to stat the old home directory: {}",
                old_home_stat.err().unwrap()
            ),
        ));
    }
}

///
fn get_ulong_range(s: &str) -> UlongRange {
    let mut result = UlongRange::new();
    let parts: Vec<&str> = s.split('-').collect();

    if parts.len() != 2 {
        return result;
    }

    match (parts[0].parse::<u64>(), parts[1].parse::<u64>()) {
        (Ok(first), Ok(last))
            if first <= last && first <= u32::MAX as u64 && last <= u32::MAX as u64 =>
        {
            result.first = first as u32;
            result.last = last as u32;
        }
        _ => {}
    }

    result
}

///
fn prepend_range(s: &str, head: &mut Option<Box<UlongRangeListEntry>>) -> bool {
    let range = get_ulong_range(s);
    if range.first > range.last {
        return false;
    }

    let mut new_entry = Box::new(UlongRangeListEntry::new(range));
    new_entry.next = head.take();
    *head = Some(new_entry);
    true
}

///
pub fn usermod_main(config: &mut Config) -> UResult<()> {
    process_root_flag("-R", config.arg_list.clone())?;

    if config.prefix.is_some() {
        config.prefix = process_prefix_flag(config)?;
        config.paths.update_path(config.prefix.clone());
    }

    config.settings.is_shadow_pwd = SystemSettings::spw_file_present(&config.paths.shadow_db_file);
    config.settings.is_shadow_grp = SystemSettings::sgr_file_present(&config.paths.gshadow_db_file);
    if let Some(subuid_db_file) = &config.paths.subuid_db_file {
        config.settings.is_sub_uid = SystemSettings::sub_uid_file_present(subuid_db_file);
    }
    if let Some(subgid_db_file) = &config.paths.subgid_db_file {
        config.settings.is_sub_gid = SystemSettings::sub_uid_file_present(subgid_db_file);
    }

    if let Some(sys_ngroups_num) = SystemSettings::get_sys_ngroups() {
        config.settings.sys_ngroups = sys_ngroups_num;
    }

    if let Some(grname) = &config.grname_string {
        let grp = prefix_getgr_nam_gid(
            grname,
            config.prefix.clone(),
            config.paths.group_db_file.clone(),
        );
        if grp.is_none() {
            return Err(USimpleError::new(
                E_NOTFOUND,
                format!("group '{}' dose not exit", grname),
            ));
        } else {
            let grp = grp.unwrap();
            config.user_newgid = Some(grp.gid);
        }
    }
    if config.user_groups_string.is_some() {
        if !get_groups(config) {
            return Err(USimpleError::new(E_NOTFOUND, ""));
        }
    }

    process_flags(config)?;

    if config.prefix.as_deref().unwrap_or("").is_empty()
        && (config.flags.uflg
            || config.flags.lflg
            || config.flags.dflg
            || (config.settings.is_sub_uid && config.flags.Vflg)
            || config.flags.Wflg)
    {
        let user_name = &config.user_name;
        let user_id = config.user_id.map(|uid| uid.as_raw()).unwrap_or(0);

        if user_busy(user_name, user_id) {
            return Err(USimpleError::new(
                E_USER_BUSY,
                format!("User {} is currently logged in", user_name),
            ));
        }
    }

    let mut lock_status = open_files(config)?;

    if config.flags.cflg
        || config.flags.dflg
        || config.flags.eflg
        || config.flags.fflg
        || config.flags.gflg
        || config.flags.Lflg
        || config.flags.lflg
        || config.flags.pflg
        || config.flags.sflg
        || config.flags.uflg
        || config.flags.Uflg
    {
        usr_update(&mut lock_status, config)?;
    }

    if config.flags.Gflg || config.flags.lflg {
        grp_update(&mut lock_status, config)?;
    }

    if config.flags.vflg && config.settings.is_sub_uid {
        let add_subuids_range_vec = config.add_subuids_range_vec.clone().unwrap();
        for range in &add_subuids_range_vec {
            prepend_range(range, &mut config.add_sub_uids);
        }

        let mut current = &config.add_sub_uids;
        while let Some(entry) = current {
            let count = entry.range.last - entry.range.first + 1;

            if let Some(ref mut uid_file) = lock_status.sub_uid_locked {
                let uid_result: Result<bool, Error> =
                    sub_uid_add(uid_file, config.user_name.clone(), entry.range.first, count);

                if let Ok(false) = uid_result {
                    return Err(USimpleError::new(
                        E_SUB_UID_UPDATE,
                        format!(
                            "failed to add uid range {}-{} to '{}'",
                            entry.range.first,
                            entry.range.last,
                            config.paths.subuid_db_file.clone().unwrap()
                        ),
                    ));
                }
            }
            current = &entry.next;
        }
    }

    if config.flags.Vflg && config.settings.is_sub_uid {
        let del_subuids_range_vec = config.del_subuids_range_vec.clone().unwrap();
        for range in &del_subuids_range_vec {
            prepend_range(range, &mut config.del_sub_uids);
        }

        let mut current = &config.del_sub_uids;
        while let Some(entry) = current {
            let count = entry.range.last - entry.range.first + 1;

            if let Some(ref mut uid_file) = lock_status.sub_uid_locked {
                let uid_result: Result<bool, Error> =
                    sub_uid_remove(uid_file, config.user_name.clone(), entry.range.first, count);

                if let Ok(false) = uid_result {
                    return Err(USimpleError::new(
                        E_SUB_UID_UPDATE,
                        format!(
                            "failed to remove uid range {}-{} to '{}'",
                            entry.range.first,
                            entry.range.last,
                            config.paths.subuid_db_file.clone().unwrap()
                        ),
                    ));
                }
            }
            current = &entry.next;
        }
    }

    if config.flags.wflg && config.settings.is_sub_gid {
        let add_subgids_range_vec = config.add_subgids_range_vec.clone().unwrap();
        for range in &add_subgids_range_vec {
            prepend_range(range, &mut config.add_sub_gids);
        }

        let mut current = &config.add_sub_gids;
        while let Some(entry) = current {
            let count = entry.range.last - entry.range.first + 1;

            if let Some(ref mut gid_lock) = lock_status.sub_gid_locked {
                let gid_result: Result<bool, Error> =
                    sub_gid_add(gid_lock, config.user_name.clone(), entry.range.first, count);

                if let Ok(false) = gid_result {
                    return Err(USimpleError::new(
                        E_SUB_GID_UPDATE,
                        format!(
                            "failed to add gid range {}-{} to '{}'",
                            entry.range.first,
                            entry.range.last,
                            config.paths.subgid_db_file.clone().unwrap()
                        ),
                    ));
                }
            }
            current = &entry.next;
        }
    }

    if config.flags.Wflg && config.settings.is_sub_gid {
        let del_subgids_range_vec = config.del_subgids_range_vec.clone().unwrap();
        for range in &del_subgids_range_vec {
            prepend_range(range, &mut config.del_sub_gids);
        }

        let mut current = &config.del_sub_gids;
        while let Some(entry) = current {
            let count = entry.range.last - entry.range.first + 1;

            if let Some(ref mut gid_lock) = lock_status.sub_gid_locked {
                let gid_result: Result<bool, Error> =
                    sub_gid_remove(gid_lock, config.user_name.clone(), entry.range.first, count);

                if let Ok(false) = gid_result {
                    return Err(USimpleError::new(
                        E_SUB_GID_UPDATE,
                        format!(
                            "failed to remove gid range {}-{} to '{}'",
                            entry.range.first,
                            entry.range.last,
                            config.paths.subgid_db_file.clone().unwrap()
                        ),
                    ));
                }
            }
            current = &entry.next;
        }
    }

    if config.flags.mflg {
        move_home(config)?;
    }

    if config.flags.lflg || config.flags.uflg {
        move_mailbox(config)?;
    }

    change_home_directory_owner(&config)?;

    Ok(())
}

//! This file is part of the easybox package.
//
// (c) Wentong Yang <ywt0821@163.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use crate::utils::{
    chkname::*,
    constants::*,
    copydir::*,
    defaults::UserAddDefaults,
    find_new_gid::find_new_gid,
    find_new_sub_gids::find_new_sub_gids,
    find_new_sub_uids::find_new_sub_uids,
    find_new_uid::find_new_uid,
    getdef::*,
    passwd::Passwd,
    prefix_flag::*,
    root_flag::process_root_flag,
    shadow::Spwd,
    strtoday::strtoday,
    subordinateio::{local_sub_gid_assigned, local_sub_uid_assigned},
};
use clap::{crate_version, Arg, Command};
use nix::{
    fcntl::{flock, open, FlockArg, OFlag},
    libc::{gid_t, uid_t},
    sys::stat::{fchmod, Mode},
    unistd::{chown, close, fchown, fsync, sysconf, Gid, Group, SysconfVar, Uid, User},
};
use std::{
    collections::HashMap,
    env,
    fs::{self, File, OpenOptions},
    io::{self, BufRead, BufReader, BufWriter, Error, Read, Seek, SeekFrom, Write},
    os::unix::{fs::PermissionsExt, io::AsRawFd},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use uucore::{
    error::{UResult, USimpleError},
    format_usage,
};

///
const DAY: i64 = 24 * 3600;

/// success
const _E_SUCCESS: i32 = 0;
/// can't update password file
const E_PW_UPDATE: i32 = 1;
/// invalid command syntax
const E_USAGE: i32 = 2;
/// invalid argument to option
const E_BAD_ARG: i32 = 3;
// UID already in use (and no -o)
const E_UID_IN_USE: i32 = 4;
/// specified group doesn't exist
const E_NOTFOUND: i32 = 6;
/// username or group name already in use
const E_NAME_IN_USE: i32 = 9;
/// can't update group file
const E_GRP_UPDATE: i32 = 10;
/// can't create home directory
const E_HOMEDIR: i32 = 12;
/// can't create mailbox file
const _E_MAILBOXFILE: i32 = 13;
/// can't update SELinux user mapping
const _E_SE_UPDATE: i32 = 14;
/// can't update the subordinate uid file
const E_SUB_UID_UPDATE: i32 = 16;
/// can't update the subordinate gid file
const E_SUB_GID_UPDATE: i32 = 18;

///
#[derive(Debug)]
#[allow(non_snake_case)]
pub struct Flag {
    ///
    bflg: bool,
    ///
    cflg: bool,
    ///
    dflg: bool,
    ///
    Dflg: bool,
    ///
    eflg: bool,
    ///
    fflg: bool,
    ///
    Fflg: bool,
    ///
    gflg: bool,
    ///
    Gflg: bool,
    ///
    kflg: bool,
    ///
    mflg: bool,
    ///
    Mflg: bool,
    ///
    Nflg: bool,
    ///
    oflg: bool,
    ///
    rflg: bool,
    ///
    sflg: bool,
    ///
    uflg: bool,
    ///
    Uflg: bool,
    ///
    btrfs_flg: bool,
}

///
impl Flag {
    ///
    fn new() -> Self {
        Self {
            bflg: false,
            cflg: false,
            dflg: false,
            Dflg: false,
            eflg: false,
            fflg: false,
            Fflg: false,
            gflg: false,
            Gflg: false,
            kflg: false,
            mflg: false,
            Mflg: false,
            Nflg: false,
            oflg: false,
            rflg: false,
            sflg: false,
            uflg: false,
            Uflg: false,
            btrfs_flg: false,
        }
    }

    ///
    fn update(mut self, options: &clap::ArgMatches) -> Flag {
        self.bflg = options.is_present(options::BASE_DIR);
        self.cflg = options.is_present(options::COMMENT);
        self.dflg = options.is_present(options::HOME_DIR);
        self.Dflg = options.is_present(options::DEFAULTS);
        self.eflg = options.is_present(options::EXPIRE_DATE);
        self.fflg = options.is_present(options::INACTIVE);
        self.Fflg = options.is_present(options::ADD_SUBIDS_FOR_SYSTEM);
        self.gflg = options.is_present(options::GID);
        self.Gflg = options.is_present(options::GROUPS);
        self.kflg = options.is_present(options::SKEL);
        self.mflg = options.is_present(options::CREATE_HOME);
        self.Mflg = options.is_present(options::NO_CREATE_HOME);
        self.Nflg = options.is_present(options::NO_USER_GROUP);
        self.oflg = options.is_present(options::NON_UNIQUE);
        self.rflg = options.is_present(options::SYSTEM);
        self.sflg = options.is_present(options::SHELL);
        self.uflg = options.is_present(options::UID);
        self.Uflg = options.is_present(options::USER_GROUP);
        self.btrfs_flg = options.is_present(options::BTRFS_SUBVOLUME_HOME);
        self
    }
}
///
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

///
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

    ///
    pub fn spw_file_present(shadow_db_file_path: &str) -> bool {
        if getdef_bool("FORCE_SHADOW") {
            return true;
        }

        Self::commonio_present(shadow_db_file_path)
    }

    ///
    pub fn sgr_file_present(gshadow_db_file_path: &str) -> bool {
        if getdef_bool("FORCE_SHADOW") {
            return true;
        }

        Self::commonio_present(gshadow_db_file_path)
    }

    ///
    pub fn sub_uid_file_present(subuid_db_file_path: &str) -> bool {
        Self::commonio_present(subuid_db_file_path)
    }

    ///
    pub fn sub_gid_file_present(subgid_db_file_path: &str) -> bool {
        Self::commonio_present(subgid_db_file_path)
    }

    ///
    pub fn commonio_present(file_path: &str) -> bool {
        Path::new(file_path).exists()
    }

    ///
    pub fn get_sys_ngroups() -> Option<i32> {
        match sysconf(SysconfVar::NGROUPS_MAX) {
            Ok(Some(value)) => Some(value as i32),
            Ok(None) => None,
            Err(_) => None,
        }
    }
}

/// Config.
pub struct Config {
    ///
    pub flag: Flag,
    ///
    pub is_shadow_pwd: bool,
    ///
    pub usage: String,
    ///
    pub username: Option<String>,
    ///
    pub badname: bool,
    ///
    pub defaults: bool,
    ///
    pub user_gid: Option<gid_t>,
    ///
    pub user_gid_str: Option<String>,
    ///
    pub uid: Option<uid_t>,
    ///
    pub comment: Option<String>,
    ///
    pub home_dir: Option<String>,
    ///
    pub shell: Option<String>,
    ///
    pub expire_date: Option<i64>,
    ///
    pub system: bool,
    ///
    pub password: Option<String>,
    ///
    pub user_group: bool,
    ///
    pub create_home: bool,
    ///
    pub prefix: Option<String>,
    ///
    pub prefix_user_home: Option<String>,
    ///
    pub paths: Paths,
    ///
    pub home_added: bool, // is home crate
    ///
    pub create_mail_spool: Option<String>,
    ///
    pub inactive_days: Option<i64>,
    ///
    pub additional_groups: Option<Vec<String>>,
    ///
    pub chroot_dir: Option<String>,
    ///
    pub settings: SystemSettings,
    ///
    pub sub_uid_start: u32,
    ///
    pub sub_gid_start: u32,
    ///
    pub subuid_count: u32,
    ///
    pub subgid_count: u32,
    ///
    pub arg_list: Vec<String>,
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

///
#[derive(Debug)]
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

/// options
pub mod options {
    ///
    pub const USER_NAME: &str = "username";
    ///
    pub const BADNAME: &str = "badname";
    ///
    pub const DEFAULTS: &str = "defaults";
    ///
    pub const GID: &str = "gid";
    ///
    pub const UID: &str = "uid";
    ///
    pub const COMMENT: &str = "comment";
    ///
    pub const HOME_DIR: &str = "home-dir";
    ///
    pub const SHELL: &str = "shell";
    ///
    pub const EXPIRE_DATE: &str = "expiredate";
    ///
    pub const SYSTEM: &str = "system";
    ///
    pub const PASSWORD: &str = "password";
    ///
    ///
    pub const NON_UNIQUE: &str = "non-unique";
    ///
    pub const KEY: &str = "key";
    ///
    pub const USER_GROUP: &str = "user-group";
    ///
    pub const NO_USER_GROUP: &str = "no-user-group";
    ///
    pub const CREATE_HOME: &str = "create-home";
    ///
    pub const NO_CREATE_HOME: &str = "no-create-home";
    ///
    pub const SKEL: &str = "skel";
    ///
    pub const PREFIX: &str = "prefix";
    ///
    pub const BASE_DIR: &str = "base-dir";
    ///
    pub const INACTIVE: &str = "inactive";
    ///
    pub const GROUPS: &str = "groups";
    ///
    pub const CHROOT_DIR: &str = "root";
    ///
    pub const BTRFS_SUBVOLUME_HOME: &str = "btrfs-subvolume-home";
    ///
    pub const ADD_SUBIDS_FOR_SYSTEM: &str = "add-subids-for-system";
}
/// Config
impl Config {
    ///
    pub fn from(
        options: &clap::ArgMatches,
        defaults_data: &mut UserAddDefaults,
        flag: Flag,
        arg_list: Vec<String>,
        usage: String,
    ) -> UResult<Self> {
        let is_shadow_pwd = spw_file_present();

        let username = options.value_of(options::USER_NAME).map(|s| s.to_string());
        let badname = options.is_present(options::BADNAME);
        let defaults = options.is_present(options::DEFAULTS);

        let uid: Option<u32> = if let Some(val) = options.value_of(options::UID) {
            match val.parse::<u32>() {
                Ok(uid_value) => {
                    if uid_value == u32::MAX {
                        return Err(
                            USimpleError::new(E_BAD_ARG, format!("invalid uid '{}'", val)).into(),
                        );
                    }
                    Some(uid_value)
                }
                Err(_) => {
                    return Err(
                        USimpleError::new(E_BAD_ARG, format!("invalid uid '{}'", val)).into(),
                    )
                }
            }
        } else {
            None
        };

        let comment = options.value_of(options::COMMENT).map(|s| s.to_string());
        if let Some(comment) = &comment {
            if !Self::is_valid(comment) {
                return Err(
                    USimpleError::new(E_BAD_ARG, format!("invalid comment '{}'", comment)).into(),
                );
            }
        }

        // -d, --home-dir HOME_DIR
        let home_dir = if let Some(base_dir_str) = options.value_of(options::HOME_DIR) {
            if !Self::is_valid_dir_path(base_dir_str) {
                return Err(USimpleError::new(
                    E_BAD_ARG,
                    format!("invalid base directory '{}'", base_dir_str),
                )
                .into());
            }
            Some(base_dir_str.to_string())
        } else {
            None
        };

        let shell: Option<String> = options.value_of(options::SHELL).map(|s| s.to_string());
        if let Some(shell) = &shell {
            Self::is_valid_shell(shell)?;
            if flag.Dflg {
                defaults_data.set_def_shell(shell.clone());
            }
        }

        let expire_date =
            Self::parse_and_set_expire_date(options, is_shadow_pwd, flag.Dflg, defaults_data)?;
        if flag.Dflg {
            if let Some(expire) = expire_date {
                defaults_data.set_def_expire(expire.to_string());
            }
        }

        let system = options.is_present(options::SYSTEM);

        let password = options.value_of(options::PASSWORD).map(|s| s.to_string());
        if let Some(password) = &password {
            if !Self::is_valid(password) {
                return Err(
                    USimpleError::new(E_BAD_ARG, format!("invalid field '{}'", password)).into(),
                );
            }
        }

        let mut invalid_entries = Vec::new();
        if let Some(values) = options.values_of(options::KEY) {
            for kv in values {
                if kv.contains(',') {
                    invalid_entries.push(kv);
                    continue;
                }
                if let Some((key, value)) = kv.split_once('=') {
                    putdef_str(key, value)?;
                } else {
                    return Err(USimpleError::new(
                        E_BAD_ARG,
                        format!("Invalid KEY=VALUE format: {}", kv),
                    )
                    .into());
                }
            }
        }

        let user_group = options.is_present(options::USER_GROUP);

        let create_home = options.is_present(options::CREATE_HOME);

        if let Some(skel_value) = options.value_of(options::SKEL) {
            if !Self::is_valid_dir_path(skel_value) {
                return Err(USimpleError::new(
                    E_BAD_ARG,
                    format!("invalid skeleton directory '{}'", skel_value),
                )
                .into());
            }
            defaults_data.set_def_template(skel_value.to_string());
        }

        let prefix = options.value_of(options::PREFIX).map(|s| s.to_string());

        if let Some(base_dir_str) = options.value_of(options::BASE_DIR) {
            if !Self::is_valid_dir_path(base_dir_str) {
                return Err(USimpleError::new(
                    E_BAD_ARG,
                    format!("invalid base directory '{}'", base_dir_str),
                )
                .into());
            }
            defaults_data.set_def_home(base_dir_str.to_string());
        }

        let inactive_days = if flag.fflg {
            match options.value_of(options::INACTIVE).unwrap().parse::<i64>() {
                Ok(days) if days >= 0 => {
                    defaults_data.set_def_inactive(days);
                    Some(days)
                }
                Ok(_) => {
                    return Err(
                        USimpleError::new(E_BAD_ARG, "invalid number of inactive days").into(),
                    )
                }
                Err(_) => {
                    return Err(
                        USimpleError::new(E_BAD_ARG, "invalid number of inactive days").into(),
                    )
                }
            }
        } else {
            None
        };

        let user_gid_str = options.value_of(options::GID).map(|s| s.to_string());

        let additional_groups = options
            .value_of(options::GROUPS)
            .map(|s| s.split(',').map(String::from).collect());

        let chroot_dir = options.value_of(options::CHROOT_DIR).map(|s| s.to_string());

        Ok(Self {
            flag,
            is_shadow_pwd,
            usage,
            username,
            badname,
            defaults,
            user_gid: None,
            user_gid_str,
            uid,
            comment,
            home_dir,
            shell,
            expire_date,
            system,
            password,
            user_group,
            create_home,
            prefix,
            prefix_user_home: None,
            paths: Paths::new(),
            home_added: false,
            create_mail_spool: None,
            inactive_days,
            additional_groups,
            chroot_dir,
            settings: SystemSettings::new(),
            sub_uid_start: 0,
            sub_gid_start: 0,
            subuid_count: 0,
            subgid_count: 0,
            arg_list,
        })
    }

    fn help_message(&self) -> String {
        self.usage.clone()
    }

    fn is_valid_dir_path(base_dir: &str) -> bool {
        let dir_valid = base_dir.chars().all(|c| c != ':' && c != '\n');
        !dir_valid || base_dir.starts_with('/')
    }

    fn parse_and_set_expire_date(
        options: &clap::ArgMatches,
        is_shadow_pwd: bool,
        defaults: bool,
        defaults_data: &mut UserAddDefaults,
    ) -> UResult<Option<i64>> {
        let mut expire_date: Option<i64> = None;
        let expire_date_string = options
            .value_of(options::EXPIRE_DATE)
            .map(|s| s.to_string());
        if let Some(date_str) = &expire_date_string {
            let res = strtoday(date_str)?;
            if res < -1 {
                return Err(
                    USimpleError::new(E_BAD_ARG, format!("invalid date '{}'", date_str)).into(),
                );
            }
            expire_date = Some(res);
        }

        if expire_date.is_some() && !is_shadow_pwd {
            return Err(USimpleError::new(E_BAD_ARG, "shadow passwords required for -e").into());
        }

        if defaults {
            if let Some(exp_date_str) = expire_date_string {
                defaults_data.set_def_expire(exp_date_str);
            }
        }
        Ok(expire_date)
    }

    fn is_valid(comment: &str) -> bool {
        comment.chars().all(|c| c != ':' && c != '\n')
    }

    fn is_valid_shell(shell: &str) -> UResult<()> {
        if shell.contains(':') || shell.contains('\n') {
            return Err(USimpleError::new(E_BAD_ARG, format!("invalid shell '{}'", shell)).into());
        }

        if !(shell.starts_with('/') || shell == "*") {
            return Err(USimpleError::new(E_BAD_ARG, format!("invalid shell '{}'", shell)).into());
        }

        if !shell.is_empty() && shell != "*" && shell != "/sbin/nologin" {
            let path = Path::new(shell);
            match fs::metadata(path) {
                Ok(metadata) => {
                    if metadata.is_dir() || (metadata.permissions().mode() & 0o111 == 0) {
                        return Err(USimpleError::new(
                            E_BAD_ARG,
                            format!("Warning: missing or non-executable shell '{}'", shell),
                        )
                        .into());
                    }
                }
                Err(_) => {
                    return Err(USimpleError::new(
                        E_BAD_ARG,
                        format!("Warning: missing or non-executable shell '{}'", shell),
                    )
                    .into());
                }
            }
        }

        Ok(())
    }
}

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

    ///
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

    ///
    pub fn update_path(&mut self, prefix: Option<String>) {
        let prefix_ref = prefix.as_ref();

        self.passwd_db_file = Self::base_path(prefix_ref, DEFAULT_PASSWD_DB_FILE);
        self.group_db_file = Self::base_path(prefix_ref, DEFAULT_GROUP_DB_FILE);
        self.shadow_db_file = Self::base_path(prefix_ref, DEFAULT_SHADOW_DB_FILE);
        self.gshadow_db_file = Self::base_path(prefix_ref, DEFAULT_GSHADOW_DB_FILE);
        self.default_db_file = Self::base_path(prefix_ref, DEFAULT_USERADD_DB_FILE);
        self.login_defs_db_file = Self::base_path(prefix_ref, DEFAULT_LOGIN_DEFS_DB_FILE);

        if let Some(subuid_db) = &mut self.subuid_db_file {
            *subuid_db = Self::base_path(prefix_ref, DEFAULT_SUBUID_DB_FILE);
        }

        if let Some(subgid_db) = &mut self.subgid_db_file {
            *subgid_db = Self::base_path(prefix_ref, DEFAULT_SUBGID_DB_FILE);
        }
    }
}

/// Parse the cmd args
pub fn parse_useradd_cmd_args(
    args: impl uucore::Args,
    about: &str,
    usage: &str,
    defaults: &mut UserAddDefaults,
) -> UResult<Config> {
    let arg_list = args.collect_lossy();
    let mut command = useradd_app(about, usage);
    let mut usage_doc = Vec::new();
    command.write_help(&mut usage_doc).unwrap();
    let usage = String::from_utf8_lossy(&usage_doc).into_owned();

    let matches = command.get_matches_from(
        &arg_list
            .iter()
            .map(std::convert::AsRef::as_ref)
            .collect::<Vec<&str>>(),
    );

    let flag = Flag::new().update(&matches);
    Config::from(&matches, defaults, flag, arg_list, usage)
}

///
pub fn useradd_app<'a>(about: &'a str, usage: &'a str) -> Command<'a> {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about(about)
        .override_usage(format_usage(usage))
        .infer_long_args(true)
        .arg(
            Arg::new(options::USER_NAME)
                .help("Name of the user to add")
                .index(1),
        )
        .arg(
            Arg::new(options::BADNAME)
                .long("badname")
                .help("do not check for bad names"),
        )
        .arg(
            Arg::new(options::DEFAULTS)
                .short('D')
                .long(options::DEFAULTS)
                .help("print or change default useradd configuration"),
        )
        .arg(
            Arg::new(options::GID)
                .short('g')
                .long(options::GID)
                .value_name("GROUP")
                .help("name or ID of the primary group of the new account")
                .takes_value(true),
        )
        .arg(
            Arg::new(options::UID)
                .short('u')
                .long(options::UID)
                .value_name("UID")
                .help("user ID of the new account")
                .takes_value(true),
        )
        .arg(
            Arg::new(options::COMMENT)
                .short('c')
                .long(options::COMMENT)
                .value_name("COMMENT")
                .help("GECOS field of the new account")
                .takes_value(true),
        )
        .arg(
            Arg::new(options::HOME_DIR)
                .short('d')
                .long(options::HOME_DIR)
                .value_name("HOME_DIR")
                .help("home directory of the new account")
                .takes_value(true),
        )
        .arg(
            Arg::new(options::SHELL)
                .short('s')
                .long(options::SHELL)
                .value_name("SHELL")
                .help("login shell of the new account")
                .takes_value(true),
        )
        .arg(
            Arg::new(options::EXPIRE_DATE)
                .short('e')
                .long(options::EXPIRE_DATE)
                .value_name("EXPIRE_DATE")
                .help("expiration date of the new account")
                .takes_value(true),
        )
        .arg(
            Arg::new(options::PASSWORD)
                .short('p')
                .long(options::PASSWORD)
                .value_name("PASSWORD")
                .help("encrypted password of the new account")
                .takes_value(true),
        )
        .arg(
            Arg::new(options::SYSTEM)
                .short('r')
                .long(options::SYSTEM)
                .help("create a system account"),
        )
        .arg(
            Arg::new(options::NON_UNIQUE)
                .short('o')
                .long(options::NON_UNIQUE)
                .help("allow to create users with duplicate (non-unique) UID"),
        )
        .arg(
            Arg::new(options::KEY)
                .short('K')
                .long(options::KEY)
                .value_name("KEY=VALUE")
                .help("override /etc/login.defs defaults")
                .takes_value(true)
                .multiple_occurrences(true),
        )
        .arg(
            Arg::new(options::USER_GROUP)
                .short('U')
                .long(options::USER_GROUP)
                .help("create a group with the same name as the user"),
        )
        .arg(
            Arg::new(options::NO_USER_GROUP)
                .short('N')
                .long(options::NO_USER_GROUP)
                .help("do not create a group with the same name as the user"),
        )
        .arg(
            Arg::new(options::CREATE_HOME)
                .short('m')
                .long(options::CREATE_HOME)
                .help("create the user's home directory"),
        )
        .arg(
            Arg::new(options::NO_CREATE_HOME)
                .short('M')
                .long(options::NO_CREATE_HOME)
                .help("do not create the user's home directory"),
        )
        .arg(
            Arg::new(options::SKEL)
                .short('k')
                .long(options::SKEL)
                .value_name("SKEL_DIR")
                .help("use this alternative skeleton directory")
                .takes_value(true),
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
            Arg::new(options::BASE_DIR)
                .short('b')
                .long(options::BASE_DIR)
                .value_name("BASE_DIR")
                .help("base directory for the home directory of the new account")
                .takes_value(true),
        )
        .arg(
            Arg::new(options::INACTIVE)
                .short('f')
                .long(options::INACTIVE)
                .value_name("INACTIVE_DAYS")
                .help("password inactivity period of the new account")
                .takes_value(true),
        )
        .arg(
            Arg::new(options::GROUPS)
                .short('G')
                .long(options::GROUPS)
                .value_name("GROUPS")
                .help("list of supplementary groups of the new account")
                .takes_value(true),
        )
        .arg(
            Arg::new(options::CHROOT_DIR)
                .short('R')
                .long(options::CHROOT_DIR)
                .value_name("CHROOT_DIR")
                .help("directory to chroot into")
                .takes_value(true),
        )
        .arg(
            Arg::new(options::BTRFS_SUBVOLUME_HOME)
                .long(options::BTRFS_SUBVOLUME_HOME)
                .help("use BTRFS subvolume for home directory"),
        )
        .arg(
            Arg::new(options::ADD_SUBIDS_FOR_SYSTEM)
                .short('F')
                .long(options::ADD_SUBIDS_FOR_SYSTEM)
                .help("add entries to sub[ud]id even when adding a system user"),
        )
}

///
fn create_home(config: &mut Config) -> UResult<()> {
    let home_dir = config.prefix_user_home.as_ref().unwrap();
    let user_id = config.uid.unwrap();
    let user_gid = config.user_gid.unwrap();

    if Path::new(home_dir).exists() {
        return Ok(());
    }

    if config.flag.btrfs_flg {
        let parent_dir = Path::new(home_dir).parent().unwrap();
        let output = std::process::Command::new("findmnt")
            .arg("-nT")
            .arg(parent_dir)
            .output()?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        if !output_str.contains("btrfs") {
            return Err(USimpleError::new(
                E_BAD_ARG,
                format!(
                    "The parent directory {} is not on a Btrfs filesystem",
                    parent_dir.display()
                ),
            )
            .into());
        }

        let output = std::process::Command::new("btrfs")
            .arg("subvolume")
            .arg("create")
            .arg(home_dir)
            .output()
            .map_err(|err| {
                USimpleError::new(
                    E_HOMEDIR,
                    format!("failed to create Btrfs subvolume {}: {}", home_dir, err),
                )
            })?;

        if !output.status.success() {
            return Err(USimpleError::new(
                E_HOMEDIR,
                format!(
                    "failed to create Btrfs subvolume {}: {}",
                    home_dir,
                    String::from_utf8_lossy(&output.stderr)
                ),
            ));
        }
    } else {
        let mut path = String::new();
        let bhome = home_dir.clone();
        let parts: Vec<&str> = bhome.split('/').collect();

        for part in parts {
            if !part.is_empty() {
                path.push('/');
                path.push_str(part);

                if Path::new(&path).exists() {
                    continue;
                }

                fs::create_dir(&path).map_err(|err| {
                    USimpleError::new(
                        E_HOMEDIR,
                        format!("cannot create directory {}: {}", path, err),
                    )
                })?;

                chown(
                    path.as_str(),
                    Some(Uid::from_raw(0)),
                    Some(Gid::from_raw(0)),
                )
                .map_err(|err| {
                    USimpleError::new(E_HOMEDIR, format!("chown on `{}` failed: {}", path, err))
                })?;

                fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).map_err(|err| {
                    USimpleError::new(E_HOMEDIR, format!("chmod on `{}` failed: {}", path, err))
                })?;
            }
        }
    }

    chown(
        home_dir.as_str(),
        Some(Uid::from_raw(user_id)),
        Some(Gid::from_raw(user_gid)),
    )
    .map_err(|err| {
        USimpleError::new(
            E_HOMEDIR,
            format!("cannot set owner for home directory {}: {}", home_dir, err),
        )
    })?;

    let mode = getdef_ulong("HOME_MODE", 0o750 & !getdef_ulong("UMASK", 0o022));
    fs::set_permissions(home_dir, fs::Permissions::from_mode(mode)).map_err(|err| {
        USimpleError::new(
            E_HOMEDIR,
            format!(
                "cannot set permissions for home directory {}: {}",
                home_dir, err
            ),
        )
    })?;
    config.home_added = true;

    Ok(())
}

///
fn check_uid_range(username: &String, system: bool, user_id: u32) -> UResult<()> {
    let uid_min: uid_t;
    let uid_max: uid_t;
    if system {
        uid_max = getdef_ulong("SYS_UID_MAX", getdef_ulong("UID_MIN", 1000) - 1);
        if user_id > uid_max {
            return Err(USimpleError::new(
                E_BAD_ARG,
                format!(
                    " warning: {}'s uid {} is greater than SYS_UID_MAX {}",
                    username, user_id, uid_max
                ),
            ));
        }
    } else {
        uid_min = getdef_ulong("UID_MIN", 1000);
        uid_max = getdef_ulong("UID_MAX", 60000);
        if user_id <= uid_max {
            if user_id < uid_min || user_id > uid_max {
                return Err(USimpleError::new(
                    E_BAD_ARG,
                    format!(
                        " warning: {}'s uid {} outside of the UID_MIN {} and UID_MAX {} range.",
                        username, user_id, uid_min, uid_max
                    ),
                ));
            }
        }
    }
    return Ok(());
}

///
fn create_user(
    config: &mut Config,
    defaults_data: &mut UserAddDefaults,
    lock_status: &mut LockStatus,
) -> UResult<()> {
    let username = config.username.as_ref().unwrap();

    let uid = config.uid.unwrap_or_else(|| get_next_uid().unwrap());

    let shadow_entry = new_spent(config, username, defaults_data)?;
    if config.is_shadow_pwd {
        append_to_shadow(config, &shadow_entry, &username, lock_status)?;
    } else {
        return Err(USimpleError::new(
            E_GRP_UPDATE,
            format!(
                "failed to prepare the new {} entry '{}'",
                "/etc/shadow",
                username.clone()
            ),
        )
        .into());
    }

    if config.settings.is_sub_uid
        && !local_sub_uid_assigned(
            config.username.as_ref().unwrap(),
            config.paths.subuid_db_file.clone().unwrap(),
        )
    {
        let subuid_entry = format!(
            "{}:{}:{}",
            username, config.sub_uid_start, config.subuid_count
        );
        append_to_subuid(config, &subuid_entry, &username, lock_status)?;
    }

    if config.settings.is_sub_gid
        && !local_sub_gid_assigned(
            config.username.as_ref().unwrap(),
            config.paths.subgid_db_file.clone().unwrap(),
        )
    {
        let subgid_entry = format!(
            "{}:{}:{}",
            username, config.sub_gid_start, config.subgid_count
        );
        append_to_subgid(config, &subgid_entry, &username, lock_status)?;
    }

    if config.flag.Uflg {
        if !config.flag.gflg {
            let gid = find_new_gid(config.system, None, false, false, &config.paths)?;
            config.user_gid = Some(gid)
        }
        let primary_group_entry = format!("{}:x:{}:", username, config.user_gid.unwrap());
        append_to_group(config, &primary_group_entry, username, lock_status)?;

        let gshadow_entry = format!("{}:!::", username);
        append_to_gshadow(config, &gshadow_entry, username, lock_status)?;
    }

    let passwd_entry = new_pwent(config, username, uid, config.user_gid);
    append_to_passwd(config, &passwd_entry, &username, lock_status)?;

    Ok(())
}

///
fn new_pwent(config: &Config, username: &str, uid: u32, gid: Option<u32>) -> String {
    let mut pwent = Passwd::new();
    pwent.pw_name = username.to_string();

    if config.is_shadow_pwd {
        pwent.pw_passwd = Some("x".to_string());
    } else {
        pwent.pw_passwd = config.password.clone();
    }

    pwent.pw_uid = uid;
    pwent.pw_gid = gid.unwrap();
    pwent.pw_gecos = config.comment.clone();
    pwent.pw_dir = config.home_dir.clone();
    pwent.pw_shell = config.shell.clone();

    let passwd_entry = format!(
        "{}:{}:{}:{}:{}:{}:{}",
        pwent.pw_name,
        pwent.pw_passwd.as_ref().unwrap_or(&"".to_string()),
        pwent.pw_uid,
        pwent.pw_gid,
        pwent.pw_gecos.as_ref().unwrap_or(&"".to_string()),
        pwent
            .pw_dir
            .as_ref()
            .unwrap_or(&format!("/home/{}", pwent.pw_name)),
        pwent.pw_shell.as_ref().unwrap_or(&"/bin/bash".to_string())
    );

    passwd_entry
}

///
fn new_spent(config: &Config, username: &str, defaults_data: &UserAddDefaults) -> UResult<String> {
    let mut spent = Spwd::new();
    spent.sp_namp = username.to_string();
    if config.password.is_some() {
        spent.sp_pwdp = config.password.clone().unwrap();
    } else {
        spent.sp_pwdp = "!".to_string();
    }
    let current_time = gettime()?;
    let lstchg = current_time / DAY;
    if lstchg != 0 {
        spent.sp_lstchg = Some(lstchg);
    } else {
        spent.sp_lstchg = None;
    }

    if !config.system {
        spent.sp_min = Some(getdef_num("PASS_MIN_DAYS", -1));
        spent.sp_max = Some(getdef_num("PASS_MAX_DAYS", -1));
        spent.sp_warn = Some(getdef_num("PASS_WARN_AGE", -1));
        spent.sp_inact = config.inactive_days.or(defaults_data.def_inactive);
        spent.sp_expire = config.expire_date;
    } else {
        spent.sp_min = None;
        spent.sp_max = None;
        spent.sp_warn = None;
        spent.sp_inact = None;
        spent.sp_expire = None;
    }
    spent.sp_flag = None;

    let shadow_entry = format!(
        "{}:{}:{}:{}:{}:{}:{}:{}:{}",
        spent.sp_namp,
        spent.sp_pwdp,
        Spwd::format_optional(spent.sp_lstchg),
        Spwd::format_optional(spent.sp_min),
        Spwd::format_optional(spent.sp_max),
        Spwd::format_optional(spent.sp_warn),
        Spwd::format_optional(spent.sp_inact),
        Spwd::format_optional(spent.sp_expire),
        Spwd::format_optional(spent.sp_flag),
    );

    Ok(shadow_entry)
}

fn update_file_entry(file: &mut File, updated_entry: &str) -> Result<(), Error> {
    let mut file_content = String::new();
    file.seek(SeekFrom::Start(0))?;
    file.read_to_string(&mut file_content)?;

    let mut lines: Vec<String> = file_content.lines().map(|line| line.to_string()).collect();

    lines.push(updated_entry.to_string());

    file.seek(SeekFrom::Start(0))?;
    file.set_len(0)?;
    file.write_all(lines.join("\n").as_bytes())?;
    file.write_all(b"\n")?;
    file.sync_all()?;

    Ok(())
}

///
fn append_to_passwd(
    config: &Config,
    content: &str,
    username: &String,
    lock_status: &mut LockStatus,
) -> UResult<()> {
    // append_to_file_with_lock_and_mode(&config.paths.passwd_db_file, content, 0o644)
    if let Some(ref mut pw_file) = lock_status.pw_locked {
        if pw_update(pw_file, &content).is_err() {
            return Err(USimpleError::new(
                E_GRP_UPDATE,
                format!(
                    "failed to prepare the new {} entry '{}'",
                    config.paths.passwd_db_file, username
                ),
            ));
        }
    } else {
        return Err(USimpleError::new(
            E_GRP_UPDATE,
            "passwd file is not locked or opened.".to_string(),
        ));
    }

    Ok(())
}

///
fn append_to_shadow(
    config: &Config,
    content: &str,
    username: &String,
    lock_status: &mut LockStatus,
) -> UResult<()> {
    // append_to_file_with_lock_and_mode(&config.paths.shadow_db_file, content, 0o640)
    if let Some(ref mut spw_file) = lock_status.spw_locked {
        if spw_update(spw_file, &content).is_err() {
            return Err(USimpleError::new(
                E_GRP_UPDATE,
                format!(
                    "failed to prepare the new {} entry '{}'",
                    config.paths.shadow_db_file, username
                ),
            ));
        }
    } else {
        return Err(USimpleError::new(
            E_GRP_UPDATE,
            "shadow file is not locked or opened.".to_string(),
        ));
    }

    Ok(())
}

///
fn append_to_group(
    config: &Config,
    content: &str,
    username: &String,
    lock_status: &mut LockStatus,
) -> UResult<()> {
    if let Some(ref mut gr_file) = lock_status.gr_locked {
        if gr_update(gr_file, &content).is_err() {
            return Err(USimpleError::new(
                E_GRP_UPDATE,
                format!(
                    "failed to prepare the new {} entry '{}'",
                    config.paths.group_db_file, username
                ),
            ));
        }
    } else {
        return Err(USimpleError::new(
            E_GRP_UPDATE,
            "group file is not locked or opened.".to_string(),
        ));
    }

    Ok(())
}

///
fn append_to_gshadow(
    config: &Config,
    content: &str,
    username: &String,
    lock_status: &mut LockStatus,
) -> UResult<()> {
    if let Some(ref mut sgr_file) = lock_status.sgr_locked {
        if sgr_update(sgr_file, &content).is_err() {
            return Err(USimpleError::new(
                E_GRP_UPDATE,
                format!(
                    "failed to prepare the new {} entry '{}'",
                    config.paths.gshadow_db_file, username
                ),
            ));
        }
    } else {
        return Err(USimpleError::new(
            E_GRP_UPDATE,
            "gshadow group file is not locked or opened.".to_string(),
        ));
    }
    Ok(())
}

///
fn append_to_subuid(
    config: &Config,
    content: &str,
    username: &String,
    lock_status: &mut LockStatus,
) -> UResult<()> {
    if let Some(ref mut sub_uid_file) = lock_status.sub_uid_locked {
        if subuid_update(sub_uid_file, &content).is_err() {
            return Err(USimpleError::new(
                E_GRP_UPDATE,
                format!(
                    "failed to prepare the new {} entry '{}'",
                    config.paths.subuid_db_file.clone().unwrap(),
                    username
                ),
            ));
        }
    } else {
        return Err(USimpleError::new(
            E_GRP_UPDATE,
            "subuid file is not locked or opened.".to_string(),
        ));
    }
    Ok(())
}

///
fn append_to_subgid(
    config: &Config,
    content: &str,
    username: &String,
    lock_status: &mut LockStatus,
) -> UResult<()> {
    if let Some(ref mut sub_gid_file) = lock_status.sub_gid_locked {
        if subgid_update(sub_gid_file, &content).is_err() {
            return Err(USimpleError::new(
                E_GRP_UPDATE,
                format!(
                    "failed to prepare the new {} entry '{}'",
                    config.paths.subgid_db_file.clone().unwrap(),
                    username
                ),
            ));
        }
    } else {
        return Err(USimpleError::new(
            E_GRP_UPDATE,
            "subgid file is not locked or opened.".to_string(),
        ));
    }
    Ok(())
}

///
fn pw_update(pw_file: &mut File, updated_entry: &str) -> Result<(), Error> {
    update_file_entry(pw_file, &updated_entry)
}

///
fn spw_update(spw_file: &mut File, updated_entry: &str) -> Result<(), Error> {
    update_file_entry(spw_file, &updated_entry)
}

///
fn gr_update(gr_file: &mut File, updated_entry: &str) -> Result<(), Error> {
    update_file_entry(gr_file, &updated_entry)
}

///
fn sgr_update(sgr_file: &mut File, updated_entry: &str) -> Result<(), Error> {
    update_file_entry(sgr_file, &updated_entry)
}

///
fn subuid_update(subuid_file: &mut File, updated_entry: &str) -> Result<(), Error> {
    update_file_entry(subuid_file, &updated_entry)
}

///
fn subgid_update(subgid_file: &mut File, updated_entry: &str) -> Result<(), Error> {
    update_file_entry(subgid_file, &updated_entry)
}

///
fn get_next_uid() -> UResult<u32> {
    let min_uid = 1000;
    let max_uid = 60000;
    for uid in min_uid..=max_uid {
        if User::from_uid(Uid::from_raw(uid)).unwrap().is_none() {
            return Ok(uid);
        }
    }
    Err(USimpleError::new(E_UID_IN_USE, "No available UID found").into())
}

///
fn spw_file_present() -> bool {
    if getdef_bool("FORCE_SHADOW") {
        return true;
    }
    commonio_present("/etc/shadow")
}

///
fn commonio_present(file_path: &str) -> bool {
    Path::new(file_path).exists()
}

///
fn gettime() -> UResult<i64> {
    let mut shadow_logfd = io::stderr();

    let fallback = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| USimpleError::new(E_BAD_ARG, "Time went backwards"))?
        .as_secs() as i64;

    if let Ok(source_date_epoch) = env::var("SOURCE_DATE_EPOCH") {
        match source_date_epoch.parse::<i64>() {
            Ok(epoch) => {
                if epoch > fallback {
                    writeln!(shadow_logfd,
                             "Environment variable $SOURCE_DATE_EPOCH: value must be smaller than or equal to the current time ({}) but was found to be: {}",
                             fallback, epoch).unwrap();
                } else {
                    return Ok(epoch);
                }
            }
            Err(e) => {
                writeln!(
                    shadow_logfd,
                    "Environment variable $SOURCE_DATE_EPOCH: failed to parse: {}",
                    e
                )
                .unwrap();
            }
        }
    }

    Ok(fallback)
}

///
fn create_mail(config: &Config) -> UResult<()> {
    if let Some(create_mail_spool) = &config.create_mail_spool {
        if create_mail_spool != "yes" {
            return Ok(());
        }
    }
    let mut spool = getdef_str("MAIL_DIR");
    if spool.is_none() && getdef_str("MAIL_FILE").is_none() {
        spool = Some("/var/mail".to_string())
    }
    if spool.is_none() {
        return Ok(());
    }

    let spool_path = Path::new(spool.as_ref().unwrap());
    let file_path: PathBuf = if let Some(prefix) = &config.prefix {
        if spool_path.is_absolute() {
            Path::new(prefix)
                .join(spool_path.strip_prefix("/").unwrap())
                .join(config.username.as_ref().unwrap())
        } else {
            Path::new(prefix)
                .join(spool_path)
                .join(config.username.as_ref().unwrap())
        }
    } else {
        spool_path.join(config.username.as_ref().unwrap())
    };

    let fd = match open(
        file_path.to_str().unwrap(),
        OFlag::O_CREAT | OFlag::O_WRONLY | OFlag::O_TRUNC | OFlag::O_EXCL,
        Mode::empty(),
    ) {
        Ok(fd) => fd,
        Err(e) => {
            eprintln!("Error creating mailbox file: {}", e);
            return Ok(());
        }
    };

    let group = prefix_getgrnam(
        "mail",
        config.prefix.clone(),
        config.paths.group_db_file.clone(),
    );
    let (gid, mode) = match group {
        Some(group) => (
            Gid::from_raw(group.gid.as_raw()),
            Mode::S_IRUSR | Mode::S_IWUSR | Mode::S_IRGRP | Mode::S_IWGRP,
        ),
        None => {
            eprintln!("Group 'mail' not found. Creating the user mailbox file with 0600 mode.");
            (
                Gid::from_raw(config.user_gid.unwrap()),
                Mode::S_IRUSR | Mode::S_IWUSR,
            )
        }
    };

    if fchown(fd, Some(Uid::from_raw(config.uid.unwrap())), Some(gid)).is_err()
        || fchmod(fd, mode).is_err()
    {
        eprintln!("Error setting mailbox file permissions");
    }

    if fsync(fd).is_err() {
        eprintln!("Error synchronizing mailbox file");
    }

    if close(fd).is_err() {
        eprintln!("Error closing mailbox file");
    }

    Ok(())
}

///
#[allow(non_snake_case)]
fn handle_G_option(config: &mut Config) -> UResult<()> {
    if let Some(groups_str) = config.additional_groups.as_ref() {
        let mut group_map = HashMap::new();
        for group_name in groups_str {
            if let Some(group) = prefix_getgrnam(
                group_name,
                config.prefix.clone(),
                config.paths.group_db_file.clone(),
            ) {
                group_map.insert(group_name, group);
            } else {
                return Err(USimpleError::new(
                    E_NOTFOUND,
                    format!("Group '{}' does not exist", group_name),
                )
                .into());
            }
        }

        for (_, mut group) in group_map {
            if group.mem.contains(&config.username.as_ref().unwrap()) {
                continue;
            }
            group
                .mem
                .push(config.username.as_ref().unwrap().to_string());
            update_group_file(&group, config.paths.group_db_file.clone())?;
        }
    }
    Ok(())
}

///
pub fn handle_g_option(config: &mut Config, defaults_data: &mut UserAddDefaults) -> UResult<()> {
    if let Some(grname) = &config.user_gid_str {
        let grp = prefix_getgr_nam_gid(
            grname.as_str(),
            config.prefix.clone(),
            config.paths.group_db_file.clone(),
        );
        if grp.is_none() {
            return Err(USimpleError::new(
                E_NOTFOUND,
                format!("group '{}' does not exist", grname),
            )
            .into());
        }
        if config.flag.Dflg {
            defaults_data.set_def_group(grp.unwrap().gid.as_raw() as gid_t);
            defaults_data.set_def_gname(grname.to_string());
        } else {
            config.user_gid = Some(grp.unwrap().gid.as_raw() as gid_t);
        }
    }
    Ok(())
}

///
fn update_group_file(group: &Group, group_file_path: String) -> UResult<()> {
    let group_file = File::open(&group_file_path)?;
    let reader = BufReader::new(group_file);

    let temp_file_path = format!("{}.tmp", group_file_path);
    let temp_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&temp_file_path)?;
    let mut writer = BufWriter::new(temp_file);

    for line in reader.lines() {
        let line = line?;
        let mut parts: Vec<&str> = line.split(':').collect();
        if parts.len() > 2 && parts[0] == group.name {
            let new_members = group.mem.join(",");
            parts[3] = &new_members;
            let new_line = format!("{}:{}:{}:{}", parts[0], parts[1], parts[2], new_members);
            writeln!(writer, "{}", new_line)?;
        } else {
            writeln!(writer, "{}", line)?;
        }
    }
    writer.flush()?;

    std::fs::rename(temp_file_path, group_file_path)?;
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
    flock(gr_fd, FlockArg::LockExclusive)
        .map_err(|_| USimpleError::new(E_GRP_UPDATE, "Cannot lock group file; try again later."))?;
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

    if config.settings.is_sub_uid
        && !local_sub_uid_assigned(
            config.username.as_ref().unwrap(),
            config.paths.subuid_db_file.clone().unwrap(),
        )
    {
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

    if config.settings.is_sub_gid
        && !local_sub_gid_assigned(
            config.username.as_ref().unwrap(),
            config.paths.subgid_db_file.clone().unwrap(),
        )
    {
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

/// handle input
pub fn handle_input(config: &mut Config, defaults_data: &mut UserAddDefaults) -> UResult<()> {
    process_root_flag("-R", config.arg_list.clone())?;

    if config.prefix.is_some() {
        config.prefix = process_prefix_flag(config)?;
        config.paths.update_path(config.prefix.clone());
    }

    config.settings.is_shadow_pwd = SystemSettings::spw_file_present(&config.paths.shadow_db_file);
    config.settings.is_shadow_grp = SystemSettings::sgr_file_present(&config.paths.gshadow_db_file);

    if let Some(name) = &config.username {
        let pwd = prefix_getpwnam(
            name,
            config.prefix.clone(),
            config.paths.passwd_db_file.clone(),
        );
        if pwd.is_some() {
            return Err(USimpleError::new(
                E_NAME_IN_USE,
                format!("user '{}' already exists", name),
            )
            .into());
        }
    }

    if config.flag.Gflg {
        handle_G_option(config)?;
    }

    if config.flag.gflg {
        handle_g_option(config, defaults_data)?;
    }

    if !config.flag.gflg && !config.flag.Nflg && !config.flag.Uflg {
        config.flag.Uflg = getdef_bool("USERGROUPS_ENAB");
    }

    if config.flag.oflg && config.uid.is_none() {
        return Err(USimpleError::new(
            E_USAGE,
            format!(
                "{} flag is only allowed with the {} flag\n{}",
                "-o",
                "-u",
                config.help_message()
            ),
        )
        .into());
    }

    if config.flag.kflg && !config.flag.mflg {
        return Err(USimpleError::new(
            E_USAGE,
            format!(
                "{} flag is only allowed with the {} flag\n{}",
                "-k",
                "-m",
                config.help_message()
            ),
        )
        .into());
    }

    if config.flag.Uflg && config.flag.gflg {
        return Err(USimpleError::new(
            E_USAGE,
            format!(
                "options {} and {} conflict\n{}",
                "-U",
                "-g",
                config.help_message()
            ),
        )
        .into());
    }

    if config.flag.Uflg && config.flag.Nflg {
        return Err(USimpleError::new(
            E_USAGE,
            format!(
                "options {} and {} conflict\n{}",
                "-U",
                "-N",
                config.help_message()
            ),
        )
        .into());
    }

    if config.flag.mflg && config.flag.Mflg {
        return Err(USimpleError::new(
            E_USAGE,
            format!(
                "options {} and {} conflict\n{}",
                "-m",
                "-M",
                config.help_message()
            ),
        )
        .into());
    }

    if config.flag.Dflg {
        if config.username.is_some() {
            return Err(USimpleError::new(E_USAGE, config.help_message()));
        }

        if config.flag.uflg
            || config.flag.dflg
            || config.flag.cflg
            || config.flag.mflg
            || config.flag.Gflg
        {
            return Err(USimpleError::new(E_USAGE, config.help_message()));
        }
    } else {
        if config.username.is_none() {
            return Err(USimpleError::new(E_USAGE, config.usage.clone()));
        }

        if let Some(name) = &config.username {
            if !is_valid_user_name(name, config.badname) {
                return Err(USimpleError::new(
                    E_USAGE,
                    format!("invalid user name '{}': use --badname to ignore", name),
                ));
            }
        }

        if !config.flag.dflg {
            config.home_dir = Some(format!(
                "{}/{}",
                defaults_data.def_home.clone().unwrap(),
                config.username.as_ref().unwrap()
            ));
        }

        if let Some(prefix) = &config.prefix {
            config.prefix_user_home =
                Some(format!("{}{}", prefix, config.home_dir.clone().unwrap()))
        } else {
            config.prefix_user_home = config.home_dir.clone();
        }
    }

    if !config.flag.gflg {
        config.user_gid = defaults_data.def_group();
    }

    if !config.flag.sflg {
        config.shell = defaults_data.def_shell();
    }

    config.create_mail_spool = defaults_data.def_create_mail_spool();

    if !config.flag.rflg {
        if getdef_bool("CREATE_HOME") {
            config.flag.mflg = true;
        }
    }

    if config.flag.Mflg {
        config.flag.mflg = false;
    }

    useradd_main(config, defaults_data)?;

    Ok(())
}

///
pub fn useradd_main(config: &mut Config, defaults_data: &mut UserAddDefaults) -> UResult<()> {
    let uid_min = getdef_ulong("UID_MIN", 1000);
    let uid_max = getdef_ulong("UID_MAX", 60000);
    config.subuid_count = getdef_ulong("SUB_UID_COUNT", 65536);
    config.subgid_count = getdef_ulong("SUB_GID_COUNT", 65536);

    let is_sub_uid = config.subuid_count > 0
        && SystemSettings::sub_uid_file_present(DEFAULT_SUBUID_DB_FILE)
        && (!config.flag.rflg || config.flag.Fflg)
        && (config
            .uid
            .map_or(true, |uid| uid <= uid_max && uid >= uid_min));

    let is_sub_gid = config.subgid_count > 0
        && SystemSettings::sub_gid_file_present(DEFAULT_SUBGID_DB_FILE)
        && (!config.flag.rflg || config.flag.Fflg)
        && (config
            .uid
            .map_or(true, |uid| uid <= uid_max && uid >= uid_min));

    config.settings.is_sub_uid = is_sub_uid;
    config.settings.is_sub_gid = is_sub_gid;

    if config.flag.Dflg {
        if config.flag.gflg
            || config.flag.bflg
            || config.flag.fflg
            || config.flag.eflg
            || config.flag.sflg
        {
            if let Err(_) = defaults_data.set_defaults(DEFAULT_USERADD_DB_FILE) {
                return Err(USimpleError::new(1, "failed to set defaults").into());
            }
            return Ok(());
        }
        defaults_data.show_defaults();
        return Ok(());
    }

    if config.flag.Uflg {
        if Group::from_name(config.username.as_ref().unwrap())
            .unwrap()
            .is_some()
        {
            return Err(USimpleError::new(
                E_NAME_IN_USE,
                format!(
                    "group {} exists - if you want to add this user to that group, use -g.",
                    config.username.as_ref().unwrap()
                ),
            )
            .into());
        }
    }

    if config.settings.is_sub_uid && config.subuid_count != 0 {
        if find_new_sub_uids(&mut config.sub_uid_start, &mut config.subuid_count) < 0 {
            return Err(
                USimpleError::new(E_SUB_UID_UPDATE, "can't create subordinate user IDs").into(),
            );
        }
    }
    if config.settings.is_sub_gid && config.subgid_count != 0 {
        if find_new_sub_gids(&mut config.sub_gid_start, &mut config.subgid_count) < 0 {
            return Err(
                USimpleError::new(E_SUB_GID_UPDATE, "can't create subordinate group IDs").into(),
            );
        }
    }

    let mut lock_status = open_files(config)?;

    if !config.flag.oflg {
        if config.uid.is_none() {
            let uid = find_new_uid(None, config.prefix.clone(), config.system, None, false)?;
            config.uid = Some(uid);
        } else {
            let uid = config.uid.unwrap();
            if User::from_uid(Uid::from_raw(uid)).unwrap().is_some() {
                return Err(
                    USimpleError::new(E_UID_IN_USE, format!("UID {} already exists", uid)).into(),
                );
            }
        }
    }

    if let Some(uid) = config.uid {
        if let Some(username) = &config.username {
            check_uid_range(username, config.system, uid)?;
        }
    }

    if config.flag.gflg && config.flag.Uflg {
        config.user_gid = config.uid;
    }

    create_user(config, defaults_data, &mut lock_status)?;

    if config.flag.mflg {
        create_home(config)?;
        if config.home_added {
            let def_template = defaults_data.def_template().unwrap();
            let prefix_user_home = config.prefix_user_home.as_deref().unwrap();

            let _res = copy_tree(
                def_template,
                prefix_user_home,
                false,
                true,
                None,
                Some(config.uid.unwrap()),
                None,
                Some(config.user_gid.unwrap()),
            )?;

            let _res = copy_tree(
                defaults_data.def_usrtemplate().unwrap(),
                prefix_user_home,
                false,
                true,
                None,
                Some(config.uid.unwrap()),
                None,
                Some(config.user_gid.unwrap()),
            );
        } else {
            eprintln!(
                "{}: warning: the home directory {} already exists.\n{}: Not copying any file from skel directory into it.",
                "useradd",
                config.prefix_user_home.as_deref().unwrap(),
                "useradd"
            );
        }
    }

    if !config.system {
        create_mail(config)?;
    }

    Ok(())
}

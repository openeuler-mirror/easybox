//! This file is part of the easybox package.
//
// (c) Wentong Yang <ywt0821@163.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use crate::utils::constants::{
    DEFAULT_GROUP_DB_FILE, DEFAULT_GSHADOW_DB_FILE, DEFAULT_LOGIN_DEFS_DB_FILE,
};
use crate::utils::getdef::{getdef_bool, getdef_str, putdef_str};
use crate::utils::prefix_flag::process_prefix_flag;
use crate::utils::root_flag::process_root_flag;
use clap::{crate_version, Arg, Command};
use nix::fcntl::{flock, FlockArg};
use nix::libc::gid_t;
use nix::unistd::User;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Error, Read, Seek, SeekFrom, Write};
use std::num::ParseIntError;
use std::os::unix::io::AsRawFd;
use std::path::Path;
use uucore::error::{UResult, USimpleError};
use uucore::format_usage;

/// invalid command syntax
const E_USAGE: i32 = 2;
/// invalid argument to option
const E_BAD_ARG: i32 = 3;
/// gid not unique (when -o not used)
const E_GID_IN_USE: i32 = 4;
/// group name not unique
const E_NAME_IN_USE: i32 = 9;
/// can't update group file
const E_GRP_UPDATE: i32 = 10;

/// Config.
pub struct Config {
    ///
    pub force: bool,
    ///
    pub group_name: String,
    ///
    pub gid: Option<gid_t>,
    ///
    pub overrides: HashMap<String, String>,
    ///
    pub non_unique: bool,
    ///
    pub password: Option<String>,
    ///
    pub system: bool,
    ///
    pub chroot_dir: Option<String>,
    ///
    pub prefix_dir: Option<String>,
    ///
    pub users: Option<Vec<String>>,
    ///
    pub paths: Paths,
    ///
    pub settings: SystemSettings,
    ///
    pub arg_list: Vec<String>,
    ///
    pub usage: String,
}

/// options
pub mod options {
    ///
    pub const FORCE: &str = "force";
    ///
    pub const GID: &str = "gid";
    ///
    pub const KEY: &str = "key";
    ///
    pub const NON_UNIQUE: &str = "non-unique";
    ///
    pub const PASSWORD: &str = "password";
    ///
    pub const SYSTEM: &str = "system";
    ///
    pub const CHROOT_DIR: &str = "root";
    ///
    pub const PREFIX_DIR: &str = "prefix";
    ///
    pub const USERS: &str = "users";
}

/// Paths structure to manage file paths
#[derive(Default, Debug)]
pub struct Paths {
    ///
    pub group_db_file: String,
    ///
    pub gshadow_db_file: String,
    ///
    pub login_defs_db_file: String,
}

///
pub struct LockStatus {
    ///
    gr_locked: Option<File>,
    ///
    sgr_locked: Option<File>,
}

///
pub struct SystemSettings {
    ///
    pub is_shadow_grp: bool,
}

/// Config
impl Config {
    ///
    pub fn from(options: &clap::ArgMatches, arg_list: Vec<String>, usage: String) -> Self {
        let force = options.is_present(options::FORCE);
        let group_name = options.value_of("group").unwrap().to_string();
        let gid = options
            .value_of(options::GID)
            .map(|v| v.parse::<gid_t>().unwrap());
        let mut overrides = HashMap::new();
        if let Some(values) = options.values_of(options::KEY) {
            for value in values {
                let parts: Vec<&str> = value.split('=').collect();
                if parts.len() == 2 {
                    overrides.insert(parts[0].to_string(), parts[1].to_string());
                }
            }
        }
        let non_unique = options.is_present(options::NON_UNIQUE);
        let password = options.value_of(options::PASSWORD).map(|v| v.to_string());
        let system = options.is_present(options::SYSTEM);
        let chroot_dir = options.value_of(options::CHROOT_DIR).map(|v| v.to_string());
        let prefix_dir = options.value_of(options::PREFIX_DIR).map(|v| v.to_string());
        let users = options
            .value_of(options::USERS)
            .map(|v| v.split(',').map(|s| s.to_string()).collect());
        Self {
            force,
            group_name,
            gid,
            overrides,
            non_unique,
            password,
            system,
            chroot_dir,
            prefix_dir,
            users,
            paths: Paths::new(),
            settings: SystemSettings::new(),
            arg_list,
            usage,
        }
    }
}

///
impl Paths {
    /// Create a new Paths structure
    pub fn new() -> Self {
        Self {
            group_db_file: DEFAULT_GROUP_DB_FILE.to_string(),
            gshadow_db_file: DEFAULT_GSHADOW_DB_FILE.to_string(),
            login_defs_db_file: DEFAULT_LOGIN_DEFS_DB_FILE.to_string(),
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

        self.group_db_file = Self::base_path(prefix_ref, DEFAULT_GROUP_DB_FILE);
        self.gshadow_db_file = Self::base_path(prefix_ref, DEFAULT_GSHADOW_DB_FILE);
        self.login_defs_db_file = Self::base_path(prefix_ref, DEFAULT_LOGIN_DEFS_DB_FILE);
    }
}

///
impl LockStatus {
    fn new() -> Self {
        LockStatus {
            gr_locked: None,
            sgr_locked: None,
        }
    }
}

impl SystemSettings {
    /// Create a new SystemSettings
    pub fn new() -> Self {
        Self {
            is_shadow_grp: false,
        }
    }

    ///
    pub fn sgr_file_present(gshadow_db_file_path: &str) -> bool {
        if getdef_bool("FORCE_SHADOW") {
            return true;
        }

        Self::commonio_present(gshadow_db_file_path)
    }
    ///
    pub fn commonio_present(file_path: &str) -> bool {
        Path::new(file_path).exists()
    }
}

/// Parse the cmd args
pub fn parse_groupadd_cmd_args(
    args: impl uucore::Args,
    about: &str,
    usage: &str,
) -> UResult<Config> {
    let arg_list = args.collect_lossy();
    let mut command = groupadd_app(about, usage);
    let mut usage_doc = Vec::new();
    command.write_help(&mut usage_doc).unwrap();

    let usage = String::from_utf8_lossy(&usage_doc).into_owned();

    let matches = command.get_matches_from(
        &arg_list
            .iter()
            .map(std::convert::AsRef::as_ref)
            .collect::<Vec<&str>>(),
    );
    // Ok(Config::from(&matches))
    Ok(Config::from(&matches, arg_list, usage))
}

///
pub fn groupadd_app<'a>(about: &'a str, usage: &'a str) -> Command<'a> {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about(about)
        .override_usage(format_usage(usage))
        .infer_long_args(true)
        .arg(
            Arg::new(options::FORCE)
                .short('f')
                .long(options::FORCE)
                .help("exit successfully if the group already exists, and cancel -g if the GID is already used"),
        )
        .arg(
            Arg::new(options::GID)
                .short('g')
                .long(options::GID)
                .value_name("GID")
                .help("use GID for the new group")
                .takes_value(true),
        )
        .arg(
            Arg::new(options::KEY)
                .short('K')
                .long(options::KEY)
                .value_name("KEY=VALUE")
                .help("override /etc/login.defs defaults")
                .multiple_occurrences(true)
                .takes_value(true),
        )
        .arg(
            Arg::new("group")
                .help("The name of the group to add")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("help")
                .short('h')
                .long("help")
                .help("Prints help information")
                .takes_value(false)
                .global(true),
        )
        .arg(
            Arg::new("version")
                .short('v')
                .long("version")
                .help("Prints version information")
                .takes_value(false)
                .global(true),
        )
        .arg(
            Arg::new(options::NON_UNIQUE)
                .short('o')
                .long(options::NON_UNIQUE)
                .help("allow to create groups with duplicate (non-unique) GID")
                .takes_value(false),
        )
        .arg(
            Arg::new(options::PASSWORD)
                .short('p')
                .long(options::PASSWORD)
                .value_name("PASSWORD")
                .help("use this encrypted password for the new group")
                .takes_value(true),
        )
        .arg(
            Arg::new(options::SYSTEM)
                .short('r')
                .long(options::SYSTEM)
                .help("create a system account"),
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
            Arg::new(options::PREFIX_DIR)
                .short('P')
                .long(options::PREFIX_DIR)
                .value_name("PREFIX_DIR")
                .help("directory prefix")
                .takes_value(true),
        )
        .arg(
            Arg::new(options::USERS)
                .short('U')
                .long(options::USERS)
                .value_name("USERS")
                .help("list of user members of this group")
                .takes_value(true),
        )
}

///
fn get_ranges(sys_group_flag: bool) -> UResult<(gid_t, gid_t)> {
    let mut gid_min = 1000;
    let mut gid_max = 60000;

    // Try getting the value from the config
    if let Some(gid_min_str) = getdef_str("GID_MIN") {
        gid_min = gid_min_str
            .parse()
            .map_err(|e: ParseIntError| USimpleError::new(E_BAD_ARG, e.to_string()))?;
    }
    if let Some(gid_max_str) = getdef_str("GID_MAX") {
        gid_max = gid_max_str
            .parse()
            .map_err(|e: ParseIntError| USimpleError::new(E_BAD_ARG, e.to_string()))?;
    }
    if sys_group_flag {
        if let Some(sys_gid_min_str) = getdef_str("SYS_GID_MIN") {
            gid_min = sys_gid_min_str
                .parse()
                .map_err(|e: ParseIntError| USimpleError::new(E_BAD_ARG, e.to_string()))?;
        } else {
            gid_min = 1;
        }
        if let Some(sys_gid_max_str) = getdef_str("SYS_GID_MAX") {
            gid_max = sys_gid_max_str
                .parse()
                .map_err(|e: ParseIntError| USimpleError::new(E_BAD_ARG, e.to_string()))?;
        } else {
            gid_max = 999;
        }
    }

    Ok((gid_min, gid_max))
}

///
fn check_gid(
    gid: gid_t,
    min: gid_t,
    max: gid_t,
    used_gids: &HashSet<gid_t>,
    non_unique: bool,
) -> Result<(), i32> {
    if gid < min || gid > max {
        return Err(nix::errno::Errno::ERANGE as i32);
    }
    if !non_unique && used_gids.contains(&gid) {
        return Err(nix::errno::Errno::EEXIST as i32);
    }
    Ok(())
}

fn get_used_gids_and_group_names(
    file_path: String,
    sys_group_flag: bool,
) -> UResult<(HashSet<gid_t>, HashSet<String>)> {
    let (gid_min, gid_max) = get_ranges(sys_group_flag)?;

    let mut used_gids = HashSet::new();
    let mut group_names = HashSet::new();

    let group_file_path = Path::new(&file_path).to_path_buf();

    // Check if the path exists
    if !group_file_path.exists() {
        return Err(USimpleError::new(
            E_GRP_UPDATE,
            format!("Group file path {:?} does not exist", group_file_path),
        )
        .into());
    }

    match File::open(&group_file_path) {
        Ok(group_file) => {
            let reader = BufReader::new(group_file);

            for line in reader.lines() {
                let line = line?;
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() > 2 {
                    if let Ok(gid) = parts[2].parse::<gid_t>() {
                        if gid >= gid_min && gid <= gid_max {
                            used_gids.insert(gid);
                            if let Some(name) = parts.get(0) {
                                group_names.insert(name.to_string());
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            return Err(USimpleError::new(
                E_GRP_UPDATE,
                format!("Error opening group file {:?}: {}", group_file_path, e),
            )
            .into());
        }
    }

    Ok((used_gids, group_names))
}

///
fn find_next_available_gid(
    gid_min: gid_t,
    gid_max: gid_t,
    used_gids: &HashSet<gid_t>,
) -> UResult<gid_t> {
    // Get the highest GID and try to increase it by 1
    if let Some(&max_gid) = used_gids.iter().max() {
        let next_gid = max_gid + 1;
        if next_gid <= gid_max && !used_gids.contains(&next_gid) {
            return Ok(next_gid);
        }
    }

    // Find the first unused GID from gid_min to gid_max
    for gid in gid_min..=gid_max {
        if !used_gids.contains(&gid) {
            return Ok(gid);
        }
    }

    Err(USimpleError::new(E_USAGE, "No unused GIDs available").into())
}

///
fn find_next_available_system_gid(
    gid_min: gid_t,
    gid_max: gid_t,
    used_gids: &HashSet<gid_t>,
) -> UResult<gid_t> {
    // Find the first unused GID from gid_max to gid_min
    for gid in (gid_min..=gid_max).rev() {
        if !used_gids.contains(&gid) {
            return Ok(gid);
        }
    }

    Err(USimpleError::new(E_USAGE, "No unused system GIDs available").into())
}

///
fn find_new_gid(
    sys_group_flag: bool,
    preferred_gid: Option<gid_t>,
    force: bool,
    non_unique: bool,
    path: &Paths,
) -> UResult<gid_t> {
    let (gid_min, gid_max) = get_ranges(sys_group_flag)?;
    let (used_gids, _) = get_used_gids_and_group_names(path.group_db_file.clone(), sys_group_flag)?;

    if let Some(preferred_gid) = preferred_gid {
        match check_gid(preferred_gid, gid_min, gid_max, &used_gids, non_unique) {
            Ok(()) => return Ok(preferred_gid),
            Err(_) => {
                if !force {
                    return Err(
                        USimpleError::new(E_GID_IN_USE, "Preferred GID is already in use").into(),
                    );
                }
            }
        }
    }

    if used_gids.is_empty() {
        return Ok(gid_min);
    }

    if !sys_group_flag {
        find_next_available_gid(gid_min, gid_max, &used_gids)
    } else {
        find_next_available_system_gid(gid_min, gid_max, &used_gids)
    }
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
fn gr_update(gr_file: &mut File, updated_entry: &str) -> Result<(), Error> {
    update_file_entry(gr_file, &updated_entry)
}

///
fn sgr_update(gr_file: &mut File, updated_entry: &str) -> Result<(), Error> {
    update_file_entry(gr_file, &updated_entry)
}

///
fn update_group(
    lock_status: &mut LockStatus,
    file_path: String,
    group_name: &str,
    gid: gid_t,
    users: Option<&Vec<String>>,
) -> UResult<()> {
    let user_list = users.map_or(String::new(), |users| users.join(","));
    let new_group_entry = format!("{}:x:{}:{}", group_name, gid, user_list);

    if let Some(ref mut gr_file) = lock_status.gr_locked {
        if gr_update(gr_file, &new_group_entry).is_err() {
            return Err(USimpleError::new(
                E_GRP_UPDATE,
                format!(
                    "failed to prepare the new {} entry '{}'",
                    file_path, group_name
                ),
            ));
        }
    } else {
        return Err(USimpleError::new(
            E_GRP_UPDATE,
            "Group file is not locked or opened.".to_string(),
        ));
    }

    Ok(())
}

///
fn update_gshadow(
    lock_status: &mut LockStatus,
    file_path: String,
    group_name: &str,
    password: Option<&String>,
) -> UResult<()> {
    let password_str = password.map_or("!".to_string(), |p| p.clone());
    let new_gshadow_entry = format!("{}:{}::", group_name, password_str);

    if let Some(ref mut sgr_file) = lock_status.sgr_locked {
        if sgr_update(sgr_file, &new_gshadow_entry).is_err() {
            return Err(USimpleError::new(
                E_GRP_UPDATE,
                format!(
                    "failed to prepare the new {} entry '{}'",
                    file_path, group_name
                ),
            ));
        }
    } else {
        return Err(USimpleError::new(
            E_GRP_UPDATE,
            "Shadow group file is not locked or opened.".to_string(),
        ));
    }
    Ok(())
}

///
fn open_files(config: &mut Config) -> UResult<LockStatus> {
    let mut lock_status = LockStatus::new();
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
    Ok(lock_status)
}

/// handle input
pub fn handle_input(config: &mut Config) -> UResult<()> {
    if !config.overrides.is_empty() {
        for (key, value) in &config.overrides {
            putdef_str(key, value)?;
        }
    }

    process_root_flag("-R", config.arg_list.clone())?;
    config.settings.is_shadow_grp = SystemSettings::sgr_file_present(&config.paths.gshadow_db_file);

    if config.prefix_dir.is_some() {
        config.prefix_dir = process_prefix_flag(config)?;
        config.paths.update_path(config.prefix_dir.clone());
    }

    let sys_group_flag = config.system;

    let (used_gids, group_names) =
        get_used_gids_and_group_names(config.paths.group_db_file.clone(), sys_group_flag).unwrap();

    if group_names.contains(&config.group_name) {
        if config.force {
            // When the -f attribute is specified, if there is a duplicate name, it will not be added and success will be returned directly.
            return Ok(());
        } else {
            return Err(USimpleError::new(
                E_GRP_UPDATE,
                format!("group '{}' already exists", config.group_name),
            )
            .into());
        }
    }

    if let Some(gid) = config.gid {
        if used_gids.contains(&gid) && !config.non_unique {
            if config.force {
                config.gid = None; // Cancel the specified GID
            } else {
                return Err(USimpleError::new(
                    E_GID_IN_USE,
                    format!("groupadd: GID '{}' already exists", gid),
                )
                .into());
            }
        }
    }

    // Verify that the user in the user list exists
    if let Some(users) = &config.users {
        for user in users {
            match User::from_name(user) {
                Ok(Some(_)) => {}
                Ok(None) => {
                    return Err(USimpleError::new(
                        E_NAME_IN_USE,
                        format!("Invalid member username {}", user),
                    )
                    .into());
                }
                Err(err) => {
                    return Err(USimpleError::new(
                        E_NAME_IN_USE,
                        format!("Error checking user {}: {}", user, err),
                    )
                    .into());
                }
            }
        }
    }

    let mut lock_status = open_files(config)?;

    match find_new_gid(
        sys_group_flag,
        config.gid,
        config.force,
        config.non_unique,
        &config.paths,
    ) {
        Ok(gid) => {
            if let Err(err) = update_group(
                &mut lock_status,
                config.paths.group_db_file.clone(),
                &config.group_name,
                gid,
                config.users.as_ref(),
            ) {
                return Err(USimpleError::new(
                    E_GRP_UPDATE,
                    format!("Failed to update group: {}", err),
                )
                .into());
            }
            if config.settings.is_shadow_grp {
                if let Err(err) = update_gshadow(
                    &mut lock_status,
                    config.paths.gshadow_db_file.clone(),
                    &config.group_name,
                    config.password.as_ref(),
                ) {
                    return Err(USimpleError::new(
                        E_GRP_UPDATE,
                        format!("Failed to update gshadow group: {}", err),
                    )
                    .into());
                }
            }
        }
        Err(err) => println!("Error: {}", err),
    }

    Ok(())
}

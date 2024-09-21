//! This file is part of the easybox package.
//
// (c) Wentong Yang <ywt0821@163.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.
use std::fs::{rename, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};

use linked_hash_map::LinkedHashMap;
use uucore::error::{FromIo, UResult};

///
#[derive(Debug, Default)]
pub struct UserAddDefaults {
    ///
    pub def_group: Option<u32>,
    ///
    pub def_groups: Option<String>,
    ///
    pub def_gname: Option<String>,
    ///
    pub def_home: Option<String>,
    ///
    pub def_shell: Option<String>,
    ///
    pub def_template: Option<String>,
    ///
    pub def_usrtemplate: Option<String>,
    ///
    pub def_create_mail_spool: Option<String>,
    ///
    pub def_log_init: Option<String>,
    ///
    pub def_inactive: Option<i64>,
    ///
    pub def_expire: Option<String>,
}

///
impl UserAddDefaults {
    ///
    pub fn new_defaults() -> Self {
        UserAddDefaults {
            def_group: Some(1000),
            def_groups: None,
            def_gname: Some("other".to_string()),
            def_home: Some("/home".to_string()),
            def_shell: Some("/bin/bash".to_string()),
            def_template: Some("/etc/skel".to_string()),
            def_usrtemplate: Some("/usr/etc/skel".to_string()),
            def_create_mail_spool: Some("yes".to_string()),
            def_log_init: Some("yes".to_string()),
            def_inactive: None,
            def_expire: None,
        }
    }

    ///
    pub fn from_file(&mut self, filename: &str) -> UResult<()> {
        let file = File::open(filename)
            .map_err_context(|| format!("Failed to open file: {}", filename))?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line =
                line.map_err_context(|| format!("Failed to read line from file: {}", filename))?;
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                match key.trim() {
                    "GROUP" => self.def_group = Some(value.trim().parse().unwrap_or(1000)),
                    "GROUPS" => self.def_groups = Some(value.trim().to_string()),
                    "HOME" => self.def_home = Some(value.trim().to_string()),
                    "SHELL" => self.def_shell = Some(value.trim().to_string()),
                    "INACTIVE" => {
                        let parsed_value: i64 = value.trim().parse().unwrap_or(-1);
                        self.def_inactive = if parsed_value == -1 {
                            None
                        } else {
                            Some(parsed_value)
                        };
                    }
                    "EXPIRE" => self.def_expire = Some(value.trim().to_string()),
                    "SKEL" => self.def_template = Some(value.trim().to_string()),
                    "USRSKEL" => self.def_usrtemplate = Some(value.trim().to_string()),
                    "CREATE_MAIL_SPOOL" => {
                        self.def_create_mail_spool = Some(value.trim().to_string())
                    }
                    "LOG_INIT" => self.def_log_init = Some(value.trim().to_string()),
                    _ => {}
                }
            }
        }

        Ok(())
    }

    ///
    pub fn show_defaults(&self) {
        println!("GROUP={}", self.def_group.unwrap_or(1000));
        println!("GROUPS={}", self.def_groups.as_deref().unwrap_or(""));
        println!("HOME={}", self.def_home.as_deref().unwrap_or("/home"));
        println!("SHELL={}", self.def_shell.as_deref().unwrap_or("/bin/bash"));
        println!("INACTIVE={}", self.def_inactive.unwrap_or(-1));
        println!("EXPIRE={}", self.def_expire.as_deref().unwrap_or(""));
        println!(
            "SKEL={}",
            self.def_template.as_deref().unwrap_or("/etc/skel")
        );
        println!(
            "USRSKEL={}",
            self.def_usrtemplate.as_deref().unwrap_or("/usr/etc/skel")
        );
        println!(
            "CREATE_MAIL_SPOOL={}",
            self.def_create_mail_spool.as_deref().unwrap_or("yes")
        );
        println!("LOG_INIT={}", self.def_log_init.as_deref().unwrap_or("yes"));
    }

    ///
    pub fn set_defaults(&self, filename: &str) -> UResult<()> {
        let mut new_lines: Vec<String> = Vec::new();
        let mut existing_settings: LinkedHashMap<String, String> = LinkedHashMap::new();
        let default_file = filename;
        let new_file = "/etc/default/new_useraddXXXXXX";

        let file = File::open(default_file)
            .map_err_context(|| format!("Failed to open file: {}", default_file))?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line
                .map_err_context(|| format!("Failed to read line from file: {}", default_file))?;
            let trimmed_line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = trimmed_line.split_once('=') {
                existing_settings.insert(key.trim().to_string(), value.trim().to_string());
            }
        }

        if let Some(def_group) = &self.def_group {
            existing_settings.insert("GROUP".to_string(), def_group.to_string());
        }
        if let Some(def_groups) = &self.def_groups {
            existing_settings.insert("GROUPS".to_string(), def_groups.clone());
        }
        if let Some(def_home) = &self.def_home {
            existing_settings.insert("HOME".to_string(), def_home.clone());
        }
        if let Some(def_shell) = &self.def_shell {
            existing_settings.insert("SHELL".to_string(), def_shell.clone());
        }
        if let Some(def_inactive) = &self.def_inactive {
            existing_settings.insert("INACTIVE".to_string(), def_inactive.to_string());
        }
        if let Some(def_expire) = &self.def_expire {
            existing_settings.insert("EXPIRE".to_string(), def_expire.clone());
        }
        if let Some(def_template) = &self.def_template {
            existing_settings.insert("SKEL".to_string(), def_template.clone());
        }
        if let Some(def_usrtemplate) = &self.def_usrtemplate {
            existing_settings.insert("USRSKEL".to_string(), def_usrtemplate.clone());
        }
        if let Some(def_create_mail_spool) = &self.def_create_mail_spool {
            existing_settings.insert(
                "CREATE_MAIL_SPOOL".to_string(),
                def_create_mail_spool.clone(),
            );
        }
        if let Some(def_log_init) = &self.def_log_init {
            existing_settings.insert("LOG_INIT".to_string(), def_log_init.clone());
        }

        let file = File::open(default_file)
            .map_err_context(|| format!("Failed to open file: {}", default_file))?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line
                .map_err_context(|| format!("Failed to read line from file: {}", default_file))?;
            let trimmed_line = line.trim();
            if trimmed_line.is_empty() || trimmed_line.starts_with('#') {
                new_lines.push(line);
            } else if let Some((key, _)) = trimmed_line.split_once('=') {
                if let Some(new_value) = existing_settings.get(key.trim()) {
                    new_lines.push(format!("{}={}", key.trim(), new_value));
                    existing_settings.remove(key.trim());
                } else {
                    new_lines.push(line);
                }
            }
        }

        for (key, value) in existing_settings {
            new_lines.push(format!("{}={}", key, value));
        }

        let mut temp_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(new_file)
            .map_err_context(|| format!("Failed to create temporary file: {}", new_file))?;
        for line in new_lines {
            writeln!(temp_file, "{}", line)
                .map_err_context(|| format!("Failed to write to temporary file: {}", new_file))?;
        }

        temp_file
            .flush()
            .map_err_context(|| format!("Failed to flush temporary file: {}", new_file))?;
        drop(temp_file);

        rename(new_file, default_file).map_err_context(|| {
            format!(
                "Failed to rename temporary file to default file: {}",
                default_file
            )
        })?;

        Ok(())
    }

    /// Getter methods
    pub fn def_group(&self) -> Option<u32> {
        self.def_group
    }

    ///
    pub fn def_groups(&self) -> Option<&str> {
        self.def_groups.as_deref()
    }

    ///
    pub fn def_gname(&self) -> Option<&str> {
        self.def_gname.as_deref()
    }

    ///
    pub fn def_home(&self) -> Option<&str> {
        self.def_home.as_deref()
    }

    ///
    pub fn def_shell(&self) -> Option<String> {
        self.def_shell.as_deref().map(|s| s.to_string())
    }

    ///
    pub fn def_template(&self) -> Option<&str> {
        self.def_template.as_deref()
    }

    ///
    pub fn def_usrtemplate(&self) -> Option<&str> {
        self.def_usrtemplate.as_deref()
    }

    ///
    pub fn def_create_mail_spool(&self) -> Option<String> {
        self.def_create_mail_spool.as_deref().map(|s| s.to_string())
    }

    ///
    pub fn def_log_init(&self) -> Option<&str> {
        self.def_log_init.as_deref()
    }

    ///
    pub fn def_inactive(&self) -> Option<i64> {
        self.def_inactive
    }

    ///
    pub fn def_expire(&self) -> Option<&str> {
        self.def_expire.as_deref()
    }

    /// Setter methods
    pub fn set_def_group(&mut self, value: u32) {
        self.def_group = Some(value);
    }

    ///
    pub fn set_def_groups(&mut self, value: String) {
        self.def_groups = Some(value);
    }

    ///
    pub fn set_def_gname(&mut self, value: String) {
        self.def_gname = Some(value);
    }

    ///
    pub fn set_def_home(&mut self, value: String) {
        self.def_home = Some(value);
    }

    ///
    pub fn set_def_shell(&mut self, value: String) {
        self.def_shell = Some(value);
    }

    ///
    pub fn set_def_template(&mut self, value: String) {
        self.def_template = Some(value);
    }

    ///
    pub fn set_def_usrtemplate(&mut self, value: String) {
        self.def_usrtemplate = Some(value);
    }

    ///
    pub fn set_def_create_mail_spool(&mut self, value: String) {
        self.def_create_mail_spool = Some(value);
    }

    ///
    pub fn set_def_log_init(&mut self, value: String) {
        self.def_log_init = Some(value);
    }

    ///
    pub fn set_def_inactive(&mut self, value: i64) {
        self.def_inactive = if value == -1 { None } else { Some(value) };
    }

    ///
    pub fn set_def_expire(&mut self, value: String) {
        self.def_expire = Some(value);
    }
}

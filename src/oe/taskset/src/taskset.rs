//! This file is part of the easybox package.
//
// (c) Jiale Xiao <xiao-xjle@qq.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use clap::Command;
use nix::unistd::Pid;
use procfs::process::Process;
use uucore::{error::UResult, help_section, help_usage};

/// Taskset common functions
pub mod taskset_common;

/// Helper functions to operate CPU_SET structure
pub mod lib_cpuset;

const ABOUT: &str = help_section!("about", "taskset.md");
const USAGE: &str = help_usage!("taskset.md");

#[uucore::main]
/// This the main of taskset
///
pub fn oemain(args: impl uucore::Args) -> UResult<()> {
    let mut config: taskset_common::Config =
        taskset_common::parse_taskset_cmd_args(args, ABOUT, USAGE)?;
    if config.all_tasks && config.command.is_none() {
        let pc = match Process::new(config.pid.into()) {
            Ok(v) => v,
            _ => return Ok(()), // Process not exist, we just finish normally.
        };
        let pc_tasks_it = pc.tasks().unwrap();
        for task in pc_tasks_it {
            config.pid = Pid::from_raw(task.unwrap().tid);
            taskset_common::do_taskset(&mut config)?;
        }
    } else {
        taskset_common::do_taskset(&mut config)?;
    }

    if config.command.is_some() {
        taskset_common::run_program(config.command.unwrap())?;
    }

    Ok(())
}

/// This the oe_app of taskset
///
pub fn oe_app<'a>() -> Command<'a> {
    taskset_common::taskset_app(ABOUT, USAGE)
}

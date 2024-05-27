// This file is part of the easybox package.
//
// (c) Haopeng Liu <657407891@qq.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.
//

use crate::common::util::*;
use std::path::Path;
const C_LSPCI_PATH: &str = "/usr/bin/lspci";

#[test]
fn test_lspci_path() {
    let test_args = &["-P"];
    let task = TestScenario::new(util_name!());
    let lspci_path = Path::new(C_LSPCI_PATH);
    if lspci_path.exists() {
        let res = task.cmd(C_LSPCI_PATH).args(test_args).succeeds();
        task.ucmd()
            .args(test_args)
            .succeeds()
            .stdout_only(res.stdout_str());
    } else {
        task.ucmd().args(test_args).succeeds();
    }
}

#[test]
fn test_lspci_dns() {
    let lspci_path = Path::new(C_LSPCI_PATH);
    let dig_path = Path::new("/usr/bin/dig");
    if !dig_path.exists() {
        let task = TestScenario::new(util_name!());
        task.ucmd().succeeds();
        return;
    }
    if lspci_path.exists() {
        let test_argsq = &["-q"];
        let taskq = TestScenario::new(util_name!());
        let resq = taskq.cmd(C_LSPCI_PATH).args(test_argsq).succeeds();
        taskq
            .ucmd()
            .args(test_argsq)
            .succeeds()
            .stdout_only(resq.stdout_str());

        let test_argsqq = &["-qq"];
        let taskqq = TestScenario::new(util_name!());
        let resqq = taskqq.cmd(C_LSPCI_PATH).args(test_argsqq).succeeds();
        taskqq
            .ucmd()
            .args(test_argsqq)
            .succeeds()
            .stdout_only(resqq.stdout_str());

        let test_args_q = &["-Q"];
        let task_q = TestScenario::new(util_name!());
        let res_q = task_q.cmd(C_LSPCI_PATH).args(test_args_q).succeeds();
        task_q
            .ucmd()
            .args(test_args_q)
            .succeeds()
            .stdout_only(res_q.stdout_str());
    } else {
        let test_argsq = &["-q"];
        let taskq = TestScenario::new(util_name!());
        taskq.ucmd().args(test_argsq).succeeds();

        let test_argsqq = &["-qq"];
        let taskqq = TestScenario::new(util_name!());
        taskqq.ucmd().args(test_argsqq).succeeds();

        let test_args_q = &["-Q"];
        let task_q = TestScenario::new(util_name!());
        task_q.ucmd().args(test_args_q).succeeds();
    };
}

#[test]
fn test_lspci_tree() {
    let lspci_path = Path::new(C_LSPCI_PATH);
    if lspci_path.exists() {
        let test_args = &["-t"];
        let task = TestScenario::new(util_name!());
        let res = task.cmd(C_LSPCI_PATH).args(test_args).succeeds();
        task.ucmd()
            .args(test_args)
            .succeeds()
            .stdout_only(res.stdout_str());

        let test_args1 = &["-t", "-v"];
        let task1 = TestScenario::new(util_name!());
        let res1 = task1.cmd(C_LSPCI_PATH).args(test_args1).succeeds();
        task1
            .ucmd()
            .args(test_args1)
            .succeeds()
            .stdout_only(res1.stdout_str());
    } else {
        let test_args = &["-t"];
        let task = TestScenario::new(util_name!());
        task.ucmd().args(test_args).succeeds();

        let test_args1 = &["-t", "-v"];
        let task1 = TestScenario::new(util_name!());
        task1.ucmd().args(test_args1).succeeds();
    }
}

#[test]
fn test_lspci_bus_centric() {
    let lspci_path = Path::new(C_LSPCI_PATH);
    if lspci_path.exists() {
        let test_args = &["-b"];
        let task = TestScenario::new(util_name!());
        let res = task.cmd(C_LSPCI_PATH).args(test_args).succeeds();
        task.ucmd()
            .args(test_args)
            .succeeds()
            .stdout_only(res.stdout_str());
    } else {
        let test_args = &["-b"];
        let task = TestScenario::new(util_name!());
        task.ucmd().args(test_args).succeeds();
    }
}

#[test]
fn test_lspci_map_mode() {
    let lspci_path = Path::new(C_LSPCI_PATH);
    if lspci_path.exists() {
        let test_args = &["-M"];
        let task = TestScenario::new(util_name!());
        let res = task.cmd(C_LSPCI_PATH).args(test_args).succeeds();
        task.ucmd()
            .args(test_args)
            .succeeds()
            .stdout_only(res.stdout_str());
    } else {
        let test_args = &["-M"];
        let task = TestScenario::new(util_name!());
        task.ucmd().args(test_args).succeeds();
    }
}

#[test]
fn test_lspci_v() {
    let lspci_path = Path::new(C_LSPCI_PATH);
    if lspci_path.exists() {
        let test_args = &["-v"];
        let task = TestScenario::new(util_name!());
        let refe = task.cmd(C_LSPCI_PATH).args(test_args).run();
        let resu = task.ucmd().args(test_args).run();
        let reflen = refe.stdout_str().split('\n').collect::<Vec<&str>>().len();
        let reslen = resu.stdout_str().split('\n').collect::<Vec<&str>>().len();
        assert!(reflen >= reslen);
    } else {
        let test_args = &["-v"];
        let task = TestScenario::new(util_name!());
        task.ucmd().args(test_args).succeeds();
    }
}

#[test]
fn test_lspci_kernel() {
    let lspci_path = Path::new(C_LSPCI_PATH);
    if lspci_path.exists() {
        let test_args = &["-k"];
        let task = TestScenario::new(util_name!());
        let refe = task.cmd(C_LSPCI_PATH).args(test_args).run();
        let resu = task.ucmd().args(test_args).run();

        assert_eq!(
            refe.stdout_str().split('\n').collect::<Vec<&str>>().len(),
            resu.stdout_str().split('\n').collect::<Vec<&str>>().len()
        );
    } else {
        let test_args = &["-k"];
        let task = TestScenario::new(util_name!());
        task.ucmd().args(test_args).succeeds();
    }
}

#[test]
fn test_lspci_hex_dump() {
    let lspci_path = Path::new(C_LSPCI_PATH);
    if lspci_path.exists() {
        let test_argsx = &["-x"];
        let taskx = TestScenario::new(util_name!());
        let resx = taskx.cmd(C_LSPCI_PATH).args(test_argsx).succeeds();
        taskx
            .ucmd()
            .args(test_argsx)
            .succeeds()
            .stdout_only(resx.stdout_str());

        let test_argsxxx = &["-xxx"];
        let taskxxx = TestScenario::new(util_name!());
        let resxxx = taskxxx.cmd(C_LSPCI_PATH).args(test_argsxxx).succeeds();
        taskxxx
            .ucmd()
            .args(test_argsxxx)
            .succeeds()
            .stdout_only(resxxx.stdout_str());

        let test_argsxxxx = &["-xxxx"];
        let taskxxxx = TestScenario::new(util_name!());
        let resxxxx = taskxxxx.cmd(C_LSPCI_PATH).args(test_argsxxxx).succeeds();
        taskxxxx
            .ucmd()
            .args(test_argsxxxx)
            .succeeds()
            .stdout_only(resxxxx.stdout_str());
    } else {
        let test_argsx = &["-x"];
        let taskx = TestScenario::new(util_name!());
        taskx.ucmd().args(test_argsx).succeeds();

        let test_argsxxx = &["-xxx"];
        let taskxxx = TestScenario::new(util_name!());
        taskxxx.ucmd().args(test_argsxxx).succeeds();

        let test_argsxxxx = &["-xxxx"];
        let taskxxxx = TestScenario::new(util_name!());
        taskxxxx.ucmd().args(test_argsxxxx).succeeds();
    }
}

#[test]
fn test_lspci_n() {
    let lspci_path = Path::new(C_LSPCI_PATH);
    if lspci_path.exists() {
        let test_args = &["-n"];
        let task = TestScenario::new(util_name!());
        let res = task.cmd(C_LSPCI_PATH).args(test_args).succeeds();
        task.ucmd()
            .args(test_args)
            .succeeds()
            .stdout_only(res.stdout_str());

        let test_argsnn = &["-nn"];
        let tasknn = TestScenario::new(util_name!());
        let resnn = tasknn.cmd(C_LSPCI_PATH).args(test_argsnn).succeeds();
        tasknn
            .ucmd()
            .args(test_argsnn)
            .succeeds()
            .stdout_only(resnn.stdout_str());
    } else {
        let test_args = &["-n"];
        let task = TestScenario::new(util_name!());
        task.ucmd().args(test_args).succeeds();

        let test_argsnn = &["-nn"];
        let tasknn = TestScenario::new(util_name!());
        tasknn.ucmd().args(test_argsnn).succeeds();
    }
}

#[test]
fn test_lspci_select_slots() {
    let lspci_path = Path::new(C_LSPCI_PATH);
    if lspci_path.exists() {
        let test_args = &["-s", "0:1"];
        let task = TestScenario::new(util_name!());
        let res = task.cmd(C_LSPCI_PATH).args(test_args).succeeds();
        task.ucmd()
            .args(test_args)
            .succeeds()
            .stdout_only(res.stdout_str());
    } else {
        let test_args = &["-s", "0:1"];
        let task = TestScenario::new(util_name!());
        task.ucmd().args(test_args).succeeds();
    }
}

#[test]
fn test_lspci_select_id() {
    let lspci_path = Path::new(C_LSPCI_PATH);
    if lspci_path.exists() {
        let test_args = &["-d", "8086:293e"];
        let task = TestScenario::new(util_name!());
        let res = task.cmd(C_LSPCI_PATH).args(test_args).succeeds();
        task.ucmd()
            .args(test_args)
            .succeeds()
            .stdout_only(res.stdout_str());
    } else {
        let test_args = &["-d", "8086:293e"];
        let task = TestScenario::new(util_name!());
        task.ucmd().args(test_args).succeeds();
    }
}

#[test]
fn test_lspci_machine() {
    let lspci_path = Path::new(C_LSPCI_PATH);
    if lspci_path.exists() {
        let test_args = &["-m"];
        let task = TestScenario::new(util_name!());
        let res = task.cmd(C_LSPCI_PATH).args(test_args).succeeds();
        task.ucmd()
            .args(test_args)
            .succeeds()
            .stdout_only(res.stdout_str());
    } else {
        let test_args = &["-m"];
        let task = TestScenario::new(util_name!());
        task.ucmd().args(test_args).succeeds();
    }
}

#[test]
fn test_lspci_no_arg() {
    let lspci_path = Path::new(C_LSPCI_PATH);
    if lspci_path.exists() {
        let task = TestScenario::new(util_name!());
        let res = task.cmd(C_LSPCI_PATH).succeeds();
        task.ucmd().succeeds().stdout_only(res.stdout_str());
    } else {
        let task = TestScenario::new(util_name!());
        task.ucmd().succeeds();
    }
}

#[test]
fn test_lspci_fixed_arg1() {
    let lspci_path = Path::new(C_LSPCI_PATH);
    let dig_path = Path::new("/usr/bin/dig");
    if !dig_path.exists() {
        let task = TestScenario::new(util_name!());
        task.ucmd().succeeds();
        return;
    }
    if lspci_path.exists() {
        let test_args = &["-q", "-s", "0:1"];
        let task = TestScenario::new(util_name!());
        let res = task.cmd(C_LSPCI_PATH).args(test_args).succeeds();
        task.ucmd()
            .args(test_args)
            .succeeds()
            .stdout_only(res.stdout_str());
    } else {
        let test_args = &["-q", "-s", "0:1"];
        let task = TestScenario::new(util_name!());
        task.ucmd().args(test_args).succeeds();
    }
}

#[test]
fn test_lspci_fixed_arg2() {
    let lspci_path = Path::new(C_LSPCI_PATH);
    if lspci_path.exists() {
        let test_args = &["-d", "8086:293e", "-m"];
        let task = TestScenario::new(util_name!());
        let res = task.cmd(C_LSPCI_PATH).args(test_args).succeeds();
        task.ucmd()
            .args(test_args)
            .succeeds()
            .stdout_only(res.stdout_str());
    } else {
        let test_args = &["-d", "8086:293e", "-m"];
        let task = TestScenario::new(util_name!());
        task.ucmd().args(test_args).succeeds();
    }
}

#[test]
fn test_lspci_ilegal_args() {
    let test_args = &["-1"];
    let task = TestScenario::new(util_name!());
    task.ucmd().args(test_args).fails().code_is(1);
}

use crate::common::util::*;
use lazy_static::lazy_static;
use regex::Regex;
const C_FREE_PATH: &str = "/usr/bin/free";
lazy_static! {
    static ref RE_MEM: Regex = Regex::new(r"(?m)^Mem:\s+\d+\s+\d+\s+\d+\s+\d+\s+\d+\s+\d+$").unwrap();
    static ref RE_SWAP: Regex = Regex::new(r"(?m)^Swap:\s+\d+\s+\d+\s+\d+$").unwrap();
    static ref RE_TOTAL: Regex = Regex::new(r"(?m)^Total:\s+\d+\s+\d+\s+\d+$").unwrap();
    static ref RE_LOW: Regex = Regex::new(r"(?m)^Low:\s+\d+\s+\d+\s+\d+$").unwrap();
    static ref RE_HIGH: Regex = Regex::new(r"(?m)^High:\s+\d+\s+\d+\s+\d+$").unwrap();
    static ref TITLE:Regex = Regex::new("               total        used        free      shared  buff/cache   available").unwrap();
    static ref TITLE_W:Regex = Regex::new("               total        used        free      shared     buffers       cache   available").unwrap();
    static ref RE_MEM_W:Regex = Regex::new(r"(?m)^Mem:\s+\d+\s+\d+\s+\d+\s+\d+\s+\d+\s+\d+\s+\d+$").unwrap();
}
fn both(regs: Vec<&Regex>, c_res: CmdResult, res: CmdResult) {
    for re in regs {
        assert!(re.is_match(std::str::from_utf8(&c_res.stdout()).expect("Not UTF8")));
        assert!(re.is_match(std::str::from_utf8(&res.stdout()).expect("Not UTF8")));
    }
}

fn both_times(regs: Vec<&Regex>, c_res: CmdResult, res: CmdResult) {
    for re in regs {
        assert_eq!(
            re.find_iter(std::str::from_utf8(&c_res.stdout()).expect("Not UTF8"))
                .count(),
            re.find_iter(std::str::from_utf8(&res.stdout()).expect("Not UTF8"))
                .count()
        );
    }
}

#[test]
fn test_free() {
    let task = TestScenario::new(util_name!());
    // Run original C taskset from system path
    let c_res = task.cmd(C_FREE_PATH).succeeds();
    // Run Rust taskset and compare the output with origin one
    let res = task.ucmd().succeeds();
    both(vec![&*TITLE, &*RE_MEM, &*RE_SWAP], c_res, res);
}

#[test]
fn test_free_t() {
    let test_args = &["-t"];
    let task = TestScenario::new(util_name!());
    // Run original C taskset from system path
    let c_res = task.cmd(C_FREE_PATH).args(test_args).succeeds();
    // Run Rust taskset and compare the output with origin one
    let res = task.ucmd().args(test_args).succeeds();
    both(vec![&*TITLE, &*RE_TOTAL], c_res, res);
}

#[test]
fn test_free_h() {
    let test_args = &["-h"];
    let task = TestScenario::new(util_name!());
    // Run original C taskset from system path
    let c_res = task.cmd(C_FREE_PATH).args(test_args).succeeds();
    // Run Rust taskset and compare the output with origin one
    let res = task.ucmd().args(test_args).succeeds();
    both(vec![&*TITLE], c_res, res);
}

#[test]
fn test_free_s() {
    let test_args = &["-c", "3", "-s", "1"];
    let task = TestScenario::new(util_name!());
    // Run original C taskset from system path
    let c_res = task.cmd(C_FREE_PATH).args(test_args).succeeds();
    // Run Rust taskset and compare the output with origin one
    let res = task.ucmd().args(test_args).succeeds();
    both_times(vec![&*TITLE, &*RE_MEM, &*RE_SWAP], c_res, res);
}

#[test]
fn test_free_l() {
    let test_args = &["-l"];
    let task = TestScenario::new(util_name!());
    // Run original C taskset from system path
    let c_res = task.cmd(C_FREE_PATH).args(test_args).succeeds();
    // Run Rust taskset and compare the output with origin one
    let res = task.ucmd().args(test_args).succeeds();
    both(vec![&*TITLE, &*RE_LOW, &*RE_HIGH], c_res, res);
}

#[test]
fn test_free_muti_unit() {
    let test_args = &["-m", "-k"];
    let task = TestScenario::new(util_name!());
    // Run original C taskset from system path
    task.cmd(C_FREE_PATH).args(test_args).fails();
    // Run Rust taskset and compare the output with origin one
    task.ucmd().args(test_args).fails();
}

#[test]
fn test_free_units() {
    let units = ["kilo", "mega", "giga", "tera", "peta", "tebi", "pebi", "si"];
    for unit in units.iter() {
        let test_arg = vec![format!("--{}", unit)];
        let task = TestScenario::new(util_name!());
        // Run original C taskset from system path
        let c_res = task.cmd(C_FREE_PATH).args(&test_arg).succeeds();
        // Run Rust taskset and compare the output with origin one
        let res = task.ucmd().args(&test_arg).succeeds();
        both(vec![&*TITLE, &*RE_MEM, &*RE_SWAP], c_res, res);
    }
    let units = ["k", "m", "g", "t"];
    for unit in units.iter() {
        let test_arg = vec![format!("-{}", unit)];
        let task = TestScenario::new(util_name!());
        // Run original C taskset from system path
        let c_res = task.cmd(C_FREE_PATH).args(&test_arg).succeeds();
        // Run Rust taskset and compare the output with origin one
        let res = task.ucmd().args(&test_arg).succeeds();
        both(vec![&*TITLE, &*RE_MEM, &*RE_SWAP], c_res, res);
    }
}

#[test]
fn test_free_invalid_unit() {
    new_ucmd!().arg("-i").fails();
}

#[test]
fn test_free_w() {
    let test_args = &["-w"];
    let task = TestScenario::new(util_name!());
    // Run original C taskset from system path
    let c_res = task.cmd(C_FREE_PATH).args(test_args).succeeds();
    // Run Rust taskset and compare the output with origin one
    let res = task.ucmd().args(test_args).succeeds();
    both(vec![&*TITLE_W, &*RE_MEM_W, &*RE_SWAP], c_res, res);
}

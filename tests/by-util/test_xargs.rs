//! This file is part of the easybox package.
//
// (c) Wentong Yang <ywt0821@163.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.
use crate::common::util::*;
use std::fs;

#[test]
fn test_xargs_0_n3() {
    // xargs  -0 -n3 < files0.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/files0.xi";

    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-0", "-n3"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["-0", "-n3"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_e_0() {
    // xargs  -E_ -0 < eof_-0.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/eof_-0.xi";

    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-E_", "-0"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["-E_", "-0"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_l2_0_ldata_0() {
    // xargs  -L2 -0 < ldata-0.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/ldata-0.xi";

    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-L2", "-0"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["-L2", "-0"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_l2_0_ldatab_0() {
    // xargs  -L2 -0 < ldatab-0.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/ldatab-0.xi";

    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-L2", "-0"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["-L2", "-0"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_l3_0_ldata_0() {
    // xargs  -L3 -0 < ldata-0.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/ldata-0.xi";

    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-L3", "-0"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["-L3", "-0"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

// #[test]
// fn test_xargs_p3_n1_larg() {
//     // xargs  -P3 -n1 -IARG sh -c ARG < Pdata.xi
//     let ts = TestScenario::new("xargs");
//     let input_file_path = "tests/fixtures/xargs/Pdata.xi";

//     let expect_result = ts
//         .cmd_keepenv("/usr/bin/xargs")
//         .args(&["-P3","-n1", "-IARG", "sh", "-c", "ARG"])
//         .pipe_in(fs::read(input_file_path).unwrap())
//         .run();

//     let actual_result = ts
//         .ucmd()
//         .args(&["-P3","-n1", "-IARG", "sh", "-c", "ARG"])
//         .pipe_in(fs::read(input_file_path).unwrap())
//         .run();

//     assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
// }

#[test]
fn test_xargs_delim_o() {
    // xargs  -d o -n1 < helloworld.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/ldata-0.xi";

    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-d", "-o", "-n1"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["-d", "-o", "-n1"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_empty_r() {
    // xargs  -r echo this plus that < /dev/null
    let ts = TestScenario::new("xargs");
    let input_file_path = "/dev/null";

    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["r", "echo", "this plus that"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["r", "echo", "this plus that"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_empty_def_r() {
    // xargs  -r < /dev/null
    let ts = TestScenario::new("xargs");
    let input_file_path = "/dev/null";

    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["r"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["r"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_idef_0() {
    // xargs  -i -0 echo from \{\} to x{}y < items-0.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/items-0.xi";

    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-i", "-0", "echo", "from \\{\\} to x{}y"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["-i", "-0", "echo", "from \\{\\} to x{}y"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_idef_s26_0() {
    // xargs  -i -s26 -0 echo from \{\} to x{}y < items-0.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/items-0.xi";

    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-i", "-s26", "-0", "echo", "from {} to x{}y"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["-i", "-s26", "-0", "echo", "from {} to x{}y"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_l1_0() {
    // xargs  -l -0 < ldata-0.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/ldata-0.xi";

    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-l", "-0"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["-l", "-0"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_l1_2_0() {
    // xargs  -l -0 < ldatab-0.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/ldatab-0.xi";

    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-l", "-0"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["-l", "-0"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_n1_0() {
    // xargs  -n1 -0 < stairs-0.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/stairs-0.xi";

    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-n1", "-0"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["-n1", "-0"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_n2_0() {
    // xargs  -n2 -0 < stairs-0.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/stairs-0.xi";

    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-n2", "-0"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["-n2", "-0"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_n2_s21_0() {
    // xargs  -n2 -s21 -0 < stairs-0.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/stairs-0.xi";

    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-n2", "-s21", "-0"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["-n2", "-s21", "-0"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_n2_s21_x_0() {
    // xargs  -n2 -s21 -x -0 < stairs-0.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/stairs-0.xi";

    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-n2", "-s21", "-x", "-0"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["-n2", "-s21", "-x", "-0"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_n3_0() {
    // xargs  -n3 -0 < stairs2-0.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/stairs2-0.xi";

    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-n3", "-0"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["-n3", "-0"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_n3_s31_0() {
    // xargs  -n3 -s31 -0 < stairs2-0.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/stairs2-0.xi";

    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-n3", "-s31", "-0"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["-n3", "-s31", "-0"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_noeof_0() {
    // xargs  -0 < noeof-0.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/noeof-0.xi";

    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-0"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["-0"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_nothing() {
    // xargs  echo this plus that < /dev/null
    let ts = TestScenario::new("xargs");
    let input_file_path = "/dev/null";

    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["echo", "this plus that"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["echo", "this plus that"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_r() {
    // xargs  -r echo this plus that < blank.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/blank.xi";

    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-r", "echo", "this plus that"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["-r", "echo", "this plus that"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_s118_0() {
    // xargs  -0 -s118 < stairs-0.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/stairs-0.xi";

    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-0", "-s118"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["-0", "-s118"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_s14_0() {
    // xargs  -0 -s14 < stairs-0.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/stairs-0.xi";

    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-0", "-s14"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["-0", "-s14"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_s14_2_0() {
    // xargs  -0 -s14 < stairs2-0.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/stairs2-0.xi";

    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-0", "-s14"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["-0", "-s14"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_s15_0() {
    // xargs  -0 -s15 < stairs-0.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/stairs-0.xi";

    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-0", "-s15"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["-0", "-s15"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_s25_0() {
    // xargs  -0 -s25 < stairs-0.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/stairs-0.xi";

    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-0", "-s25"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["-0", "-s25"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_space_0() {
    // xargs  -0 echo this plus that < space.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/stairs-0.xi";

    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-0", "-s25"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["-0", "-s25"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_space_r() {
    // xargs  -r echo this plus that < space.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/space.xi";

    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-r", "echo", "this plus that"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["-r", "echo", "this plus that"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_space_t_0() {
    // xargs  -t -0 echo this plus that < space.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/space.xi";

    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-t", "-0", "echo", "this plus that"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["-t", "-0", "echo", "this plus that"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_eeof() {
    // xargs  -E EOF < EOF.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/EOF.xi";

    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-E", "EOF"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["-E", "EOF"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_eeofb() {
    // xargs  -E EOF < EOFb.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/EOFb.xi";

    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-E", "EOF"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["-E", "EOF"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_eeeofe() {
    // xargs  -E EOF < EOFe.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/EOFe.xi";

    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-E", "EOF"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["-E", "EOF"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_e_iarg() {
    // xargs  -E_ -IARG echo from ARG to xARGy < eof_.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/eof_.xi";

    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-E_", "-IARG", "echo", "from", "ARG", "to", "xARGy"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["-E_", "-IARG", "echo", "from", "ARG", "to", "xARGy"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_e_() {
    // xargs  -E_ < eof_.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/eof_.xi";

    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-E_"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["-E_"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_iarg_e_() {
    // xargs  -IARG echo from ARG to xARGy -E_ < eof_.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/eof_.xi";

    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-IARG", "echo", "from", "ARG", "to", "xARGy", "-E_"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["-IARG", "echo", "from", "ARG", "to", "xARGy", "-E_"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_iarg_s15() {
    // xargs  -IARG -s15 echo ARG < stairs.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/stairs.xi";

    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-IARG", "-s100", "echo", "ARG"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["-IARG", "-s100", "echo", "ARG"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

// #[test]
// fn test_xargs_posix_iarg() {
//     // xargs  -IARG echo from ARG to xARGy < items.xi
//     let ts = TestScenario::new("xargs");
//     let input_file_path = "tests/fixtures/xargs/items.xi";
//     // fs::read_to_string(input_file_path);
//     let expect_result = ts
//         .cmd_keepenv("/usr/bin/xargs")
//         .args(&["-IARG", "echo", "from ARG to xARGy"])
//         .pipe_in(fs::read(input_file_path).unwrap())
//         .run();

//     let actual_result = ts
//         .ucmd()
//         .args(&["-IARG", "echo", "from ARG to xARGy"])
//         .pipe_in(fs::read(input_file_path).unwrap())
//         .run();

//     assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
// }

#[test]
fn test_xargs_l2_n2() {
    // xargs  -L2 -n2 < ldata.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/ldata.xi";

    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-L2", "-n2"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["-L2", "-n2"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_l2_2() {
    // xargs  -L2 < ldatab.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/ldatab.xi";

    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-L2"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["-L2"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_l3() {
    // xargs  -L3 < ldata.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/ldata.xi";
    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-L3"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    let actual_result = ts
        .ucmd()
        .args(&["-L3"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_arg_max_32bit_linux_bug() {
    // xargs   true  < 32767-ys.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/32767-ys.xi";
    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["true"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    let actual_result = ts
        .ucmd()
        .args(&["true"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_arg_max_64bit_linux_bug() {
    // xargs   true  < 16383-ys.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/16383-ys.xi";
    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["true"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    let actual_result = ts
        .ucmd()
        .args(&["true"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_childfail() {
    // xargs   false  < files.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/files.xi";
    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["false"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    let actual_result = ts
        .ucmd()
        .args(&["false"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_empty() {
    // xargs  echo this plus that < /dev/null
    let ts = TestScenario::new("xargs");
    let input_file_path = "/dev/null";
    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["echo", "this plus that"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    let actual_result = ts
        .ucmd()
        .args(&["echo", "this plus that"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_empty_def() {
    // xargs   < /dev/null
    let ts = TestScenario::new("xargs");
    let input_file_path = "/dev/null";
    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    let actual_result = ts.ucmd().pipe_in(fs::read(input_file_path).unwrap()).run();
    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_hithere() {
    // xargs  -s470 echo hi there < files.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/files.xi";
    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-s470", "echo", "hi there"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    let actual_result = ts
        .ucmd()
        .args(&["-s470", "echo", "hi there"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_n1() {
    // xargs  -n1 < stairs.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/stairs.xi";
    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-n1"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    let actual_result = ts
        .ucmd()
        .args(&["-n1"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_n2_s21_x() {
    // xargs  -n2 -s21 -x < stairs.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/stairs.xi";
    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-n2", "-s21", "-x"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    let actual_result = ts
        .ucmd()
        .args(&["-n2", "-s21", "-x"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_n2_s21() {
    // xargs  -n2 -s21 < stairs.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/stairs.xi";
    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-n2", "-s21"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    let actual_result = ts
        .ucmd()
        .args(&["-n2", "-s21"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_n2() {
    // xargs  -n2 < stairs.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/stairs.xi";
    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-n2"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    let actual_result = ts
        .ucmd()
        .args(&["-n2"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_n3_s31() {
    // xargs  -n3 -s31 < stairs2.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/stairs2.xi";
    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-n3", "-s31"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    let actual_result = ts
        .ucmd()
        .args(&["-n3", "-s31"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_n3() {
    // xargs  -n3 < files.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/files.xi";
    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-n3"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    let actual_result = ts
        .ucmd()
        .args(&["-n3"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_noeof() {
    // xargs   < noeof.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/noeof.xi";
    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    let actual_result = ts.ucmd().pipe_in(fs::read(input_file_path).unwrap()).run();
    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_quotes() {
    // xargs   < quotes.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/quotes.xi";
    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    let actual_result = ts.ucmd().pipe_in(fs::read(input_file_path).unwrap()).run();
    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_rc_123() {
    // xargs  -n1 -IARG sh -c ARG < ftt.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/ftt.xi";
    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-n1", "-IARG", "sh -c ARG"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    let actual_result = ts
        .ucmd()
        .args(&["-n1", "-IARG", "sh -c ARG"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_rc_124() {
    // xargs  -n1 -IARG sh -c ARG < ett.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/ett.xi";
    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-n1", "-IARG", "sh -c ARG"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    let actual_result = ts
        .ucmd()
        .args(&["-n1", "-IARG", "sh -c ARG"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_rc_125() {
    // xargs  -n1 -IARG sh -c ARG < stt.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/stt.xi";
    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-n1", "-IARG", "sh -c ARG"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    let actual_result = ts
        .ucmd()
        .args(&["-n1", "-IARG", "sh -c ARG"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_rc_126() {
    // xargs  / < /dev/null
    let ts = TestScenario::new("xargs");
    let input_file_path = "/dev/null";
    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["/"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    let actual_result = ts
        .ucmd()
        .args(&["/"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_rc_127() {
    // xargs  ./missing < /dev/null
    let ts = TestScenario::new("xargs");
    let input_file_path = "/dev/null";
    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["./missing"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    let actual_result = ts
        .ucmd()
        .args(&["./missing"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_s118() {
    // xargs  -s118 < stairs.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/stairs.xi";
    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-s118"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    let actual_result = ts
        .ucmd()
        .args(&["-s118"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_s14() {
    // xargs  -s14 < stairs.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/stairs.xi";
    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-s14"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    let actual_result = ts
        .ucmd()
        .args(&["-s14"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_s14_2() {
    // xargs  -s14 < stairs2.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/stairs2.xi";
    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-s14"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    let actual_result = ts
        .ucmd()
        .args(&["-s14"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_s15() {
    // xargs  -s15 < stairs.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/stairs.xi";
    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-s15"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    let actual_result = ts
        .ucmd()
        .args(&["-s15"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_s25() {
    // xargs  -s25 < stairs.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/stairs.xi";
    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-s25"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    let actual_result = ts
        .ucmd()
        .args(&["-s25"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_s42() {
    // xargs  -s42 < files.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/files.xi";
    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-s42"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    let actual_result = ts
        .ucmd()
        .args(&["-s42"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_s470() {
    // xargs  -s470 < files.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/files.xi";
    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-s470"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    let actual_result = ts
        .ucmd()
        .args(&["-s470"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_s48() {
    // xargs  -s48 < files.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/files.xi";
    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-s48"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    let actual_result = ts
        .ucmd()
        .args(&["-s48"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_s6() {
    // xargs  -s6 < files.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/files.xi";
    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-s6"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    let actual_result = ts
        .ucmd()
        .args(&["-s6"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_savannah_11865() {
    // xargs  -iARG -s86 echo ARG is xARGx < files.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/files.xi";
    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-iARG", "-s86", "echo", "ARG is xARGx"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    let actual_result = ts
        .ucmd()
        .args(&["-iARG", "-s86", "echo", "ARG is xARGx"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_space_l() {
    // xargs  -IARG echo from ARG to xARGy < space.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/space.xi";
    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-IARG", "echo", "from ARG to xARGy"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    let actual_result = ts
        .ucmd()
        .args(&["-IARG", "echo", "from ARG to xARGy"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_space() {
    // xargs  echo this plus that < space.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/space.xi";
    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["echo", "this plus that"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    let actual_result = ts
        .ucmd()
        .args(&["echo", "this plus that"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_sv_bug_18714() {
    // xargs  printf "\[%s\]\n" < formfeeds.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/formfeeds.xi";
    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["printf", r#""\[%s\]\n""#])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    let actual_result = ts
        .ucmd()
        .args(&["printf", r#""\[%s\]\n""#])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_sv_bug_18714b() {
    // xargs  printf "\[%s\]\n" < verticaltabs.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/files.xi";
    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["printf", r#""\[%s\]\n""#])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    let actual_result = ts
        .ucmd()
        .args(&["printf", r#""\[%s\]\n""#])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

// #[test]
// fn test_xargs_sv_bug_20273() {
//     // sh -c {/home/ywt/work/findutils-4.9.0/xargs/testsuite/../xargs  -E2; cat}  < sv-bug-20273.xi
// }

#[test]
fn test_xargs_uc_l2() {
    // xargs  -L2 < ldata.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/ldata.xi";
    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-L2"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    let actual_result = ts
        .ucmd()
        .args(&["-L2"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_unmatched_n2_x() {
    // xargs  -n2 -x < unmatched.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/unmatched.xi";
    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-n2", "-x"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    let actual_result = ts
        .ucmd()
        .args(&["-n2", "-x"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_unmatched() {
    // xargs < unmatched.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/unmatched.xi";
    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    let actual_result = ts.ucmd().pipe_in(fs::read(input_file_path).unwrap()).run();

    assert_eq!(expect_result.code(), actual_result.code());
}

#[test]
fn test_xargs_unmatched2() {
    // xargs < unmatched2.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/unmatched2.xi";
    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    let actual_result = ts.ucmd().pipe_in(fs::read(input_file_path).unwrap()).run();

    assert_eq!(expect_result.code(), actual_result.code());
}

#[test]
fn test_xargs_e() {
    // xargs  -e < eof_.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/eof_.xi";
    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-e"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    let actual_result = ts
        .ucmd()
        .args(&["-e"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_e_eof() {
    // xargs  -eEOF < eofstr.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/eofstr.xi";
    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-eEOF"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    let actual_result = ts
        .ucmd()
        .args(&["-eEOF"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_empty_t() {
    // xargs -t echo this plus that < /dev/null
    let ts = TestScenario::new("xargs");
    let input_file_path = "/dev/null";

    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-t", "echo", "this plus that"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["-t", "echo", "this plus that"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
    assert_eq!(expect_result.code(), actual_result.code());
}

#[test]
fn test_xargs_empty_def_t() {
    // xargs -t < /dev/null
    let ts = TestScenario::new("xargs");
    let input_file_path = "/dev/null";

    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-t"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["-t"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_eof1() {
    // xargs -E_ < eof1.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/eof1.xi";

    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-E_"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["-E_"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_iarg() {
    // xargs -iARG echo ARG is xARGx < files.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/files.xi";

    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-iARG", "echo", "ARG is xARGx"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["-iARG", "echo", "ARG is xARGx"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

// #[test]
// fn test_xargs_idef_s26() {
//     // xargs  -i -s26 echo from \{\} to x{}y < items.xi
// }

// #[test]
// fn test_xargs_idef() {
//     // xargs  -i echo from \{\} to x{}y < items.xi
//     let ts = TestScenario::new("xargs");
//     let input_file_path = "tests/fixtures/xargs/items.xi";

//     let expect_result = ts
//         .cmd_keepenv("/usr/bin/xargs")
//         .args(&["-i", "echo", "from \\{\\} to x{}y"])
//         .pipe_in(fs::read(input_file_path).unwrap())
//         .run();

//     let actual_result = ts
//         .ucmd()
//         .args(&["-i{}", "echo", "from \\{\\} to x{}y"])
//         .pipe_in(fs::read(input_file_path).unwrap())
//         .run();

//     assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
// }

// #[test]
// fn test_xargs_iquotes() {
//     // xargs  -i__ echo FIRST __ IS OK < quotes.xi
// }

#[test]
fn test_xargs_l1() {
    // xargs  -l < ldata.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/ldata.xi";

    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-l"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["-l"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_l1_2() {
    // xargs  -l < ldatab.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/ldatab.xi";

    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-l"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["-l"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_l1n4() {
    // xargs -l1 -n4 < files.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/files.xi";

    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-l1", "-n4"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["-l1", "-n4"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
    assert_eq!(expect_result.code(), actual_result.code());
}

#[test]
fn test_xargs_lc_l2() {
    // xargs -l2 < files.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/files.xi";

    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-l2"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["-l2"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_s25_t() {
    // xargs -s25 -t < stairs.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/stairs.xi";

    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-s25", "-t"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["-s25", "-t"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_space_t() {
    // xargs  -t echo this plus that < space.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/space.xi";

    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-t", "echo", "this plus that"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["-t", "echo", "this plus that"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_sv_bug_18713() {
    // xargs  -n1 printf "@%s@\n" < empty.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/empty.xi";
    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-n1", "printf", r#""@%s@\n""#])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();
    let actual_result = ts
        .ucmd()
        .args(&["-n1", "printf", r#""@%s@\n""#])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

#[test]
fn test_xargs_trace() {
    // xargs  -n2 -t echo  < foobar.xi
    let ts = TestScenario::new("xargs");
    let input_file_path = "tests/fixtures/xargs/foobar.xi";

    let expect_result = ts
        .cmd_keepenv("/usr/bin/xargs")
        .args(&["-n2", "-t", "echo"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    let actual_result = ts
        .ucmd()
        .args(&["-n2", "-t", "echo"])
        .pipe_in(fs::read(input_file_path).unwrap())
        .run();

    assert_eq!(expect_result.stdout_str(), actual_result.stdout_str());
}

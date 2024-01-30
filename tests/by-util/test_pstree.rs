// This file is part of the easybox package.
//
// (c) Allen Xu <xubo3006@163.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.
//

use crate::common::util::*;

#[test]
fn test_pstree() {
    new_ucmd!().succeeds().stdout_contains("easybox");
}

#[test]
fn test_pstree_pid() {
    new_ucmd!().arg("1").succeeds().stdout_contains("easybox");
}

#[test]
fn test_pstree_arg() {
    new_ucmd!().arg("-a").succeeds().stdout_contains("easybox");
}

#[test]
fn test_pstree_ascii() {
    new_ucmd!().arg("-A").succeeds().stdout_contains("easybox");
}

#[test]
fn test_pstree_compact() {
    new_ucmd!().arg("-c").succeeds().stdout_contains("easybox");
}

#[test]
fn test_pstree_color() {
    new_ucmd!()
        .args(&["-C", "age"])
        .succeeds()
        .stdout_contains("easybox");
}

#[test]
fn test_pstree_pgids() {
    new_ucmd!()
        .args(&["-g", "-l"])
        .succeeds()
        .stdout_contains("easybox");
}

#[test]
fn test_pstree_vt100() {
    new_ucmd!()
        .args(&["-G", "-l"])
        .succeeds()
        .stdout_contains("easybox");
}

#[test]
fn test_pstree_hiall() {
    new_ucmd!().arg("-h").succeeds().stdout_contains("easybox");
}

#[test]
fn test_pstree_hipid() {
    new_ucmd!()
        .args(&["-H", "1"])
        .succeeds()
        .stdout_contains("easybox");
}

#[test]
fn test_pstree_long() {
    new_ucmd!().arg("-l").succeeds().stdout_contains("easybox");
}

#[test]
fn test_pstree_sort() {
    new_ucmd!().arg("-n").succeeds().stdout_contains("easybox");
}

#[test]
fn test_pstree_ns() {
    new_ucmd!()
        .args(&["-N", "cgroup"])
        .succeeds()
        .stdout_contains("easybox");
}

#[test]
fn test_pstree_pids() {
    new_ucmd!()
        .args(&["-p", "-l"])
        .succeeds()
        .stdout_contains("easybox");
}

#[test]
fn test_pstree_parents() {
    new_ucmd!().arg("-s").succeeds().stdout_contains("easybox");
}

#[test]
fn test_pstree_nschange() {
    new_ucmd!().arg("-S").succeeds().stdout_contains("easybox");
}

#[test]
fn test_pstree_thread_names() {
    new_ucmd!().arg("-t").succeeds().stdout_contains("easybox");
}

#[test]
fn test_pstree_hide_threads() {
    new_ucmd!().arg("-T").succeeds().stdout_contains("easybox");
}

#[test]
fn test_pstree_uid_change() {
    new_ucmd!().arg("-u").succeeds().stdout_contains("easybox");
}

#[test]
fn test_pstree_unicode() {
    new_ucmd!().arg("-U").succeeds().stdout_contains("easybox");
}

#[test]
fn test_pstree_security() {
    new_ucmd!().arg("-Z").succeeds().stdout_contains("easybox");
}

// Due to different machine operating environments and different versions of pstree,
// please modify and run comparative tests according to the actual situation
// #[test]
// fn test_pstree_showpid() {
//     let pstree_out = Command::new("pstree")
//         .args(&["-p"])
//         .output()
//         .unwrap()
//         .stdout;
//     let pstree_out_str = remove_pstree_lines(String::from_utf8(pstree_out).unwrap());

//     let oepstree_out = new_ucmd!()
//         .args(&["-p"])
//         .succeeds()
//         .stdout_str()
//         .to_string();
//     let oepstree_out_str = remove_pstree_lines(oepstree_out);

//     assert_eq!(pstree_out_str, oepstree_out_str);
// }

// fn remove_pstree_lines(s: String) -> String {
//     s.lines()
//         .filter(|line| {
//             !line.contains("easybox") && !line.contains("pstree") && !line.contains("tests")
//         })
//         .map(|line| line.to_owned() + "\n")
//         .collect::<String>()
// }

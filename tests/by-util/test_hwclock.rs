use crate::common::util::*;
use chrono::{Local, NaiveDateTime, TimeZone};
use std::{fs::read_to_string, sync::Mutex};

static MUX: Mutex<()> = Mutex::new(());

const UUTILS_WARNING: &str = "uutils-tests-warning";

#[cfg(unix)]
pub fn run_ucmd_as_root_ignore_ci(
    ts: &TestScenario,
    args: &[&str],
) -> std::result::Result<CmdResult, String> {
    use std::process::Command;

    // check if we can run 'sudo'
    log_info("run", "sudo -E --non-interactive whoami");
    match Command::new("sudo")
        .env("LC_ALL", "C")
        .args(["-E", "--non-interactive", "whoami"])
        .output()
    {
        Ok(output) if String::from_utf8_lossy(&output.stdout).eq("root\n") => {
            // we can run sudo and we're root
            // run ucmd as root:
            Ok(ts
                .cmd_keepenv("sudo")
                .env("LC_ALL", "C")
                .arg("-E")
                .arg("--non-interactive")
                .arg(&ts.bin_path)
                .arg(&ts.util_name)
                .args(args)
                .run())
        }
        Ok(output)
            if String::from_utf8_lossy(&output.stderr).eq("sudo: a password is required\n") =>
        {
            Err("Cannot run non-interactive sudo".to_string())
        }
        Ok(_output) => Err("\"sudo whoami\" didn't return \"root\"".to_string()),
        Err(e) => Err(format!("{}: {}", UUTILS_WARNING, e)),
    }
}

// in my macine, /sys/class/rtc/rtc0 is the RTC interface provided by kernel
fn get_raw_hwclock_time_from_sys() -> NaiveDateTime {
    let time_raw = format!(
        "{} {}",
        read_to_string("/sys/class/rtc/rtc0/date")
            .unwrap()
            .strip_suffix("\n")
            .unwrap(),
        read_to_string("/sys/class/rtc/rtc0/time")
            .unwrap()
            .strip_suffix("\n")
            .unwrap()
    );
    NaiveDateTime::parse_from_str(&time_raw, "%Y-%m-%d %H:%M:%S").unwrap()
}

fn adjustment_file_utc() -> bool {
    !read_to_string("/etc/adjtime").unwrap().contains("LOCAL")
}

fn parse_from_output(output: &str) -> i64 {
    // program output is always local
    Local
        .from_local_datetime(
            &NaiveDateTime::parse_from_str(
                output.strip_suffix("\n").unwrap(),
                "%Y-%m-%d %H:%M:%S%.6f %z",
            )
            .unwrap(),
        )
        .unwrap()
        .timestamp_micros()
}

fn hwclock_time_to_local(is_utc: bool) -> i64 {
    match is_utc {
        true => Local.from_utc_datetime(&get_raw_hwclock_time_from_sys()),
        false => Local
            .from_local_datetime(&get_raw_hwclock_time_from_sys())
            .unwrap(),
    }
    .timestamp_micros()
}

fn parse_local_naive(time: &str) -> i64 {
    Local
        .from_local_datetime(&NaiveDateTime::parse_from_str(time, "%Y-%m-%d %H:%M:%S").unwrap())
        .unwrap()
        .timestamp_micros()
}

fn get_hardware_delay() -> i64 {
    match read_to_string("/sys/class/rtc/rtc0/name") {
        Ok(str) => {
            if str.contains("rtc_cmos") {
                return 500_000;
            } else {
                return 0;
            }
        }
        Err(_) => 500_000,
    }
}

#[test]
fn test_get_hwclock_time() {
    let _lock = MUX.lock();
    let tolerance = 1_000_000; // 500 milliseconds
    let utc = adjustment_file_utc();

    let ts = TestScenario::new(util_name!());
    let dir = ts.fixtures.clone();

    let expect = hwclock_time_to_local(utc);
    let cmd = run_ucmd_as_root_ignore_ci(&ts, &[]).unwrap();
    let actual = parse_from_output(cmd.success().stdout_str());
    assert!(actual >= expect - tolerance && actual <= expect + tolerance);

    // manually specify RTC
    let expect = hwclock_time_to_local(utc);
    let cmd = run_ucmd_as_root_ignore_ci(&ts, &["--rtc", "/dev/rtc0"]).unwrap();
    let actual = parse_from_output(cmd.success().stdout_str());
    assert!(actual >= expect - tolerance && actual <= expect + tolerance);

    // manually specify --show option
    let expect = hwclock_time_to_local(utc);
    let cmd = run_ucmd_as_root_ignore_ci(&ts, &["--show"]).unwrap();
    let actual = parse_from_output(cmd.success().stdout_str());
    assert!(actual >= expect - tolerance && actual <= expect + tolerance);

    // drift-correct test
    let mut expect = hwclock_time_to_local(utc);
    let cmd = run_ucmd_as_root_ignore_ci(
        &ts,
        &[
            "--get",
            "--adjfile",
            &format!("{}/adjtime_drift", dir.as_string()),
        ],
    )
    .unwrap();
    let actual = parse_from_output(cmd.success().stdout_str());
    expect += ((expect / 1_000_000 - 11_0000_0000) as f64 / 86400.0 * 12.3) as i64 * 1_000_000;
    assert!(actual >= expect - tolerance && actual <= expect + tolerance);

    // --localtime
    let expect = hwclock_time_to_local(false);
    let cmd = run_ucmd_as_root_ignore_ci(&ts, &["--show", "--localtime"]).unwrap();
    let actual = parse_from_output(cmd.success().stdout_str());
    assert!(actual >= expect - tolerance && actual <= expect + tolerance);

    // --utc
    let expect = hwclock_time_to_local(true);
    let cmd = run_ucmd_as_root_ignore_ci(&ts, &["--show", "--utc"]).unwrap();
    let actual = parse_from_output(cmd.success().stdout_str());
    assert!(actual >= expect - tolerance && actual <= expect + tolerance);

    // --adjfile with local
    let expect = hwclock_time_to_local(false);
    let cmd = run_ucmd_as_root_ignore_ci(
        &ts,
        &[
            "--show",
            "--adjfile",
            &format!("{}/adjtime_local", dir.as_string()),
        ],
    )
    .unwrap();
    let actual = parse_from_output(cmd.success().stdout_str());
    println!("{}", dir.as_string());
    assert!(actual >= expect - tolerance && actual <= expect + tolerance);

    // --noadjfile, should fail
    run_ucmd_as_root_ignore_ci(&ts, &["--get", "--noadjfile"])
        .unwrap()
        .failure();

    // --noadjfile, but with --localtime or --utc
    let expect = hwclock_time_to_local(false);
    let cmd = run_ucmd_as_root_ignore_ci(&ts, &["--show", "--noadjfile", "--localtime"]).unwrap();
    let actual = parse_from_output(cmd.success().stdout_str());
    assert!(actual >= expect - tolerance && actual <= expect + tolerance);

    let expect = hwclock_time_to_local(true);
    let cmd = run_ucmd_as_root_ignore_ci(&ts, &["--show", "--noadjfile", "--utc"]).unwrap();
    let actual = parse_from_output(cmd.success().stdout_str());
    assert!(actual >= expect - tolerance && actual <= expect + tolerance);
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[test]
fn test_directisa() {
    let _lock = MUX.lock();
    let utc = adjustment_file_utc();
    let tolerance = 1_000_000;
    let delay = get_hardware_delay();
    let timer = std::time::Instant::now();

    let ts = TestScenario::new(util_name!());

    let expect = hwclock_time_to_local(utc);
    let cmd = run_ucmd_as_root_ignore_ci(&ts, &["--directisa"]).unwrap();
    let actual = parse_from_output(cmd.success().stdout_str());
    assert!(actual >= expect - tolerance && actual <= expect + tolerance);

    let start_set = timer.elapsed().as_micros() as i64;
    run_ucmd_as_root_ignore_ci(
        &ts,
        &["--set", "--date", "2020-11-20 10:20:30", "--directisa"],
    )
    .unwrap()
    .success();
    let end_set = timer.elapsed().as_micros() as i64;
    // measure after set
    let hwclock_time = hwclock_time_to_local(utc);
    let expect = parse_local_naive("2020-11-20 10:20:30") + end_set - start_set - delay;
    let assert_1 = hwclock_time >= expect - tolerance && hwclock_time <= expect + tolerance;

    run_ucmd_as_root_ignore_ci(
        &ts,
        &["--set", "--date", "1999-11-20 10:20:30", "--directisa"],
    )
    .unwrap()
    .success();
    let hwclock_time = get_raw_hwclock_time_from_sys();
    let expect = parse_local_naive("1999-11-20 10:20:30");
    let expect_2 = parse_local_naive("2099-11-20 10:20:30");
    let clock_time = match utc {
        true => Local.from_utc_datetime(&hwclock_time),
        false => Local.from_local_datetime(&hwclock_time).unwrap(),
    }
    .timestamp_micros();
    let assert_2 = clock_time >= expect - tolerance && clock_time <= expect + tolerance
        || clock_time >= expect_2 - tolerance && clock_time <= expect_2 + tolerance;

    // reset to system time
    let cmd = run_ucmd_as_root_ignore_ci(&ts, &["--systohc"]).unwrap();
    if !cmd.succeeded() {
        panic!(
            "Failed to recover hardware clock time, some services may have an error later,
        but luckily we did recovered system time,
        so you could execute `sudo hwclock --systohc` to fix this minor mistake"
        )
    }
    // verify after recovery
    assert!(assert_1 && assert_2);
}

#[test]
fn test_set_hwclock_time() {
    let _lock = MUX.lock();
    let tolerance = 1_000_000;
    let utc = adjustment_file_utc();
    let delay = get_hardware_delay();

    let ts = TestScenario::new(util_name!());
    let timer = std::time::Instant::now();

    // set to 2001-11-20 10:20:30 LOCAL
    // check if the RTC value has already been set to 2001-11-20 10:20:30-32
    let start_set = timer.elapsed().as_micros() as i64;
    run_ucmd_as_root_ignore_ci(&ts, &["--set", "--date", "2001-11-20 10:20:30"])
        .unwrap()
        .success();
    let hwclock_time = hwclock_time_to_local(utc);
    let end_set = timer.elapsed().as_micros() as i64;
    let expect = parse_local_naive("2001-11-20 10:20:30") + end_set - start_set - delay;
    let assert_1: bool = hwclock_time >= expect - tolerance && hwclock_time <= expect + tolerance;

    // reset to system time
    let cmd = run_ucmd_as_root_ignore_ci(&ts, &["--systohc"]).unwrap();
    if !cmd.succeeded() {
        panic!(
            "Failed to recover hardware clock time, some services may have an error later,
        but luckily we did recovered system time,
        so you could execute `sudo hwclock --systohc` to fix this minor mistake"
        )
    }

    // --delay test, it will delay 3 seconds and then set the system time to RTC
    // after this, RTC will be 3 seconds earlier than system time
    let start_set = timer.elapsed().as_micros() as i64;
    let mut expect = Local::now().timestamp_micros() - 3_000_000;
    run_ucmd_as_root_ignore_ci(&ts, &["--systohc", "--delay", "3"])
        .unwrap()
        .success();
    let end_set = timer.elapsed().as_micros() as i64;
    expect += end_set - start_set;
    let hwclock_time = hwclock_time_to_local(utc);

    // check if the program delayed 3 seconds and set hardware clock
    let assert_2 = hwclock_time >= expect - tolerance && hwclock_time <= expect + tolerance;

    // reset to system time
    let cmd = run_ucmd_as_root_ignore_ci(&ts, &["--systohc"]).unwrap();
    if !cmd.succeeded() {
        panic!(
            "Failed to recover hardware clock time, some services may have an error later,
        but luckily we did recovered system time,
        so you could execute `sudo hwclock --systohc` to fix this minor mistake"
        )
    }
    assert!(assert_1 && assert_2);
}

#[test]
fn test_systohc() {
    let _lock = MUX.lock();
    let tolerance = 1_000_000;
    let utc = adjustment_file_utc();
    let delay = get_hardware_delay();
    let timer = std::time::Instant::now();

    let ts = TestScenario::new(util_name!());

    run_ucmd_as_root_ignore_ci(&ts, &["--set", "--date", "2001-11-20 10:20:30"])
        .unwrap()
        .success();

    let start_set = timer.elapsed().as_micros() as i64;
    let mut expect = Local::now().timestamp_micros();
    run_ucmd_as_root_ignore_ci(&ts, &["--systohc"])
        .unwrap()
        .success();
    let end_set = timer.elapsed().as_micros() as i64;
    expect += end_set - start_set - delay;
    let hwclock_time = hwclock_time_to_local(utc);
    let inf = expect - tolerance;
    let sup = expect + tolerance;

    assert!(hwclock_time >= inf && hwclock_time <= sup);
}

#[test]
fn test_set_system_time() {
    let _lock = MUX.lock();
    let tolerance = 1_000_000;
    let _utc = adjustment_file_utc();
    let delay = get_hardware_delay();

    let ts = TestScenario::new(util_name!());

    let timer = std::time::Instant::now();
    let start_backup = Local::now().timestamp_micros();

    let start_set = timer.elapsed().as_micros() as i64;
    run_ucmd_as_root_ignore_ci(&ts, &["--set", "--date", "2001-11-20 10:20:30"])
        .unwrap()
        .success();
    let end_set = timer.elapsed().as_micros() as i64;

    run_ucmd_as_root_ignore_ci(&ts, &["--hctosys"])
        .unwrap()
        .success();
    let systime_now = Local::now().timestamp_micros();
    let expect = parse_local_naive("2001-11-20 10:20:30") + end_set - start_set - delay;
    // After hardware clock set to 2001-10-20 10:20:30,
    // when we execute --hctosys, it will wait until 10:20:31 and then set systime to 10:20:31
    let inf = expect - tolerance;
    let sup = expect + tolerance;

    let assert_1 = systime_now >= inf && systime_now <= sup;

    let recover = start_backup + timer.elapsed().as_micros() as i64;
    let recover_local = Local.timestamp_micros(recover).unwrap();

    if !run_ucmd_as_root_ignore_ci(&ts, &["--set", "--date", &recover_local.to_string()])
        .unwrap()
        .succeeded()
    {
        panic!(
            "Failed to recover hardware clock time, some services may have an error later,
        but luckily we did recovered system time,
        so you could execute `sudo hwclock --systohc` to fix this minor mistake"
        )
    }
    let recover_rc = run_ucmd_as_root_ignore_ci(&ts, &["--hctosys"]).unwrap();
    if !recover_rc.succeeded() {
        panic!(
            "Failed to recover system time, some services may have an error later,
        you may need to set synchronize from an NTP server manually,
        and then execute `sudo hwclock --systohc`"
        );
    }
    assert!(assert_1);
}

#[test]
fn test_arguments_exclusive() {
    let ts = TestScenario::new(util_name!());
    let dir = ts.fixtures.clone();

    run_ucmd_as_root_ignore_ci(&ts, &["--get", "--systohc"])
        .unwrap()
        .failure();

    run_ucmd_as_root_ignore_ci(
        &ts,
        &[
            "--get",
            "--adjfile",
            &format!("{}/adjtime_drfit", dir.as_string()),
            "--noadjfile",
        ],
    )
    .unwrap()
    .failure();

    run_ucmd_as_root_ignore_ci(&ts, &["--get", "--set", "--date", "2001-10-20 10:20:30"])
        .unwrap()
        .failure();

    run_ucmd_as_root_ignore_ci(&ts, &["--systohc", "--hctosys", "--verbose"])
        .unwrap()
        .failure();
}

#[test]
fn test_arguments_invalid() {
    let _lock = MUX.lock();

    let ts = TestScenario::new(util_name!());
    let dir = ts.fixtures.clone();

    run_ucmd_as_root_ignore_ci(&ts, &["--set", "--date", "2001-10-20 25:20:30"])
        .unwrap()
        .failure();

    run_ucmd_as_root_ignore_ci(&ts, &["--set"])
        .unwrap()
        .failure();

    run_ucmd_as_root_ignore_ci(&ts, &["--param-get", "invalid-param-name", "--test"])
        .unwrap()
        .failure();

    // invalid RTC device file test
    run_ucmd_as_root_ignore_ci(
        &ts,
        &[
            "--show",
            "--rtc",
            &format!("{}/invalid_rtc", dir.as_string()),
        ],
    )
    .unwrap()
    .failure();

    // invalid adjustment file test
    run_ucmd_as_root_ignore_ci(
        &ts,
        &[
            "--get",
            "--adjfile",
            &format!("{}/invalid_adjtime", dir.as_string()),
        ],
    )
    .unwrap()
    .failure();
}

#[test]
fn test_rtc_param() {
    let _lock = MUX.lock();

    let ts = TestScenario::new(util_name!());

    // get parameter, because we don't know what the parameters are
    run_ucmd_as_root_ignore_ci(&ts, &["--param-get", "features"]).unwrap();

    // RTC parameter set will not be tested, this may cause your test machine problems
}

#[test]
fn test_voltage_low() {
    let _lock = MUX.lock();

    let ts = TestScenario::new(util_name!());

    // on some machine, run this option will cause an error
    // some linux distributions have already removed this function in hwclock of util-linux
    run_ucmd_as_root_ignore_ci(&ts, &["--vl-read"]).unwrap();
}

#[test]
fn test_hwclock_predict() {
    let _lock = MUX.lock();

    let ts = TestScenario::new(util_name!());
    let dir = ts.fixtures.clone();

    // --predict has nothing to do with the current time,
    // program running time and RTC hardware.
    // It is only related to adjfile and time options, so you can use stdout_only directly.
    run_ucmd_as_root_ignore_ci(
        &ts,
        &[
            "--predict",
            "--date",
            "2001-10-10 10:20:30",
            "--adjfile",
            &format!("{}/adjtime_drift", dir.as_string()),
        ],
    )
    .unwrap()
    .success()
    .stdout_only("2001-10-10 14:11:24.522118 +08:00\n");

    run_ucmd_as_root_ignore_ci(
        &ts,
        &[
            "--predict",
            "--date",
            "2018-10-10 10:20:30",
            "--adjfile",
            &format!("{}/adjtime_drift", dir.as_string()),
        ],
    )
    .unwrap()
    .success()
    .stdout_only("2018-10-09 16:58:33.822119 +08:00\n");
}

#[test]
fn test_update_drift() {
    let _lock = MUX.lock();

    let ts = TestScenario::new(util_name!());
    let dir = ts.fixtures.clone();

    run_ucmd_as_root_ignore_ci(
        &ts,
        &[
            "--set",
            "--date",
            "2020-11-23 10:20:30",
            "--update-drift",
            "--adjfile",
            &format!("{}/adjtime_to_update", dir.as_string()),
        ],
    )
    .unwrap()
    .success();
    let raw = dir.read("adjtime_to_update");
    // if the time we set is close and earlier, after execute, new drift factor will be negative
    // if the time we set is far from now, new drift factor will be very large, and it will be set to zero
    let drift = raw
        .split(' ')
        .into_iter()
        .next()
        .map(|drift| drift.parse::<f64>().unwrap())
        .unwrap();
    run_ucmd_as_root_ignore_ci(&ts, &["--systohc"])
        .unwrap()
        .success();
    println!("{}", drift);
    assert!(drift <= 0.0)
}

#[test]
fn test_testing_mode() {
    let _lock = MUX.lock();
    // use option --test to enter testing mode
    // in testing mode, nothing will be changed
    let test_tolerance = 1_000_000;

    let ts = TestScenario::new(util_name!());
    let dir = ts.fixtures.clone();

    // --set and --systohc use the same function, only test one of them
    let before_test = get_raw_hwclock_time_from_sys().and_utc().timestamp_micros();
    run_ucmd_as_root_ignore_ci(&ts, &["--set", "--date", "2001-11-20 10:20:30", "--test"])
        .unwrap()
        .success();
    let end_test = get_raw_hwclock_time_from_sys().and_utc().timestamp_micros();
    let inf = before_test - test_tolerance;
    let sup = before_test + test_tolerance;
    assert!(end_test >= inf && end_test <= sup);

    // --update-drift --test
    run_ucmd_as_root_ignore_ci(
        &ts,
        &[
            "--set",
            "--date",
            "2020-11-23 10:20:30",
            "--update-drift",
            "--adjfile",
            &format!("{}/adjtime_to_update", dir.as_string()),
            "--test",
        ],
    )
    .unwrap()
    .success();
    let raw = dir.read("adjtime_to_update");
    // the time we set is earlier than now, after execute, new drift factor will be negative
    let _ = raw
        .split(' ')
        .into_iter()
        .next()
        .map(|drift| assert_eq!(drift, "12.234000"));

    // update adjustment file test
    run_ucmd_as_root_ignore_ci(
        &ts,
        &[
            "--set",
            "--date",
            "2020-11-23 10:20:30",
            "--update-drift",
            "--adjfile",
            &format!("{}/adjtime_to_update", dir.as_string()),
            "--test",
        ],
    )
    .unwrap()
    .success();
    let raw = dir.read("adjtime_to_update");
    // after this, second value of adjustment file will still be 1100000000
    let mut it = raw.split(' ').into_iter();
    it.next();
    assert_eq!(it.next(), Some("1100000000"));

    run_ucmd_as_root_ignore_ci(
        &ts,
        &[
            "--adjust",
            "--adjfile",
            &format!("{}/adjtime_to_update", dir.as_string()),
            "--test",
        ],
    )
    .unwrap()
    .success();
    let raw = dir.read("adjtime_to_update");
    // after this, second value of adjustment file will still be 1100000000
    let mut it = raw.split(' ').into_iter();
    it.next();
    assert_eq!(it.next(), Some("1100000000"));
}

#[test]
fn test_systz() {
    let _lock = MUX.lock();

    let ts = TestScenario::new(util_name!());
    // Normally, the --systz function is used to set the timezone of the Linux kernel,
    // but some time-sensitive programs do not obtain the timezone configuration from the Linux kernel.
    // They usually read /etc/localtime or obtain it from a remote server.
    // so, you can use settimeofday(NULL, timezone) to set a time zone.
    // Generally, there will be no major problems.
    run_ucmd_as_root_ignore_ci(&ts, &["--systz"])
        .unwrap()
        .success();
}

#[test]
fn test_adjust() {
    let _lock = MUX.lock();
    let tolerance = 2_000_000;
    let ts = TestScenario::new(util_name!());
    let dir = ts.fixtures.clone();

    let hwclock_time = hwclock_time_to_local(adjustment_file_utc());

    run_ucmd_as_root_ignore_ci(
        &ts,
        &[
            "--adjust",
            "--adjfile",
            &format!("{}/adjtime_to_update", dir.as_string()),
        ],
    )
    .unwrap()
    .success();
    let after_adjust = hwclock_time_to_local(adjustment_file_utc());
    let raw = dir.read("adjtime_to_update");
    let expect = ((hwclock_time / 1_000_000 - 11_0000_0000) as f64 / 86400.0 * 12.234) as i64
        * 1_000_000
        + hwclock_time;
    let mut it = raw.split(' ').into_iter();
    it.next();
    assert_ne!(it.next(), Some("1100000000"));
    assert!(after_adjust >= expect - tolerance && after_adjust <= expect + tolerance);
}

#[test]
fn test_multiple_date_formats() {
    let _lock = MUX.lock();

    let ts = TestScenario::new(util_name!());

    let tests = &[
        "Wed, 02 Jun 2021 06:31:39 GMT",
        "1996-12-19T16:39:57-08:00",
        "2014-04-26 13:13:43 +0800",
        "Tue, 1 Jul 2003 10:52:37 +0200",
        "10:23",
        "10:23:23",
        "2023-10-23",
        "2023/10/20 10:23:23",
        "10:23 PM",
        "10:23 AM",
        "03:23:23 CST",
        "2015-09-30 18:48:56 UTC",
        "September 17, 2012 10:09am",
        "2021-Feb-21",
        "171113 14:14:20",
    ];

    for &test_date in tests {
        run_ucmd_as_root_ignore_ci(&ts, &["--predict", "--date", test_date])
            .unwrap()
            .success();
    }
}

#[test]
fn test_version() {
    new_ucmd!().arg("--version").succeeds();

    new_ucmd!().arg("-V").succeeds();
}

#[test]
fn test_help() {
    new_ucmd!().arg("--help").succeeds();

    new_ucmd!().arg("-h").succeeds();
}

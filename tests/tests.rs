#[macro_use]
mod common;

#[cfg(feature = "arp")]
#[path = "by-util/test_arp.rs"]
mod test_arp;

#[cfg(feature = "attr")]
#[path = "by-util/test_attr.rs"]
mod test_attr;

#[cfg(feature = "base32")]
#[path = "by-util/test_base32.rs"]
mod test_base32;

#[cfg(feature = "chage")]
#[path = "by-util/test_chage.rs"]
mod test_chage;

#[cfg(feature = "file")]
#[path = "by-util/test_file.rs"]
mod test_file;

#[cfg(feature = "flock")]
#[path = "by-util/test_flock.rs"]
mod test_flock;

#[cfg(feature = "hwclock")]
#[path = "by-util/test_hwclock.rs"]
mod test_hwclock;

#[cfg(feature = "lspci")]
#[path = "by-util/test_lspci.rs"]
mod test_lspci;

#[cfg(feature = "login")]
#[path = "by-util/test_login.rs"]
mod test_login;
#[cfg(feature = "mount")]
#[path = "by-util/test_mount.rs"]
mod test_mount;

#[cfg(feature = "pidof")]
#[path = "by-util/test_pidof.rs"]
mod test_pidof;

#[cfg(feature = "pgrep")]
#[path = "by-util/test_pgrep.rs"]
mod test_pgrep;

#[cfg(feature = "pstree")]
#[path = "by-util/test_pstree.rs"]
mod test_pstree;

#[cfg(feature = "sysctl")]
#[path = "by-util/test_sysctl.rs"]
mod test_sysctl;

#[cfg(feature = "taskset")]
#[path = "by-util/test_taskset.rs"]
mod test_taskset;

#[cfg(feature = "setsid")]
#[path = "by-util/test_setsid.rs"]
mod test_setsid;

#[cfg(feature = "umount")]
#[path = "by-util/test_umount.rs"]
mod test_umount;

#[cfg(feature = "usleep")]
#[path = "by-util/test_usleep.rs"]
mod test_usleep;

#[cfg(feature = "which")]
#[path = "by-util/test_which.rs"]
mod test_which;

#[cfg(feature = "xargs")]
#[path = "by-util/test_xargs.rs"]
mod test_xargs;

#[cfg(feature = "free")]
#[path = "by-util/test_free.rs"]
mod test_free;

#[cfg(feature = "column")]
#[path = "by-util/test_column.rs"]
mod test_column;

#[cfg(feature = "sha256sum")]
#[path = "by-util/test_sha256sum.rs"]
mod test_sha256sum;

#[cfg(feature = "grep")]
#[path = "by-util/test_grep.rs"]
mod test_grep;

#[cfg(feature = "iostat")]
#[path = "by-util/test_iostat.rs"]
mod test_iostat;

#[cfg(feature = "killall")]
#[path = "by-util/test_killall.rs"]
mod test_killall;

#[cfg(feature = "md5sum")]
#[path = "by-util/test_md5sum.rs"]
mod test_md5sum;

#[cfg(feature = "groupadd")]
#[path = "by-util/test_groupadd.rs"]
mod test_groupadd;

#[cfg(feature = "usermod")]
#[path = "by-util/test_usermod.rs"]
mod test_usermod;

#[cfg(feature = "useradd")]
#[path = "by-util/test_useradd.rs"]
mod test_useradd;

#[cfg(feature = "find")]
#[path = "by-util/test_find.rs"]
mod test_find;

#[cfg(feature = "less")]
#[path = "by-util/test_less.rs"]
mod test_less;

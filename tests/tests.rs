#[macro_use]
mod common;

#[cfg(feature = "attr")]
#[path = "by-util/test_attr.rs"]
mod test_attr;

#[cfg(feature = "base32")]
#[path = "by-util/test_base32.rs"]
mod test_base32;

#[cfg(feature = "flock")]
#[path = "by-util/test_flock.rs"]
mod test_flock;

#[cfg(feature = "hwclock")]
#[path = "by-util/test_hwclock.rs"]
mod test_hwclock;

#[cfg(feature = "lspci")]
#[path = "by-util/test_lspci.rs"]
mod test_lspci;

#[cfg(feature = "pidof")]
#[path = "by-util/test_pidof.rs"]
mod test_pidof;

#[cfg(feature = "pstree")]
#[path = "by-util/test_pstree.rs"]
mod test_pstree;

#[cfg(feature = "taskset")]
#[path = "by-util/test_taskset.rs"]
mod test_taskset;

#[cfg(feature = "setsid")]
#[path = "by-util/test_setsid.rs"]
mod test_setsid;

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

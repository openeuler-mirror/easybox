#[macro_use]
mod common;

#[cfg(feature = "base32")]
#[path = "by-util/test_base32.rs"]
mod test_base32;

#[cfg(feature = "flock")]
#[path = "by-util/test_flock.rs"]
mod test_flock;

#[cfg(feature = "hwclock")]
#[path = "by-util/test_hwclock.rs"]
mod test_hwclock;

#[cfg(feature = "pstree")]
#[path = "by-util/test_pstree.rs"]
mod test_pstree;
#[cfg(feature = "taskset")]
#[path = "by-util/test_taskset.rs"]
mod test_taskset;

//! This file is part of the easybox package.
//
// (c) Wentong Yang <ywt0821@163.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use crate::groupadd_common::Config;
use std::path::Path;
use uucore::error::{UResult, USimpleError};

/// invalid argument to option
const E_BAD_ARG: i32 = 3;

/// similar to C code process_prefix_flag
pub fn process_prefix_flag(config: &mut Config) -> UResult<Option<String>> {
    if let Some(ref mut prefix) = config.prefix_dir {
        if nix::unistd::setgid(nix::unistd::getgid()).is_err()
            || nix::unistd::setuid(nix::unistd::getuid()).is_err()
        {
            return Err(USimpleError::new(E_BAD_ARG, "failed to drop privileges").into());
        }

        if prefix.is_empty() || prefix == "/" {
            return Ok(None);
        }

        if !prefix.starts_with('/') {
            return Err(USimpleError::new(E_BAD_ARG, "prefix must be an absolute path").into());
        }

        if prefix.ends_with('/') {
            prefix.pop();
        }

        let path = Path::new(prefix);
        if !path.exists() {
            return Err(USimpleError::new(
                1,
                format!("Prefix directory '{}' does not exist", prefix),
            )
            .into());
        } else if !path.is_dir() {
            return Err(
                USimpleError::new(1, format!("Prefix '{}' is not a directory", prefix)).into(),
            );
        }
        return Ok(Some(prefix.clone()));
    }

    Ok(None)
}

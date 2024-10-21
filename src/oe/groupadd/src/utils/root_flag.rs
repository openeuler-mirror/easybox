//! This file is part of the easybox package.
//
// (c) Wentong Yang <ywt0821@163.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.
use super::exitcodes::EXIT_FAILURE;
use crate::utils::exitcodes::E_BAD_ARG;
use nix::unistd::{chdir, chroot, getgid, getuid, setresgid, setresuid};
use std::path::Path;
use uucore::error::{UResult, USimpleError};

///
pub fn process_root_flag(short_opt: &str, arg_list: Vec<String>) -> UResult<()> {
    let mut newroot: Option<String> = None;
    let mut args_iter = arg_list.iter().peekable();

    while let Some(arg) = args_iter.next() {
        let val: Option<&str> = if arg == "--root" || arg == short_opt {
            match args_iter.peek() {
                Some(next_arg) => Some(next_arg.as_str()),
                None => {
                    return Err(USimpleError::new(
                        E_BAD_ARG,
                        format!("option '{}' requires an argument", arg),
                    ));
                }
            }
        } else if arg.starts_with("--root=") {
            Some(&arg[7..])
        } else {
            None
        };

        if let Some(val) = val {
            if newroot.is_some() {
                return Err(USimpleError::new(
                    E_BAD_ARG,
                    "multiple --root options provided".to_string(),
                ));
            }
            newroot = Some(val.to_string());

            if arg == "--root" || arg == short_opt {
                args_iter.next();
            }
        }
    }

    if let Some(newroot) = newroot {
        change_root(&newroot)?;
    }

    Ok(())
}

///
pub fn change_root(newroot: &str) -> UResult<()> {
    if let Err(err) = setresgid(getgid(), getgid(), getgid())
        .and_then(|_| setresuid(getuid(), getuid(), getuid()))
    {
        return Err(USimpleError::new(
            EXIT_FAILURE,
            format!("unable to drop privileges before chroot: {}", err),
        ));
    }

    if !newroot.starts_with('/') {
        return Err(USimpleError::new(
            E_BAD_ARG,
            format!(
                "invalid chroot path '{}', only absolute paths are supported.",
                newroot
            ),
        ));
    }

    if !Path::new(newroot).exists() {
        return Err(USimpleError::new(
            E_BAD_ARG,
            format!("cannot access chroot directory {}", newroot),
        ));
    }

    if let Err(err) = chroot(Path::new(newroot)) {
        return Err(USimpleError::new(
            E_BAD_ARG,
            format!("unable to chroot to directory {}: {}", newroot, err),
        ));
    }

    if let Err(err) = chdir("/") {
        return Err(USimpleError::new(
            E_BAD_ARG,
            format!("cannot chdir in chroot directory {}: {}", newroot, err),
        ));
    }

    Ok(())
}

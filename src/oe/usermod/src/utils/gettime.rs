//! This file is part of the easybox package.
//
// (c) Wentong Yang <ywt0821@163.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.
use std::{
    env,
    io::{self, Write},
    time::{SystemTime, UNIX_EPOCH},
};

use uucore::error::{UResult, USimpleError};
///
pub const DAY: i64 = 24 * 3600;

///
pub fn gettime() -> UResult<i64> {
    let mut shadow_logfd = io::stderr();

    let fallback = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| USimpleError::new(1, "Time went backwards"))?
        .as_secs() as i64;

    if let Ok(source_date_epoch) = env::var("SOURCE_DATE_EPOCH") {
        match source_date_epoch.parse::<i64>() {
            Ok(epoch) => {
                if epoch > fallback {
                    writeln!(shadow_logfd,
                             "Environment variable $SOURCE_DATE_EPOCH: value must be smaller than or equal to the current time ({}) but was found to be: {}",
                             fallback, epoch).unwrap();
                } else {
                    return Ok(epoch);
                }
            }
            Err(e) => {
                writeln!(
                    shadow_logfd,
                    "Environment variable $SOURCE_DATE_EPOCH: failed to parse: {}",
                    e
                )
                .unwrap();
            }
        }
    }

    Ok(fallback)
}

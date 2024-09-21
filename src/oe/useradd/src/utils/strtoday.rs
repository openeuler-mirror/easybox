//! This file is part of the easybox package.
//
// (c) Wentong Yang <ywt0821@163.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

use chrono::NaiveDate;
use std::str::FromStr;
use uucore::error::{UResult, USimpleError};

///
pub fn strtoday(date_str: &str) -> UResult<i64> {
    if date_str.is_empty() {
        return Err(USimpleError::new(1, "Invalid input: empty string"));
    }

    if date_str.chars().all(|c| c.is_digit(10)) {
        return match i64::from_str(date_str) {
            Ok(days) => Ok(days),
            Err(_) => Err(USimpleError::new(1, "Invalid input: cannot parse number")),
        };
    }

    match NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .or_else(|_| NaiveDate::parse_from_str(date_str, "%y-%m-%d"))
        .or_else(|_| NaiveDate::parse_from_str(date_str, "%m/%d/%y"))
        .or_else(|_| NaiveDate::parse_from_str(date_str, "%d %B %Y"))
        .or_else(|_| NaiveDate::parse_from_str(date_str, "%d %b %y"))
        .or_else(|_| NaiveDate::parse_from_str(date_str, "%b %d, %Y"))
        .or_else(|_| NaiveDate::parse_from_str(date_str, "%d-%b-%y"))
        .or_else(|_| NaiveDate::parse_from_str(date_str, "%d%b%y"))
    {
        Ok(date) => {
            let epoch = NaiveDate::from_ymd_opt(1970, 1, 1)
                .ok_or_else(|| USimpleError::new(1, "Failed to create epoch date"))?;
            Ok(date.signed_duration_since(epoch).num_days())
        }
        Err(_) => Err(USimpleError::new(1, "Invalid input: cannot parse date")),
    }
}

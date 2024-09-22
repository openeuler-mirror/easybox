//! This file is part of the easybox package.
//
// (c) Wentong Yang <ywt0821@163.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.
///
#[derive(Clone)]
pub struct Sgrp {
    ///
    pub sg_name: String,
    ///
    pub sg_passwd: String,
    ///
    pub sg_adm: Vec<String>,
    ///
    pub sg_mem: Vec<String>,
}

///
impl Sgrp {
    ///
    pub fn new() -> Sgrp {
        Sgrp {
            sg_name: String::new(),
            sg_passwd: String::new(),
            sg_adm: Vec::new(),
            sg_mem: Vec::new(),
        }
    }
}

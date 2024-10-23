//! This file is part of the easybox package.
//
// (c) Wentong Yang <ywt0821@163.com>
//
// For the full copyright and license information, please view the LICENSE file
// that was distributed with this source code.

///
pub fn del_list(members: &mut Vec<String>, user: &str) {
    members.retain(|m| m != user);
}

///
pub fn add_list(members: &mut Vec<String>, user: &str) {
    if !members.contains(&user.to_string()) {
        members.push(user.to_string());
    }
}

///
pub fn is_on_list(list: &Vec<String>, item: &str) -> bool {
    list.contains(&item.to_string())
}

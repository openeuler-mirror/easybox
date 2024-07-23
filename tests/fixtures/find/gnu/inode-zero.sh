#!/bin/bash
# This file is part of the easybox package.
#
# (c) Xing Huang <navihx@foxmail.com>
#
# For the full copyright and license information, please view the LICENSE file
# that was distributed with this source code.

# Ensure find treats inode number 0 correctly.

source "init.sh"

f='/dev/console'
test -e "${f}" \
    && ino=$( stat -c '%i' "${f}" ) \
    && test "${ino}" = '0' \
    || skip_

echo "${f}" > exp || framework_fail_

$FIND "${f}" -inum 0 >out || fail=1
compare exp out || fail=1

$FIND "${f}" -inum 12345 || fail=1
compare /dev/null || fail=1

echo 0 > exp || framework_fail_
$FIND "${f}" -printf '%i\n' >out || fail=1
compare exp out || fail=1

exit $fail

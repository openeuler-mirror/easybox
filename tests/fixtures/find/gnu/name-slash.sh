#!/bin/bash
# This file is part of the easybox package.
#
# (c) Xing Huang <navihx@foxmail.com>
#
# For the full copyright and license information, please view the LICENSE file
# that was distributed with this source code.

# Exercise 'find -name PATTERN' behavior with a '/' in PATTERN.

source "init.sh"

# Exercises '-name PATTERN' with a '/' somewhere is PATTERN.
# Test fail if find runs.
$FIND -name 'dir/file' > out || fail=1
compare /dev/null out || fail=1

# Exercises '-name /'
echo '/' > exp || framework_fail_
$FIND / -maxdepth 0 -name '/' > out 2> err || fail=1
compare exp out || fail=1
compare /dev/null err || fail=1

exit $fail

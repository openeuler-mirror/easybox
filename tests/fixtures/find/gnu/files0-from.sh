#!/bin/bash
# This file is part of the easybox package.
#
# (c) Xing Huang <navihx@foxmail.com>
#
# For the full copyright and license information, please view the LICENSE file
# that was distributed with this source code.

# Exercise -files0-from option.

source "init.sh"

$FIND -files0-from > out && fail=1

$FIND STARTING_POINT -files0-from FILE > out && fail=1

echo "." > exp || framework_fail_
$FIND -maxdepth 0 > out || fail=1
compare exp out || fail=1

# -files0-from with argument "-" must not be combined with the -ok action.
$FIND -files0-from - -okdir echo '{}' ';' < /dev/null && fail=1

# A non-existing file.
$FIND -files0-from NOTHING && fail=1

# An empty input file
$FIND -files0-from /dev/null > out || fail=1
compare /dev/null out || fail=1

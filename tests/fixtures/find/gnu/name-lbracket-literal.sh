#!/bin/bash
# This file is part of the easybox package.
#
# (c) Xing Huang <navihx@foxmail.com>
#
# For the full copyright and license information, please view the LICENSE file
# that was distributed with this source code.

# Test that find -name treats the literal'[' argument .

source "init.sh"

touch '[' || framework_fail_
echo './[' > exp || framework_fail_

$FIND -name '[[]' -print > out || fail=1
compare exp out || fail=1

exit $fail

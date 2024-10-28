#!/bin/bash
# This file is part of the easybox package.
#
# (c) Xing Huang <navihx@foxmail.com>
#
# For the full copyright and license information, please view the LICENSE file
# that was distributed with this source code.

# find -depth: ensure to output an unreadable directory.

source "init.sh"

# Root can descend into any directory, skip.
if [ "$(id -u)" -eq 0 ]; then
    skip_
fi

# Prepare an unreadable directory
mkdir tmp tmp/dir \
    && chmod 0311 tmp/dir \
    && echo 'tmp/dir' > exp \
    || framework_fail_

$FIND tmp -depth -name dir > out || fail=1
compare exp out || fail=1

exit $fail

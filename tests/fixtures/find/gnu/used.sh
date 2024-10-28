#!/bin/bash
# This file is part of the easybox package.
#
# (c) Xing Huang <navihx@foxmail.com>
#
# For the full copyright and license information, please view the LICENSE file
# that was distributed with this source code.

# Verify that find -used works.

source "init.sh"

for d in 10 20 30 40; do
    touch -a -d "$(date -d "$d day" '+%Y-%m-%d %H:%M:%S')" t$d || fail=1
done

touch t00 || setup_fail_

for d in -45 -35 -25 -15 -5 0 5 15 25 35 45 +0 +5 +15 +25 +35 +45; do
    echo "== testing: find -used $d"
    $FIND . -type f -name 't*' -used $d > out || fail=1
    find . -type f -name 't*' -used $d > exp || framework_fail_
    sort out > out2 || framework_fail_
    sort exp > exp2 || framework_fail_
    compare out2 exp2 || fail=1
done > out

exit $fail

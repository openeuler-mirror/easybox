#!/bin/bash
# This file is part of the easybox package.
#
# (c) Xing Huang <navihx@foxmail.com>
#
# For the full copyright and license information, please view the LICENSE file
# that was distributed with this source code.

# Exercise -anewer -cnewer -newer -newerXY.

source "init.sh"

# Tests in test_killall may kill sleep in this script.
# So let we use new _snap function to take a break.
_snap () {
    start=$(date +%s)
    while [ $(($(date +%s) - start)) -lt 2 ]; do
        :
    done
}

touch file1 \
    && _snap \
    && touch file2 \
    && _snap \
    && touch file3 \
    || framework_fail_

echo "./file3" > exp || framework_fail_

for x in \
    -anewer -cnewer -newer \
    -neweraa -newerac -neweram \
    -newerca -newercc -newercm \
    -newerma -newermc -newermm \
    ; do
    rm -f out || framework_fail_
    $FIND . $x file2 -name 'file*' > out || fail=1
    compare exp out || fail=1
done

tref="$( stat -c '%y' file2 | date -d \"$(cat)\" '+%Y-%m-%d %H:%M:%S %:::z' )" || tref=''

# Skip this part if stat failed
if test "${tref}"; then
    for x in -newerat -newerct -newermt; do
        rm -f out || framework_fail_
        $FIND . $x "${tref}" -name "file*" > out || fail=1
        compare exp out || fail=1
    done
fi

exit $fail

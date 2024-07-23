#!/bin/bash
# This file is part of the easybox package.
#
# (c) Xing Huang <navihx@foxmail.com>
#
# For the full copyright and license information, please view the LICENSE file
# that was distributed with this source code.

# Ensure 'not-a-number' diagnostic for NAN arguments.

echo "before init"
ls -l

source "init.sh"

echo "after init"

# Expect no output.
> exp

for o in used amin cmin mmin atime ctime mtime; do
    echo test $o
    $FIND -$o NaN >out && fail=1
    compare exp out || fail=1
done

exit $fail

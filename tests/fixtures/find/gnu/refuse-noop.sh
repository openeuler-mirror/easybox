#!/bin/bash
# This file is part of the easybox package.
#
# (c) Xing Huang <navihx@foxmail.com>
#
# For the full copyright and license information, please view the LICENSE file
# that was distributed with this source code.

# This test verifies that find refuses the internal -noop, ---noop option.

source "init.sh"

for opt in 'noop' '--noop'; do
    rm -f out || framwork_fail_
    $FIND "-${opt}" > out && fail=1
    compare /dev/null out || fail=1
done

exit $fail

#!/bin/bash
# This file is part of the easybox package.
#
# (c) Xing Huang <navihx@foxmail.com>
#
# For the full copyright and license information, please view the LICENSE file
# that was distributed with this source code.

# Test -printf with the \c escape character.

source "init.sh"

echo 'hello^.^world' > exp || framework_fail_

$FIND . -maxdepth 0 \
    -printf 'hello^\cthere' \
    -exec printf %s {} \; \
    -printf '^world\n' \
    > out || fail=1

compare exp out || fail=1

exit $fail

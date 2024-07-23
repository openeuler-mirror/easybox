#!/bin/bash
# This file is part of the easybox package.
#
# (c) Xing Huang <navihx@foxmail.com>
#
# For the full copyright and license information, please view the LICENSE file
# that was distributed with this source code.

# Test -printf with octal and letter escapes.

source "init.sh"

echo test | od -c >/dev/null \
    || skip_

cat <<EOF > exp || framework_fail_
0000000 000 001 002 003 004 005 006 007 007 015 014 011 013 010 134 172
0000020 134
0000021
EOF

$FIND . -maxdepth 0 \
  -printf '\0\1\2\3\4\5\6\7\a\r\f\t\v\b\z\\' \
  > out 2> err || fail=1

od -t o1 < out > out2 || framework_fail_
compare exp out2 || fail=1

exit $fail

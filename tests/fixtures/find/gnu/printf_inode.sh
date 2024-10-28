#!/bin/bash
# This file is part of the easybox package.
#
# (c) Xing Huang <navihx@foxmail.com>
#
# For the full copyright and license information, please view the LICENSE file
# that was distributed with this source code.

# Verify that ls -i and find -printf %i produce the same output.

source "init.sh"

make_canonical() {
  sed -e '
    # Solaris ls outputs with leading padding blanks; strip them.
    s/^ *//g;
    # Squeeze blanks between inode number and name to one underscore.
    s/ /_/g'
}

# Create a file.
> file || framework_fail_

# Let ls(1) create the expected output.
ls -i file | make_canonical > exp || framework_fail_

rm -f out out2
$FIND file -printf '%i_%p\n' > out || fail=1
make_canonical < out > out2 || framework_fail_
compare exp out2 || fail=1

exit $fail

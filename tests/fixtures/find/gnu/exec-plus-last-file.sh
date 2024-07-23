#!/bin/bash
# This file is part of the easybox package.
#
# (c) Xing Huang <navihx@foxmail.com>
#
# For the full copyright and license information, please view the LICENSE file
# that was distributed with this source code.

# This test verifies that find invokes the given command for the
# multiple-argument syntax '-exec CMD {} +'.

source "init.sh"

# Require seq for this test
seq 2 >/dev/null 2>&1 || skip_

DIR='dir'
CMD='println'
COUNT='4000'

mkdir 'bin' \
    && printf '%s\n' '#!/bin/sh' 'printf "%s\n" "$@"' > "bin/$CMD" \
    && chmod +x "bin/$CMD" \
    && PATH="$PWD/bin:$PATH" \
    || framework_fail_

seq -f "${DIR}/%04g-file" $COUNT > exp \
  || framework_fail_

mkdir "$DIR" \
  && xargs touch < exp \
  || framework_fail_


[ $COUNT = $($FIND "$DIR" -type f -exec "$CMD" '{}' + | wc -l) ] || fail=1

exit $fail

#!/bin/bash
# This file is part of the easybox package.
#
# (c) Xing Huang <navihx@foxmail.com>
#
# For the full copyright and license information, please view the LICENSE file
# that was distributed with this source code.

source "init.sh"

mkdir dir
cd dir
touch reg
mkdir dir
ln -s reg reg-link
ln -s dir dir-link
ln -s enoent dangling-link

cd ../
find dir -mindepth 1 > all \
    && sort -o all all \
    || skip_

cat <<EOF > exp
=== -type f ===
$(grep -e 'reg$' all)
=== -type l ===
$(grep -e 'link$' all)
=== -xtype l ===
$(grep -e 'dangling-link$' all)
EOF

touch out2

$FIND dir -type f > out || fail=1
sort -o out out
echo "=== -type f ===" >> out2
cat out >> out2

$FIND dir -type l > out || fail=1
sort -o out out
echo "=== -type l ===" >> out2
cat out >> out2

$FIND dir -xtype l > out || fail=1
sort -o out out
echo "=== -xtype l ===" >> out2
cat out >> out2

compare out2 exp || ( diff out2 exp ; fail=1 )

cat out2
cat exp

exit $fail

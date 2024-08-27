#!/bin/sh
cd $1
chown root:root target/debug/easybox
chmod u+s $1/target/debug/easybox
su myuser1 -c "target/debug/easybox chage -l myuser1"
chmod u-s $1/target/debug/easybox

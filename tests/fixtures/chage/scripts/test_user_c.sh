#!/bin/sh
cd $1
su myuser1 -c "/usr/bin/chage -l myuser1"

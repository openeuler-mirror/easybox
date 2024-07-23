#!/bin/bash

skip_ () { exit 0; }
setup_fail_ () { exit 3; }
framework_fail_ () { exit 4; }
compare () { cmp -s $@; }

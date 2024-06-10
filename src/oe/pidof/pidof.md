# pidof

## Usage
```
pidof [options] [program [...]]
```

## About
Find the process ID of a running program.

## Options:
 -s, --single-shot         return one PID only
 -c, --check-root          omit processes with different root
 -q,                       quiet mode, only set the exit code
 -w, --with-workers        show kernel workers too
 -x                        also find shells running the named scripts
 -o, --omit-pid <PID,...>  omit processes with PID
 -S, --separator SEP       use SEP as separator put between PIDs
 -h, --help                display this help and exit
 -t, --lightweight         list threads too
 -V, --version             output version information and exit

# killall

## Usage
```
Usage: killall [ -Z CONTEXT ] [ -u USER ] [ -y TIME ] [ -o TIME ] [ -eIgiqrvw ]
               [ -s SIGNAL | -SIGNAL ] NAME...
       killall -l, --list
       killall -V, --version

  -e,--exact          require exact match for very long names
  -I,--ignore-case    case insensitive process name match
  -g,--process-group  kill process group instead of process
  -y,--younger-than   kill processes younger than TIME
  -o,--older-than     kill processes older than TIME
  -i,--interactive    ask for confirmation before killing
  -l,--list           list all known signal names
  -q,--quiet          don't print complaints
  -r,--regexp         interpret NAME as an extended regular expression
  -s,--signal SIGNAL  send this signal instead of SIGTERM
  -u,--user USER      kill only process(es) running as USER
  -v,--verbose        report if the signal was successfully sent
  -V,--version        display version information
  -w,--wait           wait for processes to die
  -n,--ns PID         match processes that belong to the same namespaces
                      as PID
  -Z,--context REGEXP kill only process(es) having context
                      (must precede other arguments)
```

## About

killall - kill processes by name

## DESCRIPTION
`killall` sends a signal to all processes running any of the specified commands. If no signal name is specified,
SIGTERM is sent.

Signals can be specified either by name (e.g. -HUP or -SIGHUP) or by number (e.g. -1) or by option -s.

If the command name is not regular expression (option -r) and contains a slash (/), processes executing that partic‐
ular file will be selected for killing, independent of their name.

killall returns a zero return code if at least one process has been killed for each listed command, or no commands
were listed and at least one process matched the -u and -Z search criteria. killall returns non-zero otherwise.

A killall process never kills itself (but may kill other killall processes).

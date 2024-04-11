# xargs

## Usage
```bash
xargs [options] [command [initial-arguments]]

xargs -V

```

## About
Build and execute command lines from standard input.

## Arguments
-0, --null                   items are separated by a null, notwhitespace;
                               disables quote and backslashprocessing and
                               logical EOF processing
-a, --arg-file=FILE          read arguments from FILE, not standardinput
-d, --delimiter=CHARACTER    items in input stream are separated byCHARACTER,
                               not by whitespace; disables quoteand backslash
                               processing and logical EOF processing
-E END                       set logical EOF string; if END occursas a line
                               of input, the rest of the input isignored
                               (ignored if -0 or -d was specified)
-e, --eof[=END]              equivalent to -E END if END isspecified;
                               otherwise, there is no end-of-filestring
-I R                         same as --replace=R
-i, --replace[=R]            replace R in INITIAL-ARGS with namesread
                               from standard input, split atnewlines;
                               if R is unspecified, assume {}
-L, --max-lines=MAX-LINES    use at most MAX-LINES non-blank inputlines per
                               command line
-l[MAX-LINES]                similar to -L but defaults to at mostone non-
                               blank input line if MAX-LINES is notspecified
-n, --max-args=MAX-ARGS      use at most MAX-ARGS arguments percommand line
-o, --open-tty               Reopen stdin as /dev/tty in the child process
                                 before executing the command; useful to run an
                                 interactive application.
-P, --max-procs=MAX-PROCS    run at most MAX-PROCS processes at atime
-p, --interactive            prompt before running commands
    --process-slot-var=VAR   set environment variable VAR in childprocesses
-r, --no-run-if-empty        if there are no arguments, then do notrun COMMAND;
                               if this option is not given, COMMANDwill be
                               run at least once
-s, --max-chars=MAX-CHARS    limit length of command line toMAX-CHARS
    --show-limits            show limits on command-line length
-t, --verbose                print commands before executing them
-x, --exit                   exit if the size (see -s) is exceeded
    --help                   display this help and exit
    --version                output version information and exit

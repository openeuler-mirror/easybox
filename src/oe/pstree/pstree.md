# pstree

## Usage
```
pstree [-acglpsStTuZ] [ -h | -H PID ] [ -n | -N type ] [ -A | -G | -U ] [ PID | USER ]
pstree -V
```

## About
Display a tree of processes.

## After Help
PID    start at this PID; default is 1 (init)
USER   show only trees rooted at processes of this user

## Arguments
-a, --arguments              show command line arguments
-A, --ascii                  use ASCII line drawing characters
-c, --compact-not            don't compact identical subtrees
-C, --color <TYPE>           color process by attribute
                                (age)
-g, --show-pgids             show process group ids; implies -c
-G, --vt100                  use VT100 line drawing characters
-h, --highlight-all          highlight current process and its ancestors
-H, --highlight-pid <PID>    highlight this process and its ancestors
    --help                   Print help information
-l, --long                   don't truncate long lines
-n, --numeric-sort           sort output by PID
-N, --ns-sort <TYPE>         sort output by this namespace type
                                (cgroup, ipc, mnt, net, pid, time, user, uts)
-p, --show-pids              show PIDs; implies -c
-s, --show-parents           show parents of the selected process
-S, --ns-changes             show namespace transition
-t, --thread-names           show full thread names
-T, --hide-threads           hide threads, show only process
-u, --uid-changes            show uid transitions
-U, --unicode                use UTF-8 (Unicode) line drawing characters
-V, --version                Print version information
-Z, --security-context       show security attributes

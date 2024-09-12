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

## Description

**pstree** shows running processes as a tree. The tree is rooted at either pid or init if pid is omitted. If a user name is specified, all process trees rooted at processes owned by that user are shown.

**pstree** visually merges identical branches by putting them in square brackets and prefixing them with the repetition count, e.g.

```
init-+-getty
     |-getty
     |-getty
     `-getty
```

becomes

```
init---4*[getty]
```

Child threads of a process are found under the parent process and are shown with the process name in curly braces, e.g.

```
icecast2---13*[{icecast2}]
```

## Options

- **-a, --arguments**

    Show command line arguments. If the command line of a process is swapped out, that process is shown in parentheses. **-a** implicitly disables compaction for processes but not threads.

- **-A, --ascii**

    Use ASCII characters to draw the tree.

- **-c, --compact-not**

    Disable compaction of identical subtrees. By default, subtrees are compacted whenever possible.

- **-C, --color** \<TYPE>

    Color the process name by given attribute. Currently **pstree** only accepts the value **age** which colors by process age. Processes newer than 60 seconds are green, newer than an hour yellow and the remaining red.

- **-g, --show-pgids**

    Show PGIDs. Process Group IDs are shown as decimal numbers in parentheses after each process name. If both PIDs and PGIDs are displayed then PIDs are shown first.

- **-G, --vt100**

    Use VT100 line drawing characters.

- **-h, --highlight-all**

    Highlight the current process and its ancestors. This is a no-op if the terminal doesn't support highlighting or if neither the current process nor any of its ancestors are in the subtree being shown.

- **-H, --highlight-pid** \<PID>

    Like **-h**, but highlight the specified process instead. Unlike with **-h**, **pstree** fails when using **-H** if highlighting is not available.

- **-l, --long**

    Display long lines. By default, lines are truncated to either the COLUMNS environment variable or the display width. If neither of these methods work, the default of 132 columns is used.

- **-n, --numeric-sort**

    Sort processes with the same parent by PID instead of by name. (Numeric sort.)

- **-N, --ns-sort** \<TYPE>

    Show individual trees for each namespace of the type specified. The available types are: ipc, mnt, net, pid, time, user, uts. Regular users don't have access to other users' processes information, so the output will be limited.

- **-p, --show-pids**

    Show PIDs. PIDs are shown as decimal numbers in parentheses after each process name. **-p** implicitly disables compaction.

- **-s, --show-parents**

    Show parent processes of the specified process.

- **-S, --ns-changes**

    Show namespaces transitions. Like **-N**, the output is limited when running as a regular user.

- **-t, --thread-names**

    Show full names for threads when available.

- **-T, --hide-threads**

    Hide threads and only show processes.

- **-u, --uid-changes**

    Show uid transitions. Whenever the uid of a process differs from the uid of its parent, the new uid is shown in parentheses after the process name.

- **-U, --unicode**

    Use UTF-8 (Unicode) line drawing characters. Under Linux 1.1-54 and above, UTF-8 mode is entered on the console with **echo -e ' 33%8'** and left with **echo -e ' 33%@'**.

- **-V, --version**

    Display version information.

- **-Z, --security-context**

    Show the current security attributes of the process. For SELinux systems this will be the security context.

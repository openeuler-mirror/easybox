# pgrep

## Usage
```
pgrep [options] <pattern>
```

## About

look up processes based on name and other attributes

## Description

**pgrep** looks through the currently running processes and lists the process IDs which match the selection criteria to stdout. All the criteria have to match. For example, ```$ pgrep -u root sshd``` will only list the processes whose name include **sshd** AND owned by **root**. On the other hand, ```$ pgrep -u root,daemon``` will list the processes owned by root OR **daemon**.

**pkill** will send the specified signal (by default **SIGTERM**) to each process instead of listing them on stdout.

**pidwait** will wait for each process instead of listing them on stdout.

## Options

- **--signal** signal

    Defines the signal to send to each matched process.  Either the numeric or the symbolic signal name can be used. In **pgrep** or **pidwait** mode only the long option can be used and has no effect unless used in conjunction with **--require-handler** to filter to processes with a userspace signal handler present for a particular signal.

- **-c**, **--count**

    Suppress normal output; instead print a count of matching processes.  When count does not match anything, e.g. returns zero, the command will return non-zero value. Note that for pkill and pidwait, the count is the number of matching processes, not the processes that were successfully signaled or waited for.

- **-d**, **--delimiter** delimiter

    Sets the string used to delimit each process ID in the output (by default a newline). (**pgrep** only.)

- **-e**, **--echo**

    Display name and PID of the process being killed. (**pkill** only.)

- **-f**, **--full**

    The pattern is normally only matched against the process name. When -f is set, the full command line is used.

- **-g**, **--pgroup** pgrp,...

    Only match processes in the process group IDs listed. Process group 0 is translated into **pgrep**'s, **pkill**'s, or **pidwait**'s own process group.

- **-G**, **--group** gid,...

    Only match processes whose real group ID is listed. Either the numerical or symbolical value may be used.

- **-i**, **--ignore-case**

    Match processes case-insensitively.

- **-l**, **--list-name**

    List the process name as well as the process ID. (**pgrep** only.)

- **-a**, **--list-full**

    List the full command line as well as the process ID. (**pgrep** only.)

- **-n**, **--newest**

    Select only the newest (most recently started) of the matching processes.

- **-o**, **--oldest**

    Select only the oldest (least recently started) of the matching processes.

- **-O**, **--older** secs

    Select processes older than secs.

- **-P**, **--parent** ppid,...

    Only match processes whose parent process ID is listed.

- **-s**, **--session** sid,...

    Only match processes whose process session ID is listed. Session ID 0 is translated into **pgrep**'s, **pkill**'s, or **pidwait**'s own session ID.

- **-t**, **--terminal** term,...

    Only match processes whose controlling terminal is listed. The terminal name should be specified without the "/dev/" prefix.

- **-u**, **--euid** euid,...

    Only match processes whose effective user ID is listed. Either the numerical or symbolical value may be used.

- **-U**, **--uid** uid,...

    Only match processes whose real user ID is listed. Either the numerical or symbolical value may be used.

- **-v**, **--inverse**

    Negates the matching. This option is usually used in **pgrep**'s or **pidwait**'s context. In **pkill**'s context the short option is disabled to avoid accidental usage of the option.

- **-w**, **--lightweight**

    Shows all thread ids instead of pids in **pgrep**'s or **pidwait**'s context. In **pkill**'s context this option is disabled.

- **-x**, **--exact**

    Only match processes whose names (or command lines if **-f** is specified) **exactly** match the pattern.

- **-F**, **--pidfile** file

    Read PIDs from file. This option is more useful for **pkill** or **pidwait** than **pgrep**. The filename "-" can be used to read from STDIN.

- **-L**, **--logpidfile**

    Fail if pidfile (see **-F**) not locked.

- **-r**, **--runstates** D,R,S,Z,...

    Match only processes which match the process state.

- **-A**, **--ignore-ancestors**

    Ignore all ancestors of **pgrep**, **pkill**, or **pidwait**. For example, this can be useful when elevating with **sudo** or similar tools.

- **-H**, **--require-handler**

    Only match processes with a userspace signal handler present for the signal to be sent.

- **--cgroup** name,...

    Match on provided control group (cgroup) v2 name. See **cgroups**(8)

- **--env** name[=value],..

    Match on process that have these environment variables. If the =value parameter is not defined then only the variable name is matched.

- **--ns** pid

    Match processes that belong to the same namespaces. Required to run as root to match  processes from other users. See **--nslist** for how to limit which namespaces to match.

- **--nslist** name,...

    Match only the provided namespaces. Available namespaces: ipc, mnt, net, pid, user, uts.

- **-q**, **--queue** value

    Use **sigqueue**(3) rather than **kill**(2) and the value argument is used to specify an integer to be sent with the signal. If the receiving process has installed a handler for this signal using the SA_SIGINFO flag to **sigaction**(2), then it can obtain this data via the si_value field of the siginfo_t structure.

- **-V**, **--version**

    Display version information and exit.

- **-h**, **--help**

    Display help and exit.

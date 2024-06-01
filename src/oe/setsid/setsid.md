# setsid

## Usage
```
setsid [options] <program> [arguments ...]
```

## About

Run a program in a new session.


## Description

**setsid** runs a program in a new session. The command calls **fork**(2) if already a process group leader. Otherwise, it executes a program in the current process. This default behavior is possible to override by the **--fork** option.

## Options

- **-c**, **--ctty**

    Set the controlling terminal to the current one.

- **-f**, **--fork**

    Always create a new process.

- **-w**, **--wait**

    Wait for the execution of the program to end, and return the exit status of this program as the exit status of **setsid**.

- **-V**, **--version**

    Display version information and exit.

- **-h**, **--help**

    Display help text and exit.

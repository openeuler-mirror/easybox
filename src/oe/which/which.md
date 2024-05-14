# which

## Usage
```
which [options] [--] COMMAND [...]
```

## About

Write the full path of COMMAND(s) to standard output.

## Description

**Which** takes one or more arguments. For each of its arguments it prints to stdout the full path of the executables that would have been executed when this argument had been entered at the shell prompt. It does this by searching for an executable or script in the directories listed in the environment variable **PATH** using the same algorithm as **bash**.

## Options

- **-a**, **--all**

    Print all matching executables in **PATH**, not just the first.

- **-i**, **--read-alias**

    Read  aliases  from stdin, reporting matching ones on stdout. This is useful in combination with using an alias for which itself. For example:
        **alias which=´alias | which -i´**.

- **--skip-alias**

    Ignore option '--read-alias', if any. This is useful to explicitly search for normal binaries, while  using  the '--read-alias' option in an alias or function for which.

- **--read-functions**

    Read  shell  function  definitions from stdin, reporting matching ones on stdout. This is useful in combination with using a shell function for which itself.  For example:

    **which() { declare -f | which --read-functions $@ }**

    export -f which

- **--skip-functions**

    Ignore option '--read-functions', if any. This is useful to explicitly search for normal binaries,  while using the '--read-functions' option in an alias or function for which.

- **--skip-dot**

    Skip directories in **PATH** that start with a dot.

- **--skip-tilde**

    Skip directories in **PATH** that start with a tilde and executables which reside in the **HOME** directory.

- **--show-dot**

    If  a directory in **PATH** starts with a dot and a matching executable was found for that path, then print "./programname" rather than the full path.

- **--show-tilde**

    Output a tilde when a directory matches the **HOME** directory. This option is ignored when  which  is invoked as root.

- **--tty-only**

    Stop processing options on the right if not on tty.

- **-h**, **--help**
        Display help text and exit.

- **-v**, **-V**, **--version**
        Print version and exit.

## Return Value

**Which** returns the number of failed arguments, or -1 when no 'programname' was given.

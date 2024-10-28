# find

## Usage
```
find [-H] [-L] [-P] [-D debugopts] [path...] [expression]
```

## About

Search for files in a directory hierarchy.

## Description

**Find** searches the directory tree rooted at each given starting-point by evaluating the given expression for mleft to right, according to the rules of precedence, until the outcome is known (the left hand side is false for `and` operations, true for `or`), at which point **find** moves on to the next file name. If no starting-point is specified, `.` is assumed.

## Options

The  **-H**,  **-L**,  **-P** options control the treatment of symbolic links. Command-line arguments following these are taken to names of files or directories to be examined, up to the first argument that begins with `-`, or the argument `(` or `!`. That argument and any following arguments are taken to be the expression describing what is to be searched for. If no paths are given, the current directory is used. If no expression is given, the expression `-print` is used (but you should probably consider using `-print0` instead, anyway).

Those options in **Expression**, **Operators** and **Actions** control the behaviour of **find** but are specified immediately after the last path name. The five 'real' options **-H**, **-L**, **-P**, **-D** and **-O** must appear before the first path name, if at all. A double dash `--` could theoretically be used to signal that any remaining arguments are not options, but this does not really work due to the way **find** determines the end of the following path arguments: it does that by reading until an expression argument comes (which also starts with a `-`). Now, if a path argument would start with a `-`, then find would treat it as expression argument instead. Thus to ensure that all start points are taken as such, and especially to prevent that wildcard patterns expanded by the calling shell are not mistakenly treated as expression arguments, it is generally safer to prefix wildcards or dubious path names with either `./` or to use absolute path names starting with `/`. Alternatively, it is generally safe though non-protable to use **-files0-from** to pass arbitrary starting points to **find**.

- **-P**

    Never follow symbolic links. This is the default behaviour.

- **-L**

    Follow symbolic links. When **find** examines or prints information about files, the information used shall be taken from the properties of the file to which the linlk points.

- **-H**

    Do not follow symbolic links, except while processing the command line arguments.

If more that one of **-H**, **-L** and **-P** is specified, each overrides the others; the last one appearing on the command line takes effect.

- **-D debugopts**

    Print diagnostic information. Valid debug options include

    -- **search**

        Navigate the directory tree verbosely.

    -- **tree**

        Show the expression tree in its original form.

## Expression

The part of the command line after the list of starting points is the **expression**. This is a kind of query specification describing how we match files and what we do with the files that were matched. An expression is composed of a sequence of things:

- **Tests**

    Tests return a true of false value, usually on the basis of some property of a file we are considering.

- **Actions**

    Actions have side effects (such as printing something on the standard output) and return either true or false, usually based on whether or not they are successful.

- **Global options**

    Global options affect the operation of tests and actions specified on any part of the command line. Global options always return true.

- **Positional options**

    Positional options affect only tests or actions which follow them. Positional options always return true.

- **Operators**

    Operators join together the other items within the expression. Where an operator is mission, **-a** is assumed.

The **-print** action is performed on all files for which the whole expression is true, unless it contains an action other than **-prune** or **-quit**. Actions which inhibit the default **-print** are **-delete**, **-exec**, **-ok**, **-fls**, **-fprint**, **-fprintf**, **-ls**, **-print** and **printf**.

The **-delete** action also acts like an option (since it implies **-depth**).

## Positional options

Positional options always return true. They affect only tests occurring later on the command line.

- **-daystart**

    Measure times from the beginning of today rather than from 24hours ago.

- **-follow**

    Deprecated; use the **-L** option instead. Dereference the symbolic links.

- **-regextype type**

    Changes the regular expression syntax understood by **-regex** and **-iregex** tests which occur later on the command line. For now, **find** only supports **rust** syntax in [Rust `regex` crate](https://crates.io/crates/regex) and **default** (which is **rust**).

## Global options

Global options always return true. Global options take effect even for tests which occur earlier on the command line.

- **-d**, **-depth**

    Process each directory's contents before the directory itself.

- **-files0-from file**

    Read the starting points from file in which file names are separated with NULL instead of getting them on the command line. Using this option and passing starting points on the command line is mutually exclusive.

- **-help**, **--help**

    Print the summary of the command-line usage of **find** and exit.

- **ignore_readdir_race**, **noignore_readdir_race**

    Disable / Enable the warning when **find** is searching a file which have been deleted.

- **-maxdepth levels**, **-mindepth levels**

    Descend at most levels, or do not apply any tests or actions at levels less than levels.

- **-mount**, **-xdev**

    Don't descend directories on other filesystems

- **-version**

    Print the **find** version and exit.

## Tests

A numeric argument n can e specified to tests as

- **+n**

    for greater than n,

- **-n**

    for less than n,

- **n**

    for exactly n.

Supported tests:

- **-amin n**, **-cmin n**, **-mmin n**

    File was last accessed / changed / modified less than, more than or exactly n minutes ago.

- **-anewer reference**, **-cnewer reference**, **-mnewer reference**

    Time of the last access / changes / modified of the current file more recent than that of the last modification of the reference file.

- **-atime n**, **-ctime n**, **-mtime n**

    File was last accessed / changed / modified less than, more than or exactly n days ago.

- **-newer reference**

    Time of the last data modification of the current file is more recent than of the last data modification of the reference file.

- **-newerXY reference**

    succeeds if timestamp X of the file being considered is newer than timestamp Y of reference. The letters X and Y can be any of the following letters:

    -- **a**

        The access time of the file

    -- **c**

        The status change time of the file

    -- **m**

        The modification time of the file

    -- **t**

        reference is interpreted directly as a time, in **date** format

- **-used**

    File was last accessed less than, more than or exactly n days after its status was last changed.

- **-empty**

    File is empty.

- **-executable**, **-readable**, **-writable**

    Matches files which are executable / readable / writable through access system call.

- **-perm mode**

    File's permission bits are exactly mode (octal or symbolic).

- **-perm -mode**

    All of the permission bits mode are set for the file.

- **-perm /mode**, **-perm +mode**

    Any of the permission bits mode are set for the file.

- **-true**, **-false**

    always true / false.

- **-fstype type**

    File is on a filesystem of type.

- **-uid n**, **-gid n**

    Files's numeric owner / group ID is less than, more than or exactly n.

- **-user name**, **-group name**

    File belongs to user / group with name (numeric ID also allowed).

- **-nouser**, **-nogroup**

    No user / group corresponds to file's numeric user / group ID.

- **-name pattern**, **-iname pattern**

    Base of file name (the path with the leading directories removed) matches glob pattern. An exception to this is then using only a slash as pttern, because that is a valid string for matching the root.

    Tests with **i** prefix like **-ipath** are case-insensitive.

- **-lname pattern**, **-ilname pattern**

    File is a symbolic link whose content matches glob pattern.

- **-path pattern**, **-wholename pattern**, **-ipath pattern**, **-iwholename pattern**

    File path matches glob pattern.

- **-regex pattern**, **-iregex pattern**

    File name matches regular expression pattern.

- **-inum n**

    File has inode number smaller than, greater than or exactly n.

- **-samefile name**

    File refers to the same inode as name.

- **-links n**

    File has less than, more than or exactly n hard links.

- **-size n[cwbkMG]**

    File uses less than, more than or exactly n units of space, rounding up.

- **-type c**

    File is of type c:

    -- **b**

        block (buffered) special

    -- **c**

        character (unbuffered) special

    -- **d**

        directory

    -- **p**

        named pipe (FIFO)

    -- **f**

        regular file

    -- **l**

        symbolic link; this is never true if the **-L** option or the **-follow** option is in effect, unless the symbolic link is broken.

    -- **s**

        socket

- **-xtype c**

    The same as **-type** unless the file is symbolic link. For symbolic links: if the **-H** or **-P** option was specified, true if the file is a link to a file of type c; if the **-L** option has been given, true if c is 'l'.

## Actions

- **-delete**

    Delete files or directories. False if failed.

- **-exec command ;**
- **-exec command {} +**

    Execute command; true if 0 status returned. Every `{}` will be replaced by the file name. The `+` variation appends each file name at the end of the commands, like **xarg**.

- **-execdir command ;**
- **-execdir command {} +**

    Like **-exec**, but execute the command in the directory where the file is found.

- **-ls**
- **-fls file**

    True; Print the file information in **ls -dils** format to stdout / file.

- **-print**
- **-fprint**

    True; Print the file name with a newline to stdout / file.

- **-print0**
- **-fprint0**

    True; Print the file name with a NULL to stdout / file.

- **-printf format**
- **-fprintf format**

    True; Printf format to stdout / file. Interpreting c printf '\' escapes and '%' directives. Valid directives are:

              %%     A literal percent sign.

              %a     File's last access time in the format returned by the C ctime(3) function.

              %Ak
              %Ck
              %Tk
                    File's last access / change / modify time in the format **date %k** format.

              %b     The  amount  of  disk  space  used  for  this file in 512-byte blocks.  Since disk space is allocated in multiples of the
                     filesystem block size this is usually greater than %s/512, but it can also be smaller if the file is a sparse file.

              %Bk    File's birth time, i.e., its creation time, in the format specified by k, which is the same as for  %A.   This  directive
                     produces an empty string if the underlying operating system or filesystem does not support birth times.

              %c     File's last status change time in the format returned by the C ctime(3) function.

              %d     File's depth in the directory tree; 0 means the file is a starting-point.

              %D     The device number on which the file exists (the st_dev field of struct stat), in decimal.

              %f     Print  the  basename; the file's name with any leading directories removed (only the last element).  For /, the result is
                     ‘/'.  See the EXAMPLES section for an example.

              %F     Type of the filesystem the file is on; this value can be used for -fstype.

              %g     File's group name, or numeric group ID if the group has no name.

              %G     File's numeric group ID.
              %h     Dirname; the Leading directories of the file's name (all but the last element).  If the file  name  contains  no  slashes
                     (since  it  is in the current directory) the %h specifier expands to ‘.'.  For files which are themselves directories and
                     contain a slash (including /), %h expands to the empty string.  See the EXAMPLES section for an example.

              %H     Starting-point under which file was found.

              %i     File's inode number (in decimal).

              %k     The amount of disk space used for this file in 1 KB blocks.  Since disk space is allocated in multiples of the filesystem
                     block size this is usually greater than %s/1024, but it can also be smaller if the file is a sparse file.

              %l     Object of symbolic link (empty string if file is not a symbolic link).

              %m     File's permission bits (in octal).  This option uses the ‘traditional' numbers which most Unix implementations  use,  but
                     if  your  particular implementation uses an unusual ordering of octal permissions bits, you will see a difference between
                     the actual value of the file's mode and the output of %m.  Normally you will want to have a leading zero on this  number,
                     and to do this, you should use the # flag (as in, for example, ‘%#m').

              %M     File's permissions (in symbolic form, as for ls).  This directive is supported in findutils 4.2.5 and later.

              %n     Number of hard links to file.

              %p     File's name.

              %P     File's name with the name of the starting-point under which it was found removed.

              %s     File's size in bytes.

              %S     File's  sparseness.  This is calculated as (BLOCKSIZE*st_blocks / st_size).  The exact value you will get for an ordinary
                     file of a certain length is system-dependent.  However, normally sparse files will have values less than 1.0,  and  files
                     which  use indirect blocks may have a value which is greater than 1.0.  In general the number of blocks used by a file is
                     file system dependent.  The value used for BLOCKSIZE is system-dependent, but is usually 512 bytes.  If the file size  is
                     zero,  the value printed is undefined.  On systems which lack support for st_blocks, a file's sparseness is assumed to be
                     1.0.
              %t     File's last modification time in the format returned by the C ctime(3) function.
              %u     File's user name, or numeric user ID if the user has no name.

              %U     File's numeric user ID.

              %y     File's type (like in ls -l), U=unknown type (shouldn't happen)

              %Y     File's type (like %y), plus follow symbolic links: ‘L'=loop, ‘N'=nonexistent, ‘?' for any other  error  when  determining
                     the type of the target of a symbolic link.
- **-ok**
- **-okdir**

    Like **-exec**, but ask user for confirmation.

- **-prune**

    True; if the file is a directory, do not descend into it.

- **-quit**

    Exit immediately.

## Operators

- **( expr )**

    Force precedence;

- **! expr**, **-not expr**

    True if expr is false.

- **expr1 -a epxr2**, **expr1 -and expr2**

    True if expr1 and expr2 are true.

- **expr1 -o epxr2**, **expr1 -or expr2**

    True if expr1 or expr2 is true.

- **expr1 , epxr2**

    List; both expr1 and expr2 are always evaluated. The value of expr1 is discarded; the value of the list is the value of expr2.

## Return Value

**Find** exits with status 0 if all files are processed successfully, greater than 0 if errors occur. If the return value is non-zero, you should not rely on the coreectness of the results of **find**.

# sysctl

## Usage
```
sysctl [options] [variable[=value] ...]
```

## About

configure kernel parameters at runtime

## Description

**sysctl** is used to modify kernel parameters at runtime. The parameters available are those listed under /proc/sys/. Procfs is required for **sysctl** support in Linux. You can use **sysctl** to both read and write sysctl data.

## Parameters

- **variable**

    The name of a key to read from. An example is kernel.ostype. The '/' separator is also accepted in place of a '.'.

- **variable=value**

    To set a key, use the form **variable=value** where **variable** is the key and **value** is the value to set it to. If the value contains quotes  or characters which are parsed by the shell, you may need to enclose the value in double quotes.

- **-n**, **--values**

    Use this option to disable printing of the key name when printing values.

- **-e**, **--ignore**

    Use this option to ignore errors about unknown keys.

- **-N**, **--names**

    Use this option to only print the names. It may be useful with shells that have programmable completion.

- **-q**, **--quiet**

    Use this option to not display the values set to stdout.

- **-w**, **--write**

    Force all arguments to be write arguments and print an error if they cannot be parsed this way.

- **-p[FILE]**, **--load[=FILE]**

    Load in **sysctl** settings from the file specified or **/etc/sysctl.conf** if none given. Specifying - as filename means reading data from standard input. Using this option will mean arguments to **sysctl** are files, which are read in the order they are specified. The file argument may be specified as regular expression.

- **-a**, **--all**

    Display all values currently available except deprecated and verboten parameters.

- **--deprecated**

    Include deprecated parameters to **--all** values listing.

- **-b**, **--binary**

    Print value without new line.

- **--system**

    Load settings from all system configuration files.

- **-r**, **--pattern pattern**

    Only apply settings that match **pattern**. The **pattern** uses extended regular expression syntax.

- **-A**

    Alias of **-a**

- **-d**

    Alias of **-h**

- **-f**

    Alias of **-p**

- **-X**

    Alias of **-a**

- **-o**

    Does nothing, exists for BSD compatibility.

- **-x**

    Does nothing, exists for BSD compatibility.

- **-h**, **--help**

    Display help text and exit.

- **-V**, **--version**

    Display version information and exit.

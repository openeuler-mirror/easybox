# grep

## Usage
```
grep [option...] [patterns] [file...]
```

## About

Given one or more patterns, grep searches input files for matches to the patterns. When it finds a match in a line, it copies the line to standard output (by default), or produces whatever other sort of output you have requested with options.

## Options


- **-E**, **--extended-regexp**     PATTERNS are extended regular expressions
- **-F**, **--fixed-strings**       PATTERNS are strings
- **-G**, **--basic-regexp**        PATTERNS are basic regular expressions
- **-P**, **--perl-regexp**         PATTERNS are Perl regular expressions
- **-e**, **--regexp=PATTERNS**     use PATTERNS for matching
- **-f**, **--file=FILE**           take PATTERNS from FILE
- **-i**, **--ignore-case**         ignore case distinctions in patterns and data
- **--no-ignore-case**      do not ignore case distinctions (default)
- **-w**, **--word-regexp**         match only whole words
- **-x**, **--line-regexp**         match only whole lines
- **-z**, **--null-data**           a data line ends in 0 byte, not newline

Miscellaneous:
- **-s**, **--no-messages**         suppress error messages
- **-v**, **--invert-match**        select non-matching lines
- **-V**, **--version**             display version information and exit
- **--help**                display this help text and exit

Output control:
- **-m**, --max-count=NUM       stop after NUM selected lines
- **-b**, --byte-offset         print the byte offset with output lines
- **-n**, --line-number         print line number with output lines
- **--line-buffered**       flush output on every line
- **-H**, **--with-filename**       print file name with output lines
- **-h**, **--no-filename**         suppress the file name prefix on output
- **--label=LABEL**         use LABEL as the standard input file name prefix
- **-o**, **--only-matching**       show only nonempty parts of lines that match
- **-q**, **--quiet**, **--silent**     suppress all normal output
- **--binary-files=TYPE**   assume that binary files are TYPE;
TYPE is 'binary', 'text', or 'without-match'
- **-a**, **--text**                equivalent to --binary-files=text
- **-I**                        equivalent to --binary-files=without-match
- **-d**, **--directories=ACTION**  how to handle directories;
ACTION is 'read', 'recurse', or 'skip'
- **-D**, **--devices=ACTION**      how to handle devices, FIFOs and sockets;
ACTION is 'read' or 'skip'
- **-r**, **--recursive**           like --directories=recurse
- **-R**, **--dereference-recursive**  likewise, but follow all symlinks
- **--include=GLOB**        search only files that match GLOB (a file pattern)
- **--exclude=GLOB**        skip files that match GLOB
- **--exclude-from=FILE**   skip files that match any file pattern from FILE
- **--exclude-dir=GLOB**    skip directories that match GLOB
- **-L**, **--files-without-match**  print only names of FILEs with no selected lines
- **-l**, **--files-with-matches**  print only names of FILEs with selected lines
- **-c**, **--count**               print only a count of selected lines per FILE
- **-T**, **--initial-tab**         make tabs line up (if needed)
- **-Z**, **--null**                print 0 byte after FILE name

Context control:
- **-B**, **--before-context=NUM**  print NUM lines of leading context
- **-A**, **--after-context=NUM**   print NUM lines of trailing context
- **-C**, **--context=NUM**         print NUM lines of output context
- **-NUM**                      same as --context=NUM
- **--color[=WHEN]**,
- **--colour[=WHEN]**       use markers to highlight the matching strings;
WHEN is 'always', 'never', or 'auto'
- **-U**, **--binary**              do not strip CR characters at EOL (MSDOS/Windows)

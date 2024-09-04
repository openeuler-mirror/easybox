# column

## Usage

```bash
column [options] [<file>...]
```

## About

Columnate lists.

## Description

The **column** utility formats its input into multiple columns. The util support three modes:

- **columns are filled before rows**

    This is the default mode (required by backward compatibility).

- **rows are filled before columns**

    This mode is enabled by option **-x, --fillrows**.

- **table**

    Determine the number of columns the input contains and create a table. This mode is enabled by option **-t, --table** and columns formatting is possible to modify by **--table-\*** options. Use this mode if not sure.

Input is taken from file, or otherwise from standard input. Empty lines are ignored and all invalid multibyte sequences are encoded by x\<hex> convention.

## Options

The argument columns for **--table-\*** options is a comma separated list of the column names as defined by **--table-columns** or it’s column number in order as specified by input. It’s possible to mix names and numbers. The special placeholder '0' (e.g. -R0) may be used to specify all columns.

- **-J, --json**

    Use JSON output format to print the table, the option **--table-columns** is required and the option **--table-name** is recommended.

- **-c, --output-width** \<width>

    Output is formatted to a width specified as number of characters. The original name of this option is **--columns**; this name is deprecated since v2.30. Note that input longer than width is not truncated by default.

- **-d, --table-noheadings**

    Do not print header. This option allows the use of logical column names on the command line, but keeps the header hidden when printing the table.

- **-o, --output-separator** \<string>

    Specify the columns delimiter for table output (default is two spaces).

- **-s, --separator** \<separators>

    Specify the possible input item delimiters (default is whitespace).

- **-t, --table**

    Determine the number of columns the input contains and create a table. Columns are delimited with whitespace, by default, or with the characters supplied using the **--output-separator** option. Table output is useful for pretty-printing.

- **-N, --table-columns** \<names>

    Specify the columns names by comma separated list of names. The names are used for the table header or to address column in option arguments.

- **-l, --table-columns-limit** \<number>

    Specify maximal number of the input columns. The last column will contain all remaining line data if the limit is smaller than the number of the columns in the input data.

- **-R, --table-right** \<columns>

    Right align text in the specified columns.

- **-T, --table-truncate** \<columns>

    Specify columns where text can be truncated when necessary, otherwise very long table entries may be printed on multiple lines.

- **-E, --table-noextreme** \<columns>

    Specify columns where is possible to ignore unusually long (longer than average) cells when calculate column width. The option has impact to the width calculation and table formatting, but the printed text is not affected.

    The option is used for the last visible column by default.

- **-e, --table-header-repeat**

    Print header line for each page.

- **-W, --table-wrap** \<columns>

    Specify columns where is possible to use multi-line cell for long text when necessary.

- **-H, --table-hide** \<columns>

   Don’t print specified columns. The special placeholder '-' may be used to hide all unnamed columns (see **--table-columns**).

- **-O, --table-order** \<columns>

    Specify columns order on output.

- **-n, --table-name** \<name>

    Specify the table name used for JSON output. The default is "table".

- **-L, --keep-empty-lines**

    Preserve whitespace-only lines in the input. The default is ignore empty lines at all. This option’s original name was **--table-empty-lines** but is now deprecated because it gives the false impression that the option only applies to table mode.

- **-r, --tree** \<column>

    Specify column to use tree-like output. Note that the circular dependencies and other anomalies in child and parent relation are silently ignored.

- **-i, --tree-id** \<column>

    Specify column with line ID to create child-parent relation.

- **-p, --tree-parent** \<column>

    Specify column with parent ID to create child-parent relation.

- **-x, --fillrows**

    Fill rows before filling columns.

- **-V, --version**

    Display version information and exit.

- **-h, --help**

    Display help text and exit.

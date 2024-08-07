# column

## Usage
```bash
column [options] [<file>...]

column -V

```

## About
Columnate lists.

## Arguments
-t, --table                      create a table
-n, --table-name <name>          table name for JSON output
-O, --table-order <columns>      specify order of output columns
-N, --table-columns <names>      comma separated columns names
-l, --table-columns-limit <num>  maximal number of input columns
-E, --table-noextreme <columns>  don't count long text from the columns to column width
-d, --table-noheadings           don't print header
-e, --table-header-repeat        repeat header for each page
-H, --table-hide <columns>       don't print the columns
-R, --table-right <columns>      right align text in these columns
-T, --table-truncate <columns>   truncate text in the columns when necessary
-W, --table-wrap <columns>       wrap text in the columns when necessary
-L, --keep-empty-lines           don't ignore empty lines
-J, --json                       use JSON output format for table

-r, --tree <column>              column to use tree-like output for the table
-i, --tree-id <column>           line ID to specify child-parent relation
-p, --tree-parent <column>       parent to specify child-parent relation

-c, --output-width <width>       width of output in number of characters
-o, --output-separator <string>  columns separator for table output (default is two spaces)
-s, --separator <string>         possible table delimiters
-x, --fillrows                   fill rows before columns

-h, --help                       display this help
-V, --version                    display version

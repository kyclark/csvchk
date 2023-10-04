# Rust csvchk

Vertical view of delimited text records.

## Usage

Run with `-h|--help` to read usage:

```
$ csvchk -h
Usage: csvchk [OPTIONS] [FILES]...

Arguments:
  [FILES]...  [default: -]

Options:
  -s, --separator <SEPARATOR>
  -l, --limit <LIMIT>          [default: 1]
  -n, --number
  -N, --no-headers
  -d, --dense
  -c, --columns <COLUMNS>
  -h, --help                   Print help
  -V, --version                Print version
```

## Default Limit to One Record

By default, the program will show you the first record:

```
$ csvchk tests/inputs/books.csv
// ****** Record 1 ******//
Author : Émile Zola
Year   : 1865
Title  : La Confession de Claude
```

## Default Input is STDIN

The optional "-" for the input filename indicates that STDIN is the default input stream:

```
$ cat tests/inputs/books.csv | csvchk
// ****** Record 1 ******//
Author : Émile Zola
Year   : 1865
Title  : La Confession de Claude

$ csvchk - < tests/inputs/books.csv
// ****** Record 1 ******//
Author : Émile Zola
Year   : 1865
Title  : La Confession de Claude
```

Use the `-l|--limit` option to indicate more records:

```
$ csvchk tests/inputs/books.csv --limit 2
// ****** Record 1 ******//
Author : Émile Zola
Year   : 1865
Title  : La Confession de Claude

// ****** Record 2 ******//
Author : Samuel Beckett
Year   : 1952
Title  : Waiting for Godot
```

If you use `0`, then all records will be shown:

```
$ csvchk tests/inputs/books.csv --limit 0
// ****** Record 1 ******//
Author : Émile Zola
Year   : 1865
Title  : La Confession de Claude

// ****** Record 2 ******//
Author : Samuel Beckett
Year   : 1952
Title  : Waiting for Godot

// ****** Record 3 ******//
Author : Jules Verne
Year   : 1870
Title  : 20,000 Leagues Under the Sea
```

## Number Columns

The `-n|--number` option will show you 1-based column numbers suitable for field selection with `awk`, `cut`, or `cutr`.
For instance, if I wanted to extract the year of publication:

```
$ csvchk tests/inputs/books.tsv -n
// ****** Record 1 ******//
  1 Author : Émile Zola
  2 Year   : 1865
  3 Title  : La Confession de Claude

$ cut -f 2 tests/inputs/books.tsv
Year
1865
1952
1870
```

## No Headers

Some files have no headers:

```
$ cat tests/inputs/nohdr.csv
a,b,c
d,e,f
g,h,i
```

The `-N|--no-headers` option will supply "Field*" names:

```
$ csvchk --no-headers tests/inputs/nohdr.csv
// ****** Record 1 ******//
Field1 : a
Field2 : b
Field3 : c
```

## Defining/Overriding Column Names

Use `-c|--columns` to supply your own column names, e.g., in the case of a file with no headers:

```
$ csvchk -c 1,2,3 tests/inputs/nohdr.csv
// ****** Record 1 ******//
1 : d
2 : e
3 : f
```

Even with a file that has headers, you can override the column names:

```
$ csvchk -c 1,2,3 tests/inputs/books.tsv
// ****** Record 1 ******//
1 : Émile Zola
2 : 1865
3 : La Confession de Claude
```

Note that `--no-headers` causes the first row to be treated as a data row:

```
$ csvchk -c 1,2,3 -N tests/inputs/books.tsv -l 2
// ****** Record 1 ******//
1 : Author
2 : Year
3 : Title

// ****** Record 2 ******//
1 : Émile Zola
2 : 1865
3 : La Confession de Claude
```

## Detects Record Separator

Here is a CSV file:

```
$ cat tests/inputs/books.csv
Author,Year,Title
Émile Zola,1865,La Confession de Claude
Samuel Beckett,1952,Waiting for Godot
Jules Verne,1870,"20,000 Leagues Under the Sea"
```

It assumes comma-separated:

```
$ csvchk tests/inputs/books.csv
// ****** Record 1 ******//
Author : Émile Zola
Year   : 1865
Title  : La Confession de Claude
```

Here is a tab-delimited file:

```
$ cat tests/inputs/books.tsv
Author	Year	Title
Émile Zola	1865	La Confession de Claude
Samuel Beckett	1952	Waiting for Godot
Jules Verne	1870	20,000 Leagues Under the Sea
```

It works:

```
$ csvchk tests/inputs/books.tsv
// ****** Record 1 ******//
Author : Émile Zola
Year   : 1865
Title  : La Confession de Claude
```

## Indicate Separator

This file uses semicolons:

```
$ cat tests/inputs/books.txt
Author;Year;Title
Émile Zola;1865;La Confession de Claude
Samuel Beckett;1952;Waiting for Godot
Jules Verne;1870;20,000 Leagues Under the Sea
```

So use `-s|--separator` to indicate:

```
$ csvchk -s \; tests/inputs/books.txt
// ****** Record 1 ******//
Author : Émile Zola
Year   : 1865
Title  : La Confession de Claude
```

## Multiple Files

When run with multiple files, a header is inserted.
Note in the following run that the record separator is guessed for each input file:

```
$ csvchk tests/inputs/nohdr.csv tests/inputs/movies1.csv \
> tests/inputs/movies2.csv tests/inputs/movies2.tsv tests/inputs/books.tsv
==> tests/inputs/nohdr.csv <==
// ****** Record 1 ******//
a : d
b : e
c : f

==> tests/inputs/movies1.csv <==
// ****** Record 1 ******//
title    : The Blues Brothers
year     : 1980
director : John Landis

==> tests/inputs/movies2.csv <==
// ****** Record 1 ******//
title    : The Blues Brothers
year     : 1980
director : John Landis

==> tests/inputs/movies2.tsv <==
// ****** Record 1 ******//
title    : The Blues Brothers
year     : 1980
director : John Landis

==> tests/inputs/books.tsv <==
// ****** Record 1 ******//
Author : Émile Zola
Year   : 1865
Title  : La Confession de Claude
```

## Author

Ken Youens-Clark <kyclark@gmail.com>

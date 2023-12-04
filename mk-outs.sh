#!/usr/bin/env bash

set -u

RUN="cargo run --"
OUT=tests/expected
NOHDR=tests/inputs/nohdr.csv 

rm $OUT/*

$RUN $NOHDR > $OUT/nohdr.csv.out
$RUN < tests/inputs/books.tsv > $OUT/books.tsv.stdin.out
$RUN --limit 2 $NOHDR > $OUT/nohdr.csv.limit.2.out
$RUN --no-headers $NOHDR > $OUT/nohdr.csv.no-headers.out
$RUN --number $NOHDR > $OUT/nohdr.csv.number.out
$RUN --number --no-headers $NOHDR > $OUT/nohdr.csv.no-headers.number.out
$RUN -c foo,bar,baz $NOHDR > $OUT/nohdr.csv.columns.out
$RUN -s \; tests/inputs/books.txt > $OUT/books.txt.separator.out
$RUN -d -l 10 tests/inputs/sparse.csv > $OUT/dense.out
$RUN tests/inputs/books.tsv > $OUT/books.tsv.out
$RUN -l 3 tests/inputs/movies2.csv > $OUT/movies2.csv.limit.3.out
$RUN -l 3 tests/inputs/movies2.tsv > $OUT/movies2.tsv.limit.3.out
$RUN -c 1,2,3 -N tests/inputs/books.tsv > $OUT/books.tsv.columns.no-headers.out
$RUN -N -c qseqid,sseqid,pident,length,mismatch,gapopen,qstart,qend,sstart,send,evalue,bitscore tests/inputs/blast.csv > $OUT/blast.csv.columns.no-headers.out
$RUN tests/inputs/nohdr.csv tests/inputs/movies1.csv tests/inputs/movies2.csv tests/inputs/movies2.tsv tests/inputs/books.tsv > $OUT/multiple.out
$RUN -g Hooper tests/inputs/movies2.csv > $OUT/movies2.csv.grep.hooper.titlecase.out
$RUN -g hooper tests/inputs/movies2.csv > $OUT/movies2.csv.grep.hooper.lowercase.out
$RUN -g hooper -i tests/inputs/movies2.csv > $OUT/movies2.csv.grep.hooper.lowercase.insensitive.out

use anyhow::Result;
use assert_cmd::Command;
use predicates::prelude::*;
use rand::{distributions::Alphanumeric, Rng};
use std::fs;

const PRG: &str = "csvchk";
const NOHDR: &str = "tests/inputs/nohdr.csv";
const CSV: &str = "tests/inputs/movies1.csv";
const MOVIES_CSV: &str = "tests/inputs/movies2.csv";
const MOVIES_TSV: &str = "tests/inputs/movies2.tsv";
const SPARSE: &str = "tests/inputs/sparse.csv";
const BOOKS_TSV: &str = "tests/inputs/books.tsv";
const BOOKS_TXT: &str = "tests/inputs/books.txt";
const BLAST_CSV: &str = "tests/inputs/blast.csv";

// --------------------------------------------------
fn gen_bad_file() -> String {
    loop {
        let filename = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();
        if fs::metadata(&filename).is_err() {
            return filename;
        }
    }
}

// --------------------------------------------------
#[test]
fn skips_bad_file() -> Result<()> {
    let bad = gen_bad_file();
    let expected = format!("\"{bad}\": .* [(]os error 2[)]");
    Command::cargo_bin(PRG)?
        .args(&[NOHDR, &bad, CSV])
        .assert()
        .success()
        .stderr(predicate::str::is_match(expected)?);
    Ok(())
}

// --------------------------------------------------
#[test]
fn skips_directory() -> Result<()> {
    Command::cargo_bin(PRG)?
        .args(&["."])
        .assert()
        .success()
        .stderr(predicate::str::is_match("Cannot open a directory")?);
    Ok(())
}

// --------------------------------------------------
fn dies(args: &[&str], expected: &str) -> Result<()> {
    Command::cargo_bin(PRG)?
        .args(args)
        .assert()
        .failure()
        .stderr(predicate::str::contains(expected));
    Ok(())
}

// --------------------------------------------------
#[test]
fn dies_empty_delimiter() -> Result<()> {
    dies(&[CSV, "-s", ""], "--separator \"\" must be a single byte")
}

// --------------------------------------------------
#[test]
fn dies_bad_delimiter() -> Result<()> {
    dies(
        &[CSV, "--separator", ",,"],
        "--separator \",,\" must be a single byte",
    )
}

// --------------------------------------------------
fn run(args: &[&str], expected_file: &str) -> Result<()> {
    println!("expected {}", &expected_file);
    let expected = fs::read_to_string(expected_file)?;
    Command::cargo_bin(PRG)?
        .args(args)
        .assert()
        .success()
        .stdout(expected);
    Ok(())
}

// --------------------------------------------------
#[test]
fn stdin_default() -> Result<()> {
    run_stdin(&[], BOOKS_TSV, "tests/expected/books.tsv.stdin.out")
}

// --------------------------------------------------
#[test]
fn stdin_dash() -> Result<()> {
    run_stdin(&["-"], BOOKS_TSV, "tests/expected/books.tsv.stdin.out")
}

// --------------------------------------------------
fn run_stdin(
    args: &[&str],
    input_file: &str,
    expected_file: &str,
) -> Result<()> {
    let input = fs::read_to_string(input_file)?;
    let expected = fs::read_to_string(expected_file)?;
    Command::cargo_bin(PRG)?
        .args(args)
        .write_stdin(input)
        .assert()
        .success()
        .stdout(expected);
    Ok(())
}

// --------------------------------------------------
#[test]
fn nohdr_default() -> Result<()> {
    run(&[NOHDR], "tests/expected/nohdr.csv.out")
}

// --------------------------------------------------
#[test]
fn nohdr_no_separator_short() -> Result<()> {
    run(
        &[BOOKS_TXT, "-s", ";"],
        "tests/expected/books.txt.separator.out",
    )
}

// --------------------------------------------------
#[test]
fn nohdr_no_separator_long() -> Result<()> {
    run(
        &[BOOKS_TXT, "--separator", ";"],
        "tests/expected/books.txt.separator.out",
    )
}

// --------------------------------------------------
#[test]
fn nohdr_no_limit_short() -> Result<()> {
    run(&[NOHDR, "-l", "2"], "tests/expected/nohdr.csv.limit.2.out")
}

// --------------------------------------------------
#[test]
fn nohdr_no_limit_long() -> Result<()> {
    run(
        &[NOHDR, "--limit", "2"],
        "tests/expected/nohdr.csv.limit.2.out",
    )
}

// --------------------------------------------------
#[test]
fn nohdr_no_headers_short() -> Result<()> {
    run(&[NOHDR, "-N"], "tests/expected/nohdr.csv.no-headers.out")
}

// --------------------------------------------------
#[test]
fn nohdr_no_headers_long() -> Result<()> {
    run(
        &[NOHDR, "--no-headers"],
        "tests/expected/nohdr.csv.no-headers.out",
    )
}

// --------------------------------------------------
#[test]
fn nohdr_number_short() -> Result<()> {
    run(&[NOHDR, "-n"], "tests/expected/nohdr.csv.number.out")
}

// --------------------------------------------------
#[test]
fn nohdr_number_long() -> Result<()> {
    run(&[NOHDR, "--number"], "tests/expected/nohdr.csv.number.out")
}

// --------------------------------------------------
#[test]
fn dense_short() -> Result<()> {
    run(&[SPARSE, "-d", "-l", "10"], "tests/expected/dense.out")
}

// --------------------------------------------------
#[test]
fn dense_long() -> Result<()> {
    run(&[SPARSE, "-l", "10", "--dense"], "tests/expected/dense.out")
}

// --------------------------------------------------
#[test]
fn columns_short_multi() -> Result<()> {
    run(
        &[NOHDR, "-c", "foo", "-c", "bar", "-c", "baz"],
        "tests/expected/nohdr.csv.columns.out",
    )
}

// --------------------------------------------------
#[test]
fn columns_long_multi() -> Result<()> {
    run(
        &[
            NOHDR,
            "--columns",
            "foo",
            "--columns",
            "bar",
            "--columns",
            "baz",
        ],
        "tests/expected/nohdr.csv.columns.out",
    )
}

// --------------------------------------------------
#[test]
fn columns_short_combined() -> Result<()> {
    run(
        &[NOHDR, "-c", "foo,bar,baz"],
        "tests/expected/nohdr.csv.columns.out",
    )
}

// --------------------------------------------------
#[test]
fn columns_long_combined() -> Result<()> {
    run(
        &[NOHDR, "--columns", "foo,bar,baz"],
        "tests/expected/nohdr.csv.columns.out",
    )
}

// --------------------------------------------------
#[test]
fn nohdr_no_headers_number() -> Result<()> {
    run(
        &[NOHDR, "-N", "-n"],
        "tests/expected/nohdr.csv.no-headers.number.out",
    )
}

// --------------------------------------------------
#[test]
fn books_tsv() -> Result<()> {
    run(&[BOOKS_TSV], "tests/expected/books.tsv.out")
}

// --------------------------------------------------
#[test]
fn movies2_csv() -> Result<()> {
    run(
        &[MOVIES_CSV, "-l", "3"],
        "tests/expected/movies2.csv.limit.3.out",
    )
}

// --------------------------------------------------
#[test]
fn movies2_tsv() -> Result<()> {
    run(
        &[MOVIES_TSV, "-l", "3"],
        "tests/expected/movies2.tsv.limit.3.out",
    )
}

// --------------------------------------------------
#[test]
fn books_tsv_columns_no_headers() -> Result<()> {
    run(
        &[BOOKS_TSV, "-c", "1,2,3", "-N"],
        "tests/expected/books.tsv.columns.no-headers.out",
    )
}

// --------------------------------------------------
#[test]
fn blast_csv_columns_no_headers() -> Result<()> {
    let columns = "qseqid,sseqid,pident,length,mismatch,gapopen,qstart,qend,\
                   sstart,send,evalue,bitscore";
    run(
        &[BLAST_CSV, "-c", columns, "-N"],
        "tests/expected/blast.csv.columns.no-headers.out",
    )
}

// --------------------------------------------------
#[test]
fn multiple_files() -> Result<()> {
    run(
        &[NOHDR, CSV, MOVIES_CSV, MOVIES_TSV, BOOKS_TSV],
        "tests/expected/multiple.out",
    )
}

// --------------------------------------------------
#[test]
fn grep1() -> Result<()> {
    run(
        &[MOVIES_CSV, "-g", "Hooper"],
        "tests/expected/movies2.csv.grep.hooper.titlecase.out",
    )
}

// --------------------------------------------------
#[test]
fn grep2() -> Result<()> {
    run(
        &[MOVIES_CSV, "-g", "hooper"],
        "tests/expected/movies2.csv.grep.hooper.lowercase.out",
    )
}

// --------------------------------------------------
#[test]
fn grep3() -> Result<()> {
    run(
        &[MOVIES_CSV, "-g", "hooper", "-i"],
        "tests/expected/movies2.csv.grep.hooper.lowercase.insensitive.out",
    )
}

// --------------------------------------------------
#[test]
fn grep4() -> Result<()> {
    run(
        &[MOVIES_CSV, "-g", "b[lr]", "-i", "-l", "0"],
        "tests/expected/movies2.csv.grep.regex.insensitive.out",
    )
}

// --------------------------------------------------
#[test]
fn grep5() -> Result<()> {
    run(
        &[MOVIES_CSV, "-g", "B[LR]", "-i", "-l", "0"],
        "tests/expected/movies2.csv.grep.regex.insensitive.out",
    )
}

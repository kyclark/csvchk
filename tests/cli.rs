use anyhow::Result;
use assert_cmd::Command;
use predicates::prelude::*;
use pretty_assertions::assert_eq;
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
const GREP_CSV: &str = "tests/inputs/grep.csv";

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
        .failure()
        .stdout(predicate::str::contains(format!("==> {CSV} <==")))
        .stderr(predicate::str::is_match(expected)?);
    Ok(())
}

// --------------------------------------------------
#[test]
fn skips_directory() -> Result<()> {
    Command::cargo_bin(PRG)?
        .args(&["."])
        .assert()
        .failure()
        .stderr(predicate::str::is_match("Cannot open a directory")?);
    Ok(())
}

// --------------------------------------------------
fn complains(args: &[&str], expected: &str) -> Result<()> {
    Command::cargo_bin(PRG)?
        .args(args)
        .assert()
        .failure()
        .stdout("")
        .stderr(predicate::str::contains(expected));
    Ok(())
}

// --------------------------------------------------
#[test]
fn complains_empty_delimiter() -> Result<()> {
    complains(&[CSV, "-s", ""], r#""" must be a single byte"#)
}

// --------------------------------------------------
#[test]
fn complains_bad_delimiter() -> Result<()> {
    complains(&[CSV, "--separator", ",,"], r#"",," must be a single byte"#)
}

// --------------------------------------------------
#[test]
fn complains_bad_pattern_once() -> Result<()> {
    let output = Command::cargo_bin(PRG)?
        .args(&["-g", "(", CSV, MOVIES_CSV])
        .output()?;
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr)?;
    assert_eq!(stderr, "Invalid --grep pattern \"(\"\n");
    Ok(())
}

// --------------------------------------------------
#[test]
fn complains_column_count() -> Result<()> {
    let output = Command::cargo_bin(PRG)?
        .args(&["-N", "-c", "a,b", BOOKS_TSV])
        .output()?;
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8(output.stderr)?;
    assert_eq!(
        stderr,
        format!("{BOOKS_TSV}: Record 1 has 3 fields but there are 2 column names\n")
    );
    Ok(())
}

// --------------------------------------------------
#[test]
fn grep_limit_counts_matches() -> Result<()> {
    // Matches are records 2, 4 and 5; --limit 2 must show two of them
    let output = Command::cargo_bin(PRG)?
        .args(&["-g", "yes", "-l", "2", GREP_CSV])
        .output()?;
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;
    let records: Vec<_> = stdout.lines().filter(|l| l.contains("Record")).collect();
    assert_eq!(
        records,
        [
            "// ****** Record 2 ****** //",
            "// ****** Record 4 ****** //"
        ]
    );
    Ok(())
}

// --------------------------------------------------
#[test]
fn quiet_on_broken_pipe() -> Result<()> {
    use std::io::{BufRead, BufReader};
    use std::process::Stdio;

    // Enough output to overflow the pipe buffer after the reader closes
    let input =
        std::env::temp_dir().join(format!("csvchk-pipe-{}.csv", gen_bad_file()));
    let mut data = String::from("a,b\n");
    for i in 0..100_000 {
        data.push_str(&format!("{i},x\n"));
    }
    fs::write(&input, data)?;

    let mut child = std::process::Command::new(assert_cmd::cargo::cargo_bin(PRG))
        .args(["-l", "0", input.to_str().unwrap()])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let mut first = String::new();
    BufReader::new(child.stdout.take().unwrap()).read_line(&mut first)?;
    // Dropping the reader closes the pipe
    let output = child.wait_with_output()?;
    fs::remove_file(&input)?;

    assert_eq!(first, "// ****** Record 1 ****** //\n");
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stderr)?, "");
    Ok(())
}

// --------------------------------------------------
fn run(args: &[&str], expected_file: &str) -> Result<()> {
    let expected = fs::read_to_string(expected_file)?;
    let output = Command::cargo_bin(PRG)?.args(args).output().unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    assert_eq!(stdout, expected);

    Ok(())
}

// --------------------------------------------------
fn run_stdin(args: &[&str], input_file: &str, expected_file: &str) -> Result<()> {
    let input = fs::read_to_string(input_file)?;
    let expected = fs::read_to_string(expected_file)?;
    let output = Command::cargo_bin(PRG)?
        .write_stdin(input)
        .args(args)
        .output()
        .unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    assert_eq!(stdout, expected);
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
        &[MOVIES_CSV, "-g", "b[lr]", "-i"],
        "tests/expected/movies2.csv.grep.regex.insensitive.out",
    )
}

// --------------------------------------------------
#[test]
fn grep5() -> Result<()> {
    run(
        &[MOVIES_CSV, "-g", "b[lr]", "-i", "-l", "0"],
        "tests/expected/movies2.csv.grep.regex.insensitive.limit.0.out",
    )
}

// --------------------------------------------------
#[test]
fn grep6() -> Result<()> {
    run(
        &[MOVIES_CSV, "-g", "B[LR]", "-i", "-l", "0"],
        "tests/expected/movies2.csv.grep.regex.insensitive.limit.0.out",
    )
}

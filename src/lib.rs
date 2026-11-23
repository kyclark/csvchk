use anyhow::{anyhow, bail, Result};
use clap::Parser;
use csv::ReaderBuilder;
use regex::{Regex, RegexBuilder};
use std::{
    fs::File,
    io::{self, BufRead, BufReader, Write},
    path::Path,
    process::ExitCode,
};
use tabular::{Row, Table};

// --------------------------------------------------
#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    /// Field separator, a single byte (tab for .tsv/.txt files, otherwise comma)
    #[arg(short, long, value_parser = parse_separator)]
    separator: Option<u8>,

    /// Number of records to show per file (0 for all)
    #[arg(short, long, default_value = "1")]
    limit: usize,

    /// Number the fields
    #[arg(short, long)]
    number: bool,

    /// Input has no header row
    #[arg(short = 'N', long)]
    no_headers: bool,

    /// Omit empty fields
    #[arg(short, long)]
    dense: bool,

    /// Comma-separated column names to use instead of the headers
    #[arg(short, long, value_delimiter = ',')]
    columns: Option<Vec<String>>,

    /// Only show records matching this regular expression
    #[arg(short, long)]
    grep: Option<String>,

    /// Make --grep case-insensitive
    #[arg(short, long)]
    insensitive: bool,

    /// Input files, or "-" for STDIN
    #[arg(default_value = "-")]
    files: Vec<String>,
}

// --------------------------------------------------
fn parse_separator(val: &str) -> Result<u8, String> {
    match val.as_bytes() {
        [byte] => Ok(*byte),
        _ => Err(format!(r#""{val}" must be a single byte"#)),
    }
}

// --------------------------------------------------
/// Runs the program, returning failure if any input file had an error.
/// Errors that apply to every file (e.g., a bad --grep pattern) are
/// returned as `Err` before any file is read.
pub fn run(args: Args) -> Result<ExitCode> {
    let grep = args
        .grep
        .as_ref()
        .map(|val| {
            RegexBuilder::new(val)
                .case_insensitive(args.insensitive)
                .build()
                .map_err(|_| anyhow!(r#"Invalid --grep pattern "{val}""#))
        })
        .transpose()?;

    let mut out = io::stdout().lock();
    let mut had_error = false;
    let num_files = args.files.len();

    for filename in &args.files {
        let fh = match open(filename) {
            Err(err) => {
                eprintln!(r#"Error opening "{filename}": {err}"#);
                had_error = true;
                continue;
            }
            Ok(fh) => fh,
        };

        let mut result = Ok(());
        if num_files > 1 {
            result = writeln!(out, "==> {filename} <==").map_err(Into::into);
        }
        if result.is_ok() {
            result = check(fh, filename, &args, grep.as_ref(), &mut out);
        }

        if let Err(e) = result {
            // The reader went away (e.g., piped into `head`): stop quietly.
            if is_broken_pipe(&e) {
                return Ok(ExitCode::SUCCESS);
            }
            eprintln!("{filename}: {e}");
            had_error = true;
        }
    }

    Ok(if had_error {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    })
}

// --------------------------------------------------
fn is_broken_pipe(err: &anyhow::Error) -> bool {
    err.downcast_ref::<io::Error>()
        .is_some_and(|e| e.kind() == io::ErrorKind::BrokenPipe)
}

// --------------------------------------------------
fn check(
    fh: impl BufRead,
    filename: &str,
    args: &Args,
    grep: Option<&Regex>,
    out: &mut impl Write,
) -> Result<()> {
    let delim = args.separator.unwrap_or_else(|| guess_separator(filename));

    let mut reader = ReaderBuilder::new()
        .delimiter(delim)
        .has_headers(!args.no_headers)
        .from_reader(fh);

    // Column names: from --columns, else the header row, else generated
    // as Field1..FieldN from the first record.
    let mut columns: Option<Vec<String>> = if let Some(columns) = &args.columns {
        Some(columns.clone())
    } else if !args.no_headers {
        Some(reader.headers()?.iter().map(String::from).collect())
    } else {
        None
    };

    let mut num_shown = 0;
    for (record_num, record) in reader.records().enumerate() {
        let record = record?;

        if let Some(re) = grep {
            let joined = record.iter().collect::<Vec<_>>().join(" ");
            if !re.is_match(&joined) {
                continue;
            }
        }

        let columns = columns.get_or_insert_with(|| {
            (1..=record.len()).map(|i| format!("Field{i}")).collect()
        });

        if columns.len() != record.len() {
            bail!(
                "Record {} has {} fields but there are {} column names",
                record_num + 1,
                record.len(),
                columns.len()
            )
        }

        writeln!(out, "// ****** Record {} ****** //", record_num + 1)?;
        let mut table = Table::new("{:<} : {:<}");
        for (fnum, (col, value)) in columns.iter().zip(&record).enumerate() {
            if args.dense && value.is_empty() {
                continue;
            }

            let column = if args.number {
                format!("{:3} {col}", fnum + 1)
            } else {
                col.to_string()
            };
            table.add_row(Row::new().with_cell(column).with_cell(value));
        }
        writeln!(out, "{table}")?;

        num_shown += 1;
        if args.limit > 0 && num_shown >= args.limit {
            break;
        }
    }

    Ok(())
}

// --------------------------------------------------
fn guess_separator(filename: &str) -> u8 {
    if filename.ends_with(".txt") || filename.ends_with(".tsv") {
        b'\t'
    } else {
        b','
    }
}

// --------------------------------------------------
#[test]
fn test_guess_separator() {
    assert_eq!(guess_separator("foo"), b',');
    assert_eq!(guess_separator("foo.csv"), b',');
    assert_eq!(guess_separator("foo.txt"), b'\t');
    assert_eq!(guess_separator("foo.tsv"), b'\t');
}

// --------------------------------------------------
#[test]
fn test_parse_separator() {
    assert_eq!(parse_separator(","), Ok(b','));
    assert_eq!(parse_separator("\t"), Ok(b'\t'));
    assert!(parse_separator("").is_err());
    assert!(parse_separator(",,").is_err());
}

// --------------------------------------------------
fn open(filename: &str) -> Result<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => {
            if Path::new(filename).is_dir() {
                bail!("Cannot open a directory")
            } else {
                Ok(Box::new(BufReader::new(File::open(filename)?)))
            }
        }
    }
}

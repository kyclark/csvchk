use anyhow::{bail, Result};
use clap::Parser;
use csv::ReaderBuilder;
use std::{
    fs::File,
    io::{self, BufRead, BufReader},
};
use tabular::{Row, Table};

// --------------------------------------------------
#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    #[arg(short, long)]
    separator: Option<String>,

    #[arg(
        short,
        long,
        default_value = "1",
        value_parser = clap::value_parser!(usize),
    )]
    limit: usize,

    #[arg(short, long, default_value = "false")]
    number: bool,

    #[arg(short = 'N', long, default_value = "false")]
    no_headers: bool,

    #[arg(short, long, default_value = "false")]
    dense: bool,

    #[arg(short, long, value_delimiter = ',')]
    columns: Option<Vec<String>>,

    #[arg(default_value = "-")]
    files: Vec<String>,
}

// --------------------------------------------------
fn main() {
    if let Err(e) = run(Args::parse()) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

// --------------------------------------------------
fn run(args: Args) -> Result<()> {
    let num_files = args.files.len();
    for filename in &args.files {
        match open(filename) {
            Err(err) => eprintln!("{}: {}", filename, err),
            Ok(fh) => {
                if num_files > 1 {
                    println!("==> {filename} <==");
                }
                check(fh, filename, &args)?
            }
        }
    }

    Ok(())
}

// --------------------------------------------------
fn check(fh: impl BufRead, filename: &str, args: &Args) -> Result<()> {
    let default_sep = guess_separator(filename);
    let delim = match &args.separator {
        Some(separator) => {
            let delim_bytes = separator.as_bytes();
            if delim_bytes.len() != 1 {
                bail!("--separator \"{separator}\" must be a single byte");
            }
            delim_bytes.first().unwrap()
        }
        _ => &default_sep,
    };

    let mut reader = ReaderBuilder::new()
        .delimiter(*delim)
        .has_headers(!args.no_headers)
        .from_reader(fh);

    let headers: Option<Vec<String>> = if let Some(columns) = &args.columns {
        Some(columns.clone())
    } else if !args.no_headers {
        let h = &reader.headers()?.clone();
        Some(h.iter().map(|c| c.to_string()).collect())
    } else {
        None
    };

    for (record_num, record) in reader.records().enumerate() {
        let record = record?;
        let values: Vec<&str> = record.iter().collect();
        let columns = &headers.clone().unwrap_or(
            (1..=values.len()).map(|i| format!("Field{i}")).collect(),
        );

        if columns.len() != values.len() {
            bail!("Column names do not match record values")
        }

        println!("// ****** Record {} ******//", record_num + 1);
        let mut table = Table::new("{:<} : {:<}");
        for (fnum, (col, value)) in columns.iter().zip(&values).enumerate() {
            if args.dense && value == &"" {
                continue;
            }

            let column = if args.number {
                format!("{:3} {col}", fnum + 1)
            } else {
                col.to_string()
            };
            table.add_row(Row::new().with_cell(column).with_cell(value));
        }
        println!("{}", table);

        if i + 1 == args.limit {
            break;
        }
    }

    Ok(())
}

// --------------------------------------------------
fn guess_separator(filename: &str) -> u8 {
    if filename.ends_with("tsv") {
        b'\t'
    } else {
        b','
    }
}

// --------------------------------------------------
#[test]
fn test_guess_separator() {
    assert_eq!(guess_separator("foo.tsv"), b'\t');
    assert_eq!(guess_separator("foo.csv"), b',');
    assert_eq!(guess_separator("foo"), b',');
}

// --------------------------------------------------
fn open(filename: &str) -> Result<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

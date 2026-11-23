use clap::Parser;
use csvchk::{run, Args};
use std::process::ExitCode;

// --------------------------------------------------
fn main() -> ExitCode {
    match run(Args::parse()) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

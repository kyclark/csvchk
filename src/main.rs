use clap::Parser;
use csvchk::{Args, run};

// --------------------------------------------------
fn main() {
    if let Err(e) = run(Args::parse()) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

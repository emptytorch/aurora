use std::path::PathBuf;

use clap::{Parser, Subcommand};

mod ast;
mod diagnostic;
mod lexer;
mod machine;
mod parser;
mod span;
mod token;
mod validated;
mod validator;
mod value;

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    cmd: Command,
}

#[derive(Subcommand)]
enum Command {
    Run {
        /// Path to the `.au` file to execute.
        path: PathBuf,
        /// Name of an entry to execute
        entry: Option<String>,
    },
}

fn main() {
    match Args::parse().cmd {
        Command::Run { path, entry } => {
            let input = std::fs::read_to_string(&path).expect("could not read .au file");
            if let Err(err) = machine::execute(&input, entry) {
                match err {
                    machine::ExecutionError::Diagnostic(d) => diagnostic::dump(&input, &path, &d),
                    machine::ExecutionError::Runtime(e) => eprintln!("error: {e}"),
                }
            };
        }
    }
}

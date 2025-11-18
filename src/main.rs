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
    },
}

fn main() {
    match Args::parse().cmd {
        Command::Run { path } => {
            let input = std::fs::read_to_string(&path).expect("could not read .au file");
            if let Err(diagnostic) = machine::execute(&input) {
                diagnostic::dump(&input, &path, &diagnostic);
            };
        }
    }
}

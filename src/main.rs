use std::path::{Path, PathBuf};

use anyhow::Context;
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
        /// Path to the `.au` file to execute
        path: PathBuf,
        /// Name of an entry to execute
        #[arg(long)]
        entry: Option<String>,
    },
}

fn run(path: &Path, entry: Option<String>) -> anyhow::Result<()> {
    let input = std::fs::read_to_string(path)
        .with_context(|| format!("could not read `{}`", path.to_string_lossy()))?;

    match machine::execute(&input, entry) {
        Ok(responses) => {
            for response in responses {
                println!("{}", response.pretty_body());
            }
        }
        Err(err) => match err {
            machine::ExecutionError::Diagnostic(d) => diagnostic::dump(&input, path, &d),
            machine::ExecutionError::Runtime(e) => eprintln!("error: {e}"),
        },
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    match Args::parse().cmd {
        Command::Run { path, entry } => run(&path, entry)?,
    }

    Ok(())
}

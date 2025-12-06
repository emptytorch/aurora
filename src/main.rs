use std::{
    collections::{HashMap, hash_map},
    path::{Path, PathBuf},
};

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
        /// Define a variable
        #[arg(long("var"), value_parser=parse_var_value)]
        vars: Vec<(String, String)>,
    },
}

fn parse_var_value(raw: &str) -> anyhow::Result<(String, String)> {
    if let Some((name, value)) = raw.split_once('=') {
        Ok((name.to_string(), value.to_string()))
    } else {
        Err(anyhow::anyhow!("expected `name=value`"))
    }
}

fn validate_vars(vars: Vec<(String, String)>) -> anyhow::Result<HashMap<String, String>> {
    // TODO: proper validation
    let mut validated_vars = HashMap::with_capacity(vars.len());

    for (name, value) in vars {
        match validated_vars.entry(name.clone()) {
            hash_map::Entry::Occupied(_) => anyhow::bail!("Duplicate variable found `{name}`"),
            hash_map::Entry::Vacant(vacant) => _ = vacant.insert(value),
        }
    }

    Ok(validated_vars)
}

fn run(path: &Path, entry: Option<String>, vars: Vec<(String, String)>) -> anyhow::Result<()> {
    let validated_vars = validate_vars(vars)?;
    let input = std::fs::read_to_string(path)
        .with_context(|| format!("could not read `{}`", path.to_string_lossy()))?;

    match machine::execute(&input, entry, &validated_vars) {
        Ok(responses) => {
            for response in responses {
                if response.status.is_success() && !response.body.is_empty() {
                    println!("{}", response.pretty_body());
                }
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
        Command::Run { path, entry, vars } => run(&path, entry, vars)?,
    }

    Ok(())
}

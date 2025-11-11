use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};

use crate::diagnostic::{Diagnostic, Level};

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

fn dump_diagnostic(input: &str, path: &Path, diagnostic: &Diagnostic) {
    let report = &[annotate_snippets::Level::ERROR
        .primary_title(&diagnostic.message)
        .element(
            annotate_snippets::Snippet::source(input)
                .line_start(1)
                .path(path.to_string_lossy())
                // TODO: multiple labels
                .annotations(diagnostic.labels.iter().map(|label| {
                    annotate_snippets::AnnotationKind::Primary
                        .span(label.span.start..label.span.end)
                        .label(&label.message)
                })),
        )];
    let renderer = annotate_snippets::Renderer::styled()
        .decor_style(annotate_snippets::renderer::DecorStyle::Unicode);
    println!("{}", renderer.render(report));
}

fn main() {
    match Args::parse().cmd {
        Command::Run { path } => {
            let input = std::fs::read_to_string(&path).expect("could not read .au file");
            // TODO: pretty diagnostics
            if let Err(diagnostic) = machine::execute(&input) {
                dump_diagnostic(&input, &path, &diagnostic);
            };
        }
    }
}

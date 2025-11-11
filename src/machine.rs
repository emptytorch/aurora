use std::collections::HashMap;

use crate::{
    diagnostic::Diagnostic,
    parser,
    validated::{Entry, Expr, ExprKind, HttpMethod},
    validator,
    value::Value,
};

pub fn execute(input: &str) -> Result<(), Diagnostic> {
    let items = parser::parse(input)?;
    let entries = validator::validate(items)?;
    let machine = Machine::new(entries);
    machine.execute()
}

struct Machine<'input> {
    entries: HashMap<&'input str, Entry<'input>>,
    client: reqwest::blocking::Client,
}

impl<'input> Machine<'input> {
    fn new(entries: HashMap<&'input str, Entry<'input>>) -> Self {
        Self {
            entries,
            client: reqwest::blocking::Client::new(),
        }
    }

    fn execute(&self) -> Result<(), Diagnostic> {
        for entry in self.entries.values() {
            self.execute_entry(entry)?;
        }

        Ok(())
    }

    fn execute_entry(&self, entry: &Entry<'input>) -> Result<(), Diagnostic> {
        let Some(request) = &entry.request else {
            println!(
                "I could not find any request in entry `{}`. Skipping...",
                entry.name
            );
            return Ok(());
        };

        let url = self.eval_expr(&request.url)?;
        match request.method {
            HttpMethod::Get => {
                // TODO: error handling
                let response = self.client.get(url.string()).send().unwrap();
                // TODO: format
                println!("{:#?}", response);
            }
        }

        Ok(())
    }

    fn eval_expr(&self, expr: &Expr) -> Result<Value, Diagnostic> {
        match &expr.kind {
            ExprKind::StringLiteral(s) => Ok(Value::String(s.to_owned())),
        }
    }
}

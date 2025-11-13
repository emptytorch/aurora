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
                let mut req = self.client.get(url.string());
                if let Some(expr) = &entry.headers {
                    let headers = self.eval_headers(expr)?;
                    for (k, v) in headers {
                        req = req.header(k, v);
                    }
                    println!("{:?}", req);
                }

                // TODO: error handling
                let response = req.send().unwrap();
                // TODO: format
                println!("{:#?}", response);
            }
        }

        Ok(())
    }

    // fn map_headers(&self, dictionary: &Value) -> reqwest::header::HeaderMap {
    //     if let Value::Dictionary(dict) = dictionary {
    //     } else {
    //         unreachable!("Body of `[Headers]` should be a dictionary")
    //     };
    // }
    //
    fn eval_headers(&self, expr: &Expr) -> Result<HashMap<String, String>, Diagnostic> {
        if let ExprKind::Dictionary(fields) = &expr.kind {
            fields
                .iter()
                .map(|f| {
                    Ok((
                        self.eval_expr(&f.key)?.string().to_owned(),
                        self.eval_expr(&f.value)?.string().to_owned(),
                    ))
                })
                .collect::<Result<HashMap<_, _>, _>>()
        } else {
            unreachable!("Body of `[Headers]` should be a dictionary")
        }
    }

    fn eval_expr(&self, expr: &Expr) -> Result<Value, Diagnostic> {
        match &expr.kind {
            ExprKind::StringLiteral(s) => Ok(Value::String(s.to_owned())),
            ExprKind::Dictionary(fields) => {
                let mut map = HashMap::new();
                for field in fields {
                    let key = self.eval_expr(&field.key)?.string().to_owned();
                    let value = self.eval_expr(&field.value)?;
                    map.insert(key, value);
                }
                Ok(Value::Dictionary(map))
            }
        }
    }
}

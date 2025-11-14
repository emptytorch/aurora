use std::{collections::HashMap, str::FromStr};

use crate::{
    diagnostic::Diagnostic,
    parser,
    validated::{Entry, Expr, ExprKind, HttpMethod, SourceFile},
    validator,
    value::Value,
};

pub fn execute(input: &str) -> Result<(), Diagnostic> {
    let items = parser::parse(input)?;
    let file = validator::validate(items)?;
    let mut machine = Machine::new(file);
    machine.execute()
}

struct Machine<'input> {
    source_file: SourceFile<'input>,
    names: HashMap<String, Value>,
    client: reqwest::blocking::Client,
}

impl<'input> Machine<'input> {
    fn new(source_file: SourceFile<'input>) -> Self {
        Self {
            source_file,
            names: HashMap::new(),
            client: reqwest::blocking::Client::new(),
        }
    }

    fn execute(&mut self) -> Result<(), Diagnostic> {
        for (name, expr) in &self.source_file.globals {
            let value = self.eval_expr(expr)?;
            self.names.insert(name.to_string(), value);
        }

        for entry in self.source_file.entries.values() {
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
                    let dictionary = self.eval_expr(expr)?;
                    let headers = self.map_headers(&dictionary);
                    req = req.headers(headers);
                }

                if let Some(body) = &entry.body {
                    let value = self.eval_expr(body)?;
                    req = req.body(value.to_json().to_string());
                }

                // TODO: error handling
                let response = req.send().unwrap();
                // TODO: format
                println!("{:#?}", response);
            }
            HttpMethod::Post => {
                let mut req = self.client.post(url.string());
                if let Some(expr) = &entry.headers {
                    let dictionary = self.eval_expr(expr)?;
                    let headers = self.map_headers(&dictionary);
                    req = req.headers(headers);
                }

                if let Some(body) = &entry.body {
                    let value = self.eval_expr(body)?;
                    req = req.body(value.to_json().to_string());
                }

                // TODO: error handling
                let response = req.send().unwrap();
                // TODO: format
                println!("{:#?}", response);
            }
        }

        Ok(())
    }

    fn map_headers(&self, dictionary: &Value) -> reqwest::header::HeaderMap {
        if let Value::Dictionary(dict) = dictionary {
            let mut header_map = reqwest::header::HeaderMap::with_capacity(dict.len());
            // TODO: error handling
            for (k, v) in dict {
                header_map.insert(
                    reqwest::header::HeaderName::from_str(k).unwrap(),
                    reqwest::header::HeaderValue::from_str(v.string()).unwrap(),
                );
            }
            header_map
        } else {
            unreachable!("Body of `[Headers]` should be a dictionary")
        }
    }

    fn eval_expr(&self, expr: &Expr) -> Result<Value, Diagnostic> {
        match &expr.kind {
            ExprKind::StringLiteral(s) => Ok(Value::String(s.to_owned())),
            ExprKind::IntegerLiteral(i) => Ok(Value::Integer(*i)),
            ExprKind::Dictionary(fields) => {
                let mut map = HashMap::new();
                for field in fields {
                    let key = self.eval_expr(&field.key)?.string().to_owned();
                    let value = self.eval_expr(&field.value)?;
                    map.insert(key, value);
                }
                Ok(Value::Dictionary(map))
            }
            ExprKind::NameRef(name) => Ok(self.names[name].clone()),
        }
    }
}

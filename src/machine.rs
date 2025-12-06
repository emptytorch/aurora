use std::{collections::HashMap, str::FromStr};

use indexmap::IndexMap;

use crate::{
    diagnostic::Diagnostic,
    validated::{Entry, Expr, ExprKind, HttpMethod, SourceFile, TemplatePart},
    validator,
    value::Value,
};

#[derive(Debug)]
pub struct Response {
    pub status: StatusCode,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

#[derive(Debug, Clone, Copy)]
pub struct StatusCode(u16);

impl StatusCode {
    pub fn is_success(self) -> bool {
        (200..300).contains(&self.0)
    }
}

impl Response {
    pub fn pretty_body(&self) -> String {
        let content_type = self
            .headers
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case("Content-Type"))
            .map(|(_, v)| v.as_str())
            .unwrap_or_default();

        let body_str = String::from_utf8_lossy(&self.body);
        if content_type.contains("application/json") {
            return serde_json::from_str::<serde_json::Value>(&body_str)
                .map(|v| serde_json::to_string_pretty(&v).unwrap_or_else(|_| body_str.to_string()))
                .unwrap_or_else(|_| body_str.to_string());
        }

        body_str.to_string()
    }
}

#[derive(Debug)]
pub enum ExecutionError {
    Diagnostic(Diagnostic),
    Runtime(RuntimeError),
}

impl From<Diagnostic> for ExecutionError {
    fn from(value: Diagnostic) -> Self {
        ExecutionError::Diagnostic(value)
    }
}

#[derive(Debug)]
pub enum RuntimeError {
    EntryNotFound(String),
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::EntryNotFound(entry) => {
                write!(f, "I couldn't find any entry named `{entry}`")
            }
        }
    }
}

pub fn execute(
    input: &str,
    entry_name: Option<String>,
    external_vars: &HashMap<String, String>,
) -> Result<Vec<Response>, ExecutionError> {
    let file = validator::validate(input, external_vars)?;
    let mut machine = Machine::new();
    machine.execute(file, entry_name, external_vars)
}

struct Machine {
    names: HashMap<String, Value>,
    client: reqwest::blocking::Client,
}

impl<'input> Machine {
    fn new() -> Self {
        Self {
            names: HashMap::new(),
            client: reqwest::blocking::Client::new(),
        }
    }

    fn execute(
        &mut self,
        source_file: SourceFile<'input>,
        entry_name: Option<String>,
        external_vars: &HashMap<String, String>,
    ) -> Result<Vec<Response>, ExecutionError> {
        for (name, value) in external_vars {
            self.names
                .insert(name.clone(), Value::String(value.clone()));
        }

        for konst in source_file.globals.values() {
            let value = self.eval_expr(&konst.expr)?;
            self.names.insert(konst.name.text.to_string(), value);
        }

        match entry_name {
            Some(name) => {
                let entry = source_file
                    .entries
                    .get(name.as_str())
                    .ok_or(ExecutionError::Runtime(RuntimeError::EntryNotFound(name)))?;

                if let Some(response) = self.execute_entry(entry)? {
                    Ok(vec![response])
                } else {
                    Ok(vec![])
                }
            }
            None => {
                let mut responses = vec![];
                for entry in source_file.entries.values() {
                    if let Some(response) = self.execute_entry(entry)? {
                        responses.push(response);
                    }
                }

                Ok(responses)
            }
        }
    }

    fn execute_entry(&self, entry: &Entry<'input>) -> Result<Option<Response>, ExecutionError> {
        let Some(request) = &entry.request else {
            println!(
                "I could not find any request in entry `{}`. Skipping...",
                entry.name.text
            );
            return Ok(None);
        };

        let url = self.eval_expr(&request.url)?;
        let mut req = match request.method {
            HttpMethod::Get => self.client.get(url.string()),
            HttpMethod::Post => self.client.post(url.string()),
            HttpMethod::Put => self.client.put(url.string()),
            HttpMethod::Patch => self.client.patch(url.string()),
            HttpMethod::Delete => self.client.delete(url.string()),
        };

        if let Some(expr) = &entry.headers {
            let value = self.eval_expr(expr)?;
            let headers = self.map_headers(value.dictionary());
            req = req.headers(headers);
        }

        if let Some(body) = &entry.body {
            let value = self.eval_expr(body)?;
            req = req.body(value.to_json().to_string());
        }

        // TODO: error handling
        let response = req.send().unwrap();
        Ok(Some(Response {
            status: StatusCode(response.status().as_u16()),
            headers: response
                .headers()
                .iter()
                .map(|(name, value)| (name.to_string(), value.to_str().unwrap().to_string()))
                .collect(),
            // TODO: error handling
            body: response.bytes().unwrap().to_vec(),
        }))
    }

    fn map_headers(&self, dictionary: &IndexMap<String, Value>) -> reqwest::header::HeaderMap {
        let mut header_map = reqwest::header::HeaderMap::with_capacity(dictionary.len());
        // TODO: error handling
        for (k, v) in dictionary {
            header_map.insert(
                reqwest::header::HeaderName::from_str(k).unwrap(),
                reqwest::header::HeaderValue::from_str(v.string()).unwrap(),
            );
        }
        header_map
    }

    fn eval_expr(&self, expr: &Expr) -> Result<Value, ExecutionError> {
        match &expr.kind {
            ExprKind::StringLiteral(parts) => {
                let mut out = String::new();
                for part in parts {
                    match part {
                        TemplatePart::Literal(s) => {
                            out.push_str(s);
                        }
                        TemplatePart::Expr(expr) => {
                            let value = self.eval_expr(expr)?;
                            out.push_str(&value.to_string());
                        }
                    }
                }

                Ok(Value::String(out))
            }
            ExprKind::IntegerLiteral(i) => Ok(Value::Integer(*i)),
            ExprKind::FloatLiteral(f) => Ok(Value::Float(*f)),
            ExprKind::NullLiteral => Ok(Value::Null),
            ExprKind::Dictionary(fields) => {
                let mut map = IndexMap::with_capacity(fields.len());
                for field in fields {
                    let key = self.eval_expr(&field.key)?.string().to_owned();
                    let value = self.eval_expr(&field.value)?;
                    map.insert(key, value);
                }
                Ok(Value::Dictionary(map))
            }
            ExprKind::Array(elems) => {
                let mut values = Vec::with_capacity(elems.len());
                for elem in elems {
                    let value = self.eval_expr(elem)?;
                    values.push(value);
                }
                Ok(Value::Array(values))
            }
            ExprKind::NameRef(name) => Ok(self.names[name].clone()),
        }
    }
}

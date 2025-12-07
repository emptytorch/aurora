use std::collections::HashMap;

use indexmap::IndexMap;

use crate::{
    client::{HttpClient, ReqwestHttpClient},
    diagnostic::Diagnostic,
    validated::{Entry, Expr, ExprKind, HttpMethod, SourceFile, TemplatePart},
    validator,
    value::Value,
};

#[derive(Debug)]
pub struct Request {
    pub method: HttpMethod,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
}

#[derive(Debug)]
pub struct Response {
    pub status: StatusCode,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

#[derive(Debug, Clone, Copy)]
pub struct StatusCode(u16);

impl From<u16> for StatusCode {
    fn from(value: u16) -> Self {
        StatusCode(value)
    }
}

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
    let client = ReqwestHttpClient::new();
    let mut machine = Machine::new(client);
    machine.execute(file, entry_name, external_vars)
}

struct Machine<C: HttpClient> {
    names: HashMap<String, Value>,
    client: C,
}

impl<'input, C: HttpClient> Machine<C> {
    fn new(client: C) -> Self {
        Self {
            names: HashMap::new(),
            client,
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

        let mut headers = vec![];
        if let Some(expr) = &entry.headers {
            let value = self.eval_expr(expr)?;
            for (k, v) in value.dictionary() {
                headers.push((k.clone(), v.string().to_string()));
            }
        }

        let body = if let Some(expr) = &entry.body {
            Some(self.eval_expr(expr)?.to_json().to_string())
        } else {
            None
        };

        let request = Request {
            method: request.method,
            url: url.string().to_string(),
            headers,
            body,
        };

        let response = self.client.send(request);
        Ok(Some(response))
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

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Dictionary(HashMap<String, Value>),
}

impl Value {
    pub fn string(&self) -> &str {
        match self {
            Value::String(s) => s,
            _ => panic!("Expected a string"),
        }
    }
}

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Integer(i64),
    Float(f64),
    Dictionary(HashMap<String, Value>),
}

impl Value {
    pub fn string(&self) -> &str {
        match self {
            Value::String(s) => s,
            _ => panic!("Expected a string"),
        }
    }

    pub fn dictionary(&self) -> &HashMap<String, Value> {
        match self {
            Value::Dictionary(d) => d,
            _ => panic!("Expected a dictionary"),
        }
    }

    pub fn to_json(&self) -> serde_json::Value {
        match self {
            Value::String(s) => serde_json::Value::String(s.clone()),
            Value::Integer(i) => serde_json::Value::Number((*i).into()),
            Value::Float(f) => serde_json::Value::Number(
                serde_json::Number::from_f64(*f).expect("Number should be finite"),
            ),
            Value::Dictionary(d) => {
                let mut map = serde_json::Map::new();
                for (k, v) in d {
                    map.insert(k.clone(), v.to_json());
                }
                serde_json::Value::Object(map)
            }
        }
    }
}

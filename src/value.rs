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

    pub fn stringify(&self) -> String {
        match self {
            Value::String(s) => stringify_string(s),
            Value::Integer(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Dictionary(d) => {
                let mut keys: Vec<&String> = d.keys().collect();
                keys.sort();
                let inner = keys
                    .iter()
                    .map(|it| format!("{}: {}", stringify_string(it), d[*it].stringify()))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{{{}}}", inner)
            }
        }
    }
}

fn stringify_string(s: &str) -> String {
    let mut out = String::new();
    out.push('"');
    for c in s.chars() {
        match c {
            '\n' => out.push_str("\\n"),
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stringify_string_simple() {
        let v = Value::String("foo".to_string());
        assert_eq!(v.stringify(), r#""foo""#);
    }

    #[test]
    fn stringify_string_newline() {
        let v = Value::String("foo\nbar".into());
        assert_eq!(v.stringify(), r#""foo\nbar""#);
    }

    #[test]
    fn stringify_string_quote() {
        let v = Value::String(r#"foo"bar"#.into());
        assert_eq!(v.stringify(), r#""foo\"bar""#);
    }

    #[test]
    fn stringify_string_backslash() {
        let v = Value::String(r#"foo\bar"#.into());
        assert_eq!(v.stringify(), r#""foo\\bar""#);
    }

    #[test]
    fn stringify_string_mixed() {
        let v = Value::String("one\\two\nthree\"end\"".into());
        assert_eq!(v.stringify(), r#""one\\two\nthree\"end\"""#);
    }

    #[test]
    fn stringify_integer() {
        let v = Value::Integer(42);
        assert_eq!(v.stringify(), "42");
    }

    #[test]
    fn stringify_float() {
        let v = Value::Float(1.23);
        assert_eq!(v.stringify(), "1.23");
    }

    #[test]
    fn stringify_dict_simple() {
        let mut map = std::collections::HashMap::new();
        map.insert("a".to_string(), Value::Integer(1));
        map.insert("b".to_string(), Value::Integer(2));

        let v = Value::Dictionary(map);
        assert_eq!(v.stringify(), r#"{"a": 1, "b": 2}"#);
    }

    #[test]
    fn stringify_dict_with_escaped_key_and_value() {
        let mut map = std::collections::HashMap::new();
        map.insert(
            r#"ke"y"#.to_string(),
            Value::String(r#"va"lue"#.to_string()),
        );

        let v = Value::Dictionary(map);
        assert_eq!(v.stringify(), r#"{"ke\"y": "va\"lue"}"#);
    }

    #[test]
    fn stringify_nested_dict() {
        let mut inner = std::collections::HashMap::new();
        inner.insert("x".to_string(), Value::Integer(9));

        let mut outer = std::collections::HashMap::new();
        outer.insert("inner".to_string(), Value::Dictionary(inner));

        let v = Value::Dictionary(outer);
        assert_eq!(v.stringify(), r#"{"inner": {"x": 9}}"#);
    }
}

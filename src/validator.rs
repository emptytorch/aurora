use std::collections::{HashMap, hash_map};

use crate::{
    ast,
    diagnostic::{Diagnostic, Level},
    span::Span,
    validated::{self, Entry},
};

pub fn validate<'input>(
    items: Vec<ast::Item<'input>>,
) -> Result<HashMap<&'input str, Entry<'input>>, Diagnostic> {
    let mut entries = HashMap::new();
    for item in &items {
        match &item.kind {
            ast::ItemKind::Entry(entry) => {
                let entry_name = entry.name;
                let validated_entry = validate_entry(entry)?;
                match entries.entry(entry_name.text) {
                    hash_map::Entry::Occupied(_) => {
                        return Err(Diagnostic::error(
                            format!("The entry `{}` is defined multiple times", entry_name.text),
                            entry_name.span,
                        )
                        .label(
                            "I have already seen an entry with this name",
                            entry_name.span,
                            Level::Error,
                        ));
                    }
                    hash_map::Entry::Vacant(vacant) => _ = vacant.insert(validated_entry),
                }
            }
        }
    }

    Ok(entries)
}

fn validate_entry<'input>(
    entry: &ast::Entry<'input>,
) -> Result<validated::Entry<'input>, Diagnostic> {
    let mut validated_request = None;
    for item in &entry.body {
        match &item.kind {
            ast::EntryItemKind::Request(request) => {
                let validated_url = validate_expr(&request.url)?;
                if validated_url.ty != validated::Ty::String {
                    return Err(
                        Diagnostic::error("Mismatched types", request.url.span).label(
                            "I was expecting a string here",
                            request.url.span,
                            Level::Error,
                        ),
                    );
                }

                match validated_request {
                    Some(_) => {
                        return Err(Diagnostic::error(
                            format!("Entry `{}` contains multiple requests", entry.name.text),
                            item.span,
                        )
                        .label(
                            format!(
                                "I was expecting to find one request in entry `{}`",
                                entry.name.text
                            ),
                            item.span,
                            Level::Error,
                        ));
                    }
                    None => {
                        validated_request = Some(validated::Request {
                            method: match request.method {
                                ast::HttpMethod::Get => validated::HttpMethod::Get,
                            },
                            url: validated_url,
                        })
                    }
                }
            }
        }
    }

    Ok(validated::Entry {
        name: entry.name.text,
        request: validated_request,
    })
}

fn validate_expr(expr: &ast::Expr) -> Result<validated::Expr, Diagnostic> {
    match expr.kind {
        ast::ExprKind::StringLiteral(s) => {
            let unescaped = unescape_string(s, expr.span)?;
            Ok(validated::Expr {
                kind: validated::ExprKind::StringLiteral(unescaped),
                ty: validated::Ty::String,
            })
        }
    }
}

fn unescape_string(raw: &str, span: Span) -> Result<String, Diagnostic> {
    let mut result = String::new();
    let mut escape = false;
    // Skip surrounding quotes
    for (i, c) in raw[1..raw.len() - 1].char_indices() {
        if escape {
            let unescaped = match c {
                'n' => '\n',
                '\\' => '\\',
                '"' => '"',
                _ => {
                    let absolute_index = span.start + 1 + i;
                    let span = Span::new(absolute_index, absolute_index + c.len_utf8());
                    return Err(
                        Diagnostic::error(format!("Unknown character escape `{c}`"), span).label(
                            "I don't know how to handle this character escape",
                            span,
                            Level::Error,
                        ),
                    );
                }
            };
            result.push(unescaped);
            escape = false;
        } else if c == '\\' {
            escape = true;
        } else {
            result.push(c);
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unescape_ok(raw: &str) -> String {
        unescape_string(raw, Span::new(0, raw.len())).expect("String should unescape successfully")
    }

    #[test]
    fn unescape_string_simple() {
        assert_eq!(unescape_ok(r#""foo""#), "foo");
    }

    #[test]
    fn unescape_string_with_newline() {
        assert_eq!(unescape_ok(r#""foo\nbar""#), "foo\nbar");
    }

    #[test]
    fn unescape_string_with_quote() {
        assert_eq!(unescape_ok(r#""foo\"bar\"""#), "foo\"bar\"");
    }

    #[test]
    fn unescape_string_with_backslash() {
        assert_eq!(unescape_ok(r#""foo\\bar""#), "foo\\bar");
    }

    #[test]
    fn unescape_string_mixed() {
        assert_eq!(
            unescape_ok(r#""one\\two\nthree\"end\"""#),
            "one\\two\nthree\"end\""
        );
    }

    #[test]
    fn unescape_string_invalid_escape_points_to_correct_span() {
        let input = r#""foo\qbar""#;
        let string_span = Span::new(0, 10);
        let diagnostic =
            unescape_string(input, string_span).expect_err("unknown character escape should fail");
        let expected_span = Span::new(5, 6);

        assert_eq!(diagnostic.span, expected_span);
        assert_eq!(diagnostic.message, "Unknown character escape `q`");

        assert_eq!(diagnostic.labels.len(), 1);
        let label = &diagnostic.labels[0];
        assert_eq!(label.span, expected_span);
        assert_eq!(
            label.message,
            "I don't know how to handle this character escape"
        );
    }
}

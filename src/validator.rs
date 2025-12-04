use indexmap::{IndexMap, map};

use crate::{
    ast,
    diagnostic::{Diagnostic, Level},
    parser,
    span::Span,
    validated::{self},
};

pub fn validate<'input>(input: &'input str) -> Result<validated::SourceFile<'input>, Diagnostic> {
    let items = parser::parse(input)?;
    let validator = Validator::new();
    validator.validate(items)
}

struct Validator<'input> {
    globals: IndexMap<&'input str, validated::Const<'input>>,
    entries: IndexMap<&'input str, validated::Entry<'input>>,
}

impl<'input> Validator<'input> {
    fn new() -> Self {
        Self {
            globals: IndexMap::new(),
            entries: IndexMap::new(),
        }
    }

    fn validate(
        mut self,
        items: Vec<ast::Item<'input>>,
    ) -> Result<validated::SourceFile<'input>, Diagnostic> {
        for item in items {
            match item.kind {
                ast::ItemKind::Entry(entry) => {
                    let entry_name = entry.name;
                    let validated_entry = self.validate_entry(entry)?;
                    match self.entries.entry(entry_name.text) {
                        map::Entry::Occupied(occupied) => {
                            return Err(Diagnostic::error(
                                format!(
                                    "The entry `{}` is defined multiple times",
                                    entry_name.text
                                ),
                                entry_name.span,
                            )
                            .primary_label(
                                "I have already seen an entry with this name",
                                Level::Error,
                            )
                            .label(
                                "It was first defined here",
                                occupied.get().name.span,
                                Level::Error,
                            ));
                        }
                        map::Entry::Vacant(vacant) => _ = vacant.insert(validated_entry),
                    }
                }
                ast::ItemKind::Const(name, expr) => {
                    let validated_expr = self.validate_expr(expr)?;
                    match self.globals.entry(name.text) {
                        map::Entry::Occupied(occupied) => {
                            return Err(Diagnostic::error(
                                format!("The variable `{}` is defined multiple times", name.text),
                                name.span,
                            )
                            .primary_label(
                                "I have already seen a variable with this name",
                                Level::Error,
                            )
                            .label(
                                "It was first defined here",
                                occupied.get().name.span,
                                Level::Error,
                            ));
                        }
                        map::Entry::Vacant(vacant) => {
                            _ = vacant.insert(validated::Const {
                                name: validated::Name {
                                    text: name.text,
                                    span: name.span,
                                },
                                expr: validated_expr,
                            })
                        }
                    }
                }
            }
        }

        Ok(validated::SourceFile {
            entries: self.entries,
            globals: self.globals,
        })
    }

    fn validate_entry(
        &self,
        entry: ast::Entry<'input>,
    ) -> Result<validated::Entry<'input>, Diagnostic> {
        let mut validated_request = None;
        let mut validated_headers = None;
        let mut validated_body = None;
        for item in entry.body {
            match item.kind {
                ast::EntryItemKind::Request(request) => {
                    let url_span = request.url.span;
                    let validated_url = self.validate_expr(request.url)?;
                    if validated_url.ty != validated::Ty::String {
                        return Err(Diagnostic::error("Mismatched types", url_span)
                            .primary_label("I was expecting a string here", Level::Error));
                    }

                    match validated_request {
                        Some(_) => {
                            return Err(Diagnostic::error(
                                format!("Entry `{}` contains multiple requests", entry.name.text),
                                item.span,
                            )
                            .primary_label(
                                format!(
                                    "I was expecting to find one request in entry `{}`",
                                    entry.name.text
                                ),
                                Level::Error,
                            ));
                        }
                        None => {
                            validated_request = Some(validated::Request {
                                method: match request.method {
                                    ast::HttpMethod::Get => validated::HttpMethod::Get,
                                    ast::HttpMethod::Post => validated::HttpMethod::Post,
                                    ast::HttpMethod::Put => validated::HttpMethod::Put,
                                    ast::HttpMethod::Patch => validated::HttpMethod::Patch,
                                    ast::HttpMethod::Delete => validated::HttpMethod::Delete,
                                },
                                url: validated_url,
                            })
                        }
                    }
                }
                ast::EntryItemKind::Section(name, body) => {
                    let body_span = body.span;
                    let validated_expr = self.validate_expr(body)?;
                    match name.text {
                        "Headers" => {
                            if let validated::Ty::Dictionary(value_types) = &validated_expr.ty {
                                if !value_types.iter().all(|it| *it == validated::Ty::String) {
                                    return Err(Diagnostic::error("Unexpected types", body_span)
                                        .primary_label(
                                            "I was expecting all the values to be strings here",
                                            Level::Error,
                                        ));
                                }
                            } else {
                                return Err(Diagnostic::error("Unexpected type", body_span)
                                    .primary_label(
                                        "I was expecting a dictionary here",
                                        Level::Error,
                                    ));
                            };

                            match validated_headers {
                                Some(_) => {
                                    return Err(Diagnostic::error(
                                                format!(
                                                    "Entry `{}` contains multiple `[Headers]` sections",
                                                    entry.name.text
                                                ),
                                                item.span,
                                            )
                                            .primary_label(
                                                format!(
                                                    "I was expecting to find at most one `[Headers]` section in entry `{}`",
                                                    entry.name.text
                                                ),
                                                Level::Error,
                                            ));
                                }
                                None => {
                                    validated_headers = Some(validated_expr);
                                }
                            }
                        }
                        "Body" => {
                            if !matches!(validated_expr.ty, validated::Ty::Dictionary(_)) {
                                return Err(Diagnostic::error("Unexpected type", body_span)
                                    .primary_label(
                                        "I was expecting a dictionary here",
                                        Level::Error,
                                    ));
                            }

                            match validated_body {
                                Some(_) => {
                                    return Err(Diagnostic::error(
                                                format!(
                                                    "Entry `{}` contains multiple `[Body]` sections",
                                                    entry.name.text
                                                ),
                                                item.span,
                                            )
                                            .primary_label(
                                                format!(
                                                    "I was expecting to find at most one `[Body]` section in entry `{}`",
                                                    entry.name.text
                                                ),
                                                Level::Error,
                                            ));
                                }
                                None => {
                                    validated_body = Some(validated_expr);
                                }
                            }
                        }
                        _ => {
                            return Err(Diagnostic::error(
                                format!("Unknown section name `{}`", name.text),
                                name.span,
                            )
                            .primary_label(
                                "I don't know what to do with this section here",
                                Level::Error,
                            ));
                        }
                    }
                }
            }
        }

        Ok(validated::Entry {
            name: validated::Name {
                text: entry.name.text,
                span: entry.name.span,
            },
            request: validated_request,
            headers: validated_headers,
            body: validated_body,
        })
    }

    fn validate_expr(&self, expr: ast::Expr<'input>) -> Result<validated::Expr, Diagnostic> {
        match expr.kind {
            ast::ExprKind::StringLiteral(parts) => {
                let mut validated_parts = vec![];
                for part in parts {
                    match part {
                        ast::TemplatePart::Literal(raw) => {
                            let unescaped = unescape_string(raw, expr.span)?;
                            validated_parts.push(validated::TemplatePart::Literal(unescaped));
                        }
                        ast::TemplatePart::Expr(expr) => {
                            let validated_expr = self.validate_expr(expr)?;
                            validated_parts.push(validated::TemplatePart::Expr(validated_expr));
                        }
                    }
                }
                Ok(validated::Expr {
                    kind: validated::ExprKind::StringLiteral(validated_parts),
                    span: expr.span,
                    ty: validated::Ty::String,
                })
            }
            ast::ExprKind::IntegerLiteral(s) => {
                let value = s
                    .parse::<i64>()
                    .map_err(|_| Diagnostic::error("Invalid integer literal", expr.span))?;

                Ok(validated::Expr {
                    kind: validated::ExprKind::IntegerLiteral(value),
                    span: expr.span,
                    ty: validated::Ty::Integer,
                })
            }
            ast::ExprKind::FloatLiteral(s) => {
                let value = s
                    .parse::<f64>()
                    .map_err(|_| Diagnostic::error("Invalid float literal", expr.span))?;

                Ok(validated::Expr {
                    kind: validated::ExprKind::FloatLiteral(value),
                    span: expr.span,
                    ty: validated::Ty::Float,
                })
            }
            ast::ExprKind::NullLiteral => Ok(validated::Expr {
                kind: validated::ExprKind::NullLiteral,
                span: expr.span,
                ty: validated::Ty::Null,
            }),
            ast::ExprKind::Dictionary(fields) => self.validate_dictionary_fields(fields, expr.span),
            ast::ExprKind::Array(elements) => self.validate_array_elements(elements, expr.span),
            ast::ExprKind::NameRef(name) => {
                if let Some(konst) = self.globals.get(name) {
                    Ok(validated::Expr {
                        kind: validated::ExprKind::NameRef(name.to_string()),
                        span: konst.expr.span,
                        ty: konst.expr.ty.clone(),
                    })
                } else {
                    Err(Diagnostic::error("Unknown identifier", expr.span)
                        .primary_label("I don't know what this name is referring to", Level::Error))
                }
            }
        }
    }

    fn validate_dictionary_fields(
        &self,
        fields: Vec<ast::DictionaryField<'input>>,
        dictionary_span: Span,
    ) -> Result<validated::Expr, Diagnostic> {
        let mut validated_fields = Vec::with_capacity(fields.len());

        for field in fields {
            let key_span = field.key.span;
            let key = self.validate_expr(field.key)?;
            if key.ty != validated::Ty::String {
                return Err(Diagnostic::error("Mismatched types", key_span)
                    .primary_label("I was expecting a string as key here", Level::Error));
            }
            let value = self.validate_expr(field.value)?;
            validated_fields.push(validated::DictionaryField { key, value });
        }

        let value_types = validated_fields
            .iter()
            .map(|it| it.value.ty.clone())
            .collect();

        Ok(validated::Expr {
            kind: validated::ExprKind::Dictionary(validated_fields),
            span: dictionary_span,
            ty: validated::Ty::Dictionary(value_types),
        })
    }

    fn validate_array_elements(
        &self,
        elements: Vec<ast::Expr<'input>>,
        array_span: Span,
    ) -> Result<validated::Expr, Diagnostic> {
        let mut validated_exprs = Vec::with_capacity(elements.len());
        for elem in elements {
            let validated_expr = self.validate_expr(elem)?;
            validated_exprs.push(validated_expr);
        }

        let ty = self.infer_array_type(&validated_exprs);
        Ok(validated::Expr {
            kind: validated::ExprKind::Array(validated_exprs),
            span: array_span,
            ty: validated::Ty::Array(Box::new(ty)),
        })
    }

    fn infer_array_type(&self, elements: &[validated::Expr]) -> validated::Ty {
        let mut unique_types = vec![];
        for elem in elements.iter() {
            let ty = elem.ty.clone();
            if !unique_types.contains(&ty) {
                unique_types.push(ty);
            }
        }

        match unique_types.len() {
            0 => validated::Ty::Unknown,
            1 => unique_types.pop().unwrap(),
            _ => self.make_union_ty(unique_types),
        }
    }

    fn make_union_ty(&self, tys: Vec<validated::Ty>) -> validated::Ty {
        let mut flat = vec![];

        for ty in tys {
            match ty {
                validated::Ty::Union(inner) => {
                    for inner_ty in inner {
                        if !flat.contains(&inner_ty) {
                            flat.push(inner_ty);
                        }
                    }
                }
                other => {
                    if !flat.contains(&other) {
                        flat.push(other);
                    }
                }
            }
        }

        match flat.len() {
            0 => validated::Ty::Unknown,
            1 => flat.pop().unwrap(),
            _ => validated::Ty::Union(flat),
        }
    }
}

fn unescape_string(raw: &str, span: Span) -> Result<String, Diagnostic> {
    let mut result = String::new();
    let mut escape = false;
    for (i, c) in raw.char_indices() {
        if escape {
            let unescaped = match c {
                'n' => '\n',
                '\\' => '\\',
                '"' => '"',
                _ => {
                    let absolute_index = span.start + 1 + i;
                    let span = Span::new(absolute_index, absolute_index + c.len_utf8());
                    return Err(
                        Diagnostic::error(format!("Unknown character escape `{c}`"), span)
                            .primary_label(
                                "I don't know how to handle this character escape",
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
        assert_eq!(unescape_ok("foo"), "foo");
    }

    #[test]
    fn unescape_string_with_newline() {
        assert_eq!(unescape_ok(r#"foo\nbar"#), "foo\nbar");
    }

    #[test]
    fn unescape_string_with_quote() {
        assert_eq!(unescape_ok(r#"foo\"bar\""#), "foo\"bar\"");
    }

    #[test]
    fn unescape_string_with_backslash() {
        assert_eq!(unescape_ok(r#"foo\\bar"#), "foo\\bar");
    }

    #[test]
    fn unescape_string_mixed() {
        assert_eq!(
            unescape_ok(r#"one\\two\nthree\"end\""#),
            "one\\two\nthree\"end\""
        );
    }

    #[test]
    fn unescape_string_invalid_escape_points_to_correct_span() {
        let input = r#"foo\qbar"#;
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

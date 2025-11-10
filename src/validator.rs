use std::collections::{HashMap, hash_map};

use crate::{
    ast,
    diagnostic::{Diagnostic, Level},
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
                        // TODO: label
                        return Err(Diagnostic::error(
                            format!("Entry `{}` contains multiple requests", entry.name.text),
                            item.span,
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
            // TODO: escape sequences
            Ok(validated::Expr {
                kind: validated::ExprKind::StringLiteral(s.replace('"', "")),
                ty: validated::Ty::String,
            })
        }
    }
}


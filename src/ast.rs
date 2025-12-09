use std::fmt;

use crate::span::Span;

macro_rules! writeind {
    ($f:expr, $indent:expr, $($arg:tt)*) => {{
        writeln!($f, "{}{}", " ".repeat($indent), format!($($arg)*))
    }};
}

#[derive(Debug, Clone)]
pub struct SourceFile<'input> {
    pub items: Vec<Item<'input>>,
    pub span: Span,
}

impl<'input> SourceFile<'input> {
    pub fn dump<W: fmt::Write>(&self, w: &mut W, indent: usize) -> fmt::Result {
        writeind!(
            w,
            indent,
            "SourceFile@{}..{}",
            self.span.start,
            self.span.end
        )?;
        for item in &self.items {
            item.dump(w, indent + 1)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Item<'input> {
    pub kind: ItemKind<'input>,
    pub span: Span,
}

impl<'input> Item<'input> {
    pub fn dump<W: fmt::Write>(&self, w: &mut W, indent: usize) -> fmt::Result {
        match &self.kind {
            ItemKind::Entry(entry) => {
                writeind!(w, indent, "Entry@{}..{}", self.span.start, self.span.end)?;
                entry.dump(w, indent + 1)
            }
            ItemKind::Const(name, expr) => {
                writeind!(w, indent, "Const@{}..{}", self.span.start, self.span.end)?;
                name.dump(w, indent + 1)?;
                expr.dump(w, indent + 1)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum ItemKind<'input> {
    Entry(Entry<'input>),
    Const(Name<'input>, Expr<'input>),
}

#[derive(Debug, Clone)]
pub struct Entry<'input> {
    pub name: Name<'input>,
    pub body: Vec<EntryItem<'input>>,
}

impl<'input> Entry<'input> {
    pub fn dump<W: fmt::Write>(&self, w: &mut W, indent: usize) -> fmt::Result {
        self.name.dump(w, indent)?;
        for item in &self.body {
            item.dump(w, indent)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct EntryItem<'input> {
    pub kind: EntryItemKind<'input>,
    pub span: Span,
}

impl<'input> EntryItem<'input> {
    pub fn dump<W: fmt::Write>(&self, w: &mut W, indent: usize) -> fmt::Result {
        match &self.kind {
            EntryItemKind::Request(req) => {
                writeind!(w, indent, "Request@{}..{}", self.span.start, self.span.end)?;
                req.dump(w, indent + 1)
            }
            EntryItemKind::Section(name, body) => {
                writeind!(w, indent, "Section@{}..{}", self.span.start, self.span.end)?;
                name.dump(w, indent + 1)?;
                body.dump(w, indent + 1)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum EntryItemKind<'input> {
    Request(Request<'input>),
    Section(Name<'input>, Expr<'input>),
}

#[derive(Debug, Clone, Copy)]
pub struct Name<'input> {
    pub text: &'input str,
    pub span: Span,
}

impl<'input> Name<'input> {
    pub fn dump<W: fmt::Write>(&self, w: &mut W, indent: usize) -> fmt::Result {
        writeind!(
            w,
            indent,
            "Name@{}..{} {}",
            self.span.start,
            self.span.end,
            self.text
        )
    }
}

#[derive(Debug, Clone)]
pub struct Request<'input> {
    pub method: HttpMethod,
    pub url: Expr<'input>,
}

impl<'input> Request<'input> {
    pub fn dump<W: fmt::Write>(&self, w: &mut W, indent: usize) -> fmt::Result {
        writeind!(w, indent, "{}", self.method)?;
        self.url.dump(w, indent)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

impl fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Patch => "PATCH",
            HttpMethod::Delete => "DELETE",
        };
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone)]
pub struct Expr<'input> {
    pub kind: ExprKind<'input>,
    pub span: Span,
}

impl<'input> Expr<'input> {
    pub fn dump<W: fmt::Write>(&self, w: &mut W, indent: usize) -> fmt::Result {
        match &self.kind {
            ExprKind::NameRef(name) => {
                writeind!(
                    w,
                    indent,
                    "NameRef@{}..{} {}",
                    self.span.start,
                    self.span.end,
                    name
                )
            }
            ExprKind::StringLiteral(parts) => {
                writeind!(
                    w,
                    indent,
                    "StringLiteral@{}..{}",
                    self.span.start,
                    self.span.end
                )?;
                for part in parts {
                    part.dump(w, indent + 1)?;
                }
                Ok(())
            }
            ExprKind::IntegerLiteral(lit) => {
                writeind!(
                    w,
                    indent,
                    "IntegerLiteral@{}..{} {}",
                    self.span.start,
                    self.span.end,
                    lit
                )
            }
            ExprKind::FloatLiteral(lit) => {
                writeind!(
                    w,
                    indent,
                    "FloatLiteral@{}..{} {}",
                    self.span.start,
                    self.span.end,
                    lit
                )
            }
            ExprKind::NullLiteral => {
                writeind!(
                    w,
                    indent,
                    "NullLiteral@{}..{}",
                    self.span.start,
                    self.span.end
                )
            }
            ExprKind::Dictionary(fields) => {
                writeind!(
                    w,
                    indent,
                    "Dictionary@{}..{}",
                    self.span.start,
                    self.span.end
                )?;
                for field in fields {
                    field.dump(w, indent + 1)?;
                }
                Ok(())
            }
            ExprKind::Array(elems) => {
                writeind!(w, indent, "Array@{}..{}", self.span.start, self.span.end)?;
                for elem in elems {
                    elem.dump(w, indent + 1)?;
                }
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum ExprKind<'input> {
    NameRef(&'input str),
    StringLiteral(Vec<TemplatePart<'input>>),
    IntegerLiteral(&'input str),
    FloatLiteral(&'input str),
    NullLiteral,
    Dictionary(Vec<DictionaryField<'input>>),
    Array(Vec<Expr<'input>>),
}

#[derive(Debug, Clone)]
pub enum TemplatePart<'input> {
    Literal(&'input str, Span),
    Expr(Expr<'input>),
}

impl<'input> TemplatePart<'input> {
    pub fn dump<W: fmt::Write>(&self, w: &mut W, indent: usize) -> fmt::Result {
        match self {
            TemplatePart::Literal(lit, span) => {
                writeind!(w, indent, "Literal@{}..{} {}", span.start, span.end, lit)
            }
            TemplatePart::Expr(expr) => expr.dump(w, indent),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DictionaryField<'input> {
    pub key: Expr<'input>,
    pub value: Expr<'input>,
}

impl<'input> DictionaryField<'input> {
    pub fn dump<W: fmt::Write>(&self, w: &mut W, indent: usize) -> fmt::Result {
        self.key.dump(w, indent)?;
        self.value.dump(w, indent)
    }
}

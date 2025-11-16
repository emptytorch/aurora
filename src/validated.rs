use std::collections::HashMap;

pub struct SourceFile<'input> {
    pub entries: HashMap<&'input str, Entry<'input>>,
    pub globals: HashMap<&'input str, Expr>,
}

#[derive(Debug, Clone)]
pub struct Entry<'input> {
    pub name: &'input str,
    pub request: Option<Request>,
    pub headers: Option<Expr>,
    pub body: Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct Request {
    pub method: HttpMethod,
    pub url: Expr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
}

#[derive(Debug, Clone)]
pub struct Expr {
    pub kind: ExprKind,
    pub ty: Ty,
}

#[derive(Debug, Clone)]
pub enum ExprKind {
    NameRef(String),
    StringLiteral(String),
    IntegerLiteral(i64),
    FloatLiteral(f64),
    Dictionary(Vec<DictionaryField>),
}

#[derive(Debug, Clone)]
pub struct DictionaryField {
    pub key: Expr,
    pub value: Expr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ty {
    String,
    Integer,
    Float,
    Dictionary(Vec<Ty>),
}

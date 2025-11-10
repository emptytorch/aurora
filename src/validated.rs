#[derive(Debug, Clone)]
pub struct Entry<'input> {
    pub name: &'input str,
    pub request: Option<Request>,
}

#[derive(Debug, Clone)]
pub struct Request {
    pub method: HttpMethod,
    pub url: Expr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
}

#[derive(Debug, Clone)]
pub struct Expr {
    pub kind: ExprKind,
    pub ty: Ty,
}

#[derive(Debug, Clone)]
pub enum ExprKind {
    StringLiteral(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ty {
    String,
}

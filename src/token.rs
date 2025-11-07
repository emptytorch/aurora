use crate::span::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind<'input> {
    Identifier(&'input str),
    HttpMethod(HttpMethod),
    Integer(&'input str),
    String(&'input str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token<'input> {
    pub kind: TokenKind<'input>,
    pub span: Span,
}

impl<'input> Token<'input> {
    pub fn new(kind: TokenKind<'input>, span: Span) -> Self {
        Self { kind, span }
    }
}

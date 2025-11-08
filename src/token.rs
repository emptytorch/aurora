use crate::span::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind<'input> {
    Identifier(&'input str),
    HttpMethod(HttpMethod),
    Keyword(Keyword),
    Integer(&'input str),
    String(&'input str),
    /// `:`
    Colon,
    /// E.g., `{`
    OpenDelim(Delim),
    /// E.g., `}`
    CloseDelim(Delim),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    /// `GET`
    Get,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    /// `entry`
    Entry,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Delim {
    /// `{` or `}`
    Brace,
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

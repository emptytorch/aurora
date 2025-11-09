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
    Delim(Delim),
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
    /// `{`
    OpenBrace,
    /// `}`
    CloseBrace,
}

impl Delim {
    pub fn is_open(&self) -> bool {
        match self {
            Delim::OpenBrace => true,
            Delim::CloseBrace => false,
        }
    }
}

impl std::fmt::Display for Delim {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Delim::OpenBrace => write!(f, "{{"),
            Delim::CloseBrace => write!(f, "}}"),
        }
    }
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

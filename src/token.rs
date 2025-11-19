use crate::span::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind<'input> {
    /// E.g., `foo"`
    Identifier(&'input str),
    /// E.g., `GET`
    HttpMethod(HttpMethod),
    /// E.g.,`entry`
    Keyword(Keyword),
    /// E.g., `12.3`
    Float(&'input str),
    /// E.g., `123`
    Integer(&'input str),
    /// E.g., `"foo{{bar}}baz"`
    String(Vec<TemplatePart<'input>>),
    /// `:`
    Colon,
    /// `,`
    Comma,
    /// `=`
    Eq,
    /// E.g., `{`
    Delim(Delim),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    /// `GET`
    Get,
    /// `POST`
    Post,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    /// `entry`
    Entry,
    /// `const`
    Const,
    /// `null`
    Null,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Delim {
    /// `{`
    OpenBrace,
    /// `[`
    OpenBrack,
    /// `}`
    CloseBrace,
    /// `]`
    CloseBrack,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplatePart<'input> {
    Literal(&'input str),
    Code(Vec<Token<'input>>),
}

impl Delim {
    pub fn is_open(&self) -> bool {
        match self {
            Delim::OpenBrace | Delim::OpenBrack => true,
            Delim::CloseBrace | Delim::CloseBrack => false,
        }
    }
}

impl std::fmt::Display for Delim {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Delim::OpenBrace => write!(f, "{{"),
            Delim::OpenBrack => write!(f, "["),
            Delim::CloseBrace => write!(f, "}}"),
            Delim::CloseBrack => write!(f, "]"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token<'input> {
    pub kind: TokenKind<'input>,
    pub span: Span,
    pub skipped_newline: bool,
}

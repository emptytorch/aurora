use crate::span::Span;

#[derive(Debug, Clone)]
pub struct SourceFile<'input> {
    pub items: Vec<Item<'input>>,
}

#[derive(Debug, Clone)]
pub struct Item<'input> {
    pub kind: ItemKind<'input>,
    pub span: Span,
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

#[derive(Debug, Clone)]
pub struct EntryItem<'input> {
    pub kind: EntryItemKind<'input>,
    pub span: Span,
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

#[derive(Debug, Clone)]
pub struct Request<'input> {
    pub method: HttpMethod,
    pub url: Expr<'input>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

#[derive(Debug, Clone)]
pub struct Expr<'input> {
    pub kind: ExprKind<'input>,
    pub span: Span,
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
    Literal(&'input str),
    Expr(Expr<'input>),
}

#[derive(Debug, Clone)]
pub struct DictionaryField<'input> {
    pub key: Expr<'input>,
    pub value: Expr<'input>,
}

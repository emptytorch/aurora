use indexmap::IndexMap;

pub struct SourceFile<'input> {
    pub entries: IndexMap<&'input str, Entry<'input>>,
    pub globals: IndexMap<&'input str, Expr>,
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
    StringLiteral(Vec<TemplatePart>),
    IntegerLiteral(i64),
    FloatLiteral(f64),
    NullLiteral,
    Dictionary(Vec<DictionaryField>),
    Array(Vec<Expr>),
}

#[derive(Debug, Clone)]
pub enum TemplatePart {
    Literal(String),
    Expr(Expr),
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
    Null,
    Dictionary(Vec<Ty>),
    Array(Box<Ty>),
    Unknown,
}

impl std::fmt::Display for Ty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ty::String => write!(f, "string"),
            Ty::Integer => write!(f, "int"),
            Ty::Float => write!(f, "float"),
            Ty::Null => write!(f, "null"),
            Ty::Dictionary(_) => write!(f, "dict"),
            Ty::Array(ty) => write!(f, "{ty}[]"),
            Ty::Unknown => write!(f, "?"),
        }
    }
}

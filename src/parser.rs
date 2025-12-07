use crate::{
    ast::{
        DictionaryField, Entry, EntryItem, EntryItemKind, Expr, ExprKind, HttpMethod, Item,
        ItemKind, Name, Request, SourceFile, TemplatePart,
    },
    diagnostic::{Diagnostic, Level},
    lexer,
    span::Span,
    token::{self, Delim, Keyword, Token, TokenKind},
};

pub fn parse<'input>(input: &'input str) -> Result<SourceFile<'input>, Diagnostic> {
    let tokens = lexer::lex(input)?;
    let mut parser = Parser::new(tokens);
    parser.parse()
}

struct Parser<'input> {
    tokens: Vec<Token<'input>>,
    pos: usize,
}

impl<'input> Parser<'input> {
    fn new(tokens: Vec<Token<'input>>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn parse(&mut self) -> Result<SourceFile<'input>, Diagnostic> {
        let mut items = vec![];
        while self.peek().is_some() {
            let item = self.parse_item()?;
            items.push(item);
        }
        Ok(SourceFile { items })
    }

    fn parse_item(&mut self) -> Result<Item<'input>, Diagnostic> {
        if let Some(span) = self.eat_keyword(Keyword::Entry) {
            return self.parse_entry(span);
        }

        if let Some(span) = self.eat_keyword(Keyword::Const) {
            return self.parse_const(span);
        }

        Err(Diagnostic::error("Expected item", self.peek_span())
            .primary_label("I was expecting an item here", Level::Error))
    }

    fn parse_entry(&mut self, entry_span: Span) -> Result<Item<'input>, Diagnostic> {
        let name = self.parse_name().ok_or(
            Diagnostic::error("Expected identifier", self.peek_span())
                .primary_label("I was expecting a name here", Level::Error),
        )?;

        _ = self.expect_delim(Delim::OpenBrace)?;
        let mut entry_items = vec![];
        while let Some(item) = self.opt_parse_entry_item()? {
            entry_items.push(item);
        }
        let close_span = self.expect_delim(Delim::CloseBrace)?;
        let span = entry_span.to(close_span);
        Ok(Item {
            kind: ItemKind::Entry(Entry {
                name,
                body: entry_items,
            }),
            span,
        })
    }

    fn parse_const(&mut self, const_span: Span) -> Result<Item<'input>, Diagnostic> {
        let name = self.parse_name().ok_or(
            Diagnostic::error("Expected identifier", self.peek_span())
                .primary_label("I was expecting a variable name here", Level::Error),
        )?;

        if self.eat(TokenKind::Eq).is_none() {
            return Err(Diagnostic::error("Expected `=`", self.peek_span()));
        }

        let expr = self.parse_expr()?;
        self.expect_newline()?;
        let span = const_span.to(expr.span);
        Ok(Item {
            kind: ItemKind::Const(name, expr),
            span,
        })
    }

    fn opt_parse_entry_item(&mut self) -> Result<Option<EntryItem<'input>>, Diagnostic> {
        match self.peek() {
            Some(&Token {
                kind: TokenKind::HttpMethod(token::HttpMethod::Get),
                span: method_span,
                ..
            }) => {
                self.bump();
                let url = self.parse_expr()?;
                self.expect_newline()?;
                let url_span = url.span;
                Ok(Some(EntryItem {
                    kind: EntryItemKind::Request(Request {
                        method: HttpMethod::Get,
                        url,
                    }),
                    span: method_span.to(url_span),
                }))
            }
            Some(&Token {
                kind: TokenKind::HttpMethod(token::HttpMethod::Post),
                span: method_span,
                ..
            }) => {
                self.bump();
                let url = self.parse_expr()?;
                let url_span = url.span;
                Ok(Some(EntryItem {
                    kind: EntryItemKind::Request(Request {
                        method: HttpMethod::Post,
                        url,
                    }),
                    span: method_span.to(url_span),
                }))
            }
            Some(&Token {
                kind: TokenKind::HttpMethod(token::HttpMethod::Put),
                span: method_span,
                ..
            }) => {
                self.bump();
                let url = self.parse_expr()?;
                let url_span = url.span;
                Ok(Some(EntryItem {
                    kind: EntryItemKind::Request(Request {
                        method: HttpMethod::Put,
                        url,
                    }),
                    span: method_span.to(url_span),
                }))
            }
            Some(&Token {
                kind: TokenKind::HttpMethod(token::HttpMethod::Patch),
                span: method_span,
                ..
            }) => {
                self.bump();
                let url = self.parse_expr()?;
                let url_span = url.span;
                Ok(Some(EntryItem {
                    kind: EntryItemKind::Request(Request {
                        method: HttpMethod::Patch,
                        url,
                    }),
                    span: method_span.to(url_span),
                }))
            }
            Some(&Token {
                kind: TokenKind::HttpMethod(token::HttpMethod::Delete),
                span: method_span,
                ..
            }) => {
                self.bump();
                let url = self.parse_expr()?;
                let url_span = url.span;
                Ok(Some(EntryItem {
                    kind: EntryItemKind::Request(Request {
                        method: HttpMethod::Delete,
                        url,
                    }),
                    span: method_span.to(url_span),
                }))
            }
            Some(&Token {
                kind: TokenKind::Delim(Delim::OpenBrack),
                span: open_span,
                ..
            }) => {
                self.bump();
                let name = self.parse_name().ok_or(
                    Diagnostic::error("Expected identifier", self.peek_span())
                        .primary_label("I was expecting a section name here", Level::Error),
                )?;
                _ = self.expect_delim(Delim::CloseBrack)?;
                let body = self.parse_expr()?;
                let span = open_span.to(body.span);
                Ok(Some(EntryItem {
                    kind: EntryItemKind::Section(name, body),
                    span,
                }))
            }
            _ => Ok(None),
        }
    }

    fn parse_expr(&mut self) -> Result<Expr<'input>, Diagnostic> {
        match self.opt_parse_expr()? {
            Some(expr) => Ok(expr),
            None => Err(Diagnostic::error("Expected expression", self.peek_span())
                .primary_label("I was expecting an expression here", Level::Error)),
        }
    }

    fn opt_parse_expr(&mut self) -> Result<Option<Expr<'input>>, Diagnostic> {
        match self.peek() {
            Some(&Token {
                kind: TokenKind::Identifier(s),
                span,
                ..
            }) => {
                self.bump();
                Ok(Some(Expr {
                    kind: ExprKind::NameRef(s),
                    span,
                }))
            }
            Some(&Token {
                kind: TokenKind::String(ref parts),
                span,
                ..
            }) => {
                let parts = parts.clone();
                self.bump();
                let mut ast_parts = vec![];
                for part in parts {
                    match part {
                        token::TemplatePart::Literal(s) => {
                            ast_parts.push(TemplatePart::Literal(s));
                        }
                        token::TemplatePart::Code(tokens) => {
                            let mut parser = Parser::new(tokens);
                            let expr = parser.parse_expr()?;
                            ast_parts.push(TemplatePart::Expr(expr));
                        }
                    }
                }

                Ok(Some(Expr {
                    kind: ExprKind::StringLiteral(ast_parts),
                    span,
                }))
            }
            Some(&Token {
                kind: TokenKind::Integer(s),
                span,
                ..
            }) => {
                self.bump();
                Ok(Some(Expr {
                    kind: ExprKind::IntegerLiteral(s),
                    span,
                }))
            }
            Some(&Token {
                kind: TokenKind::Float(s),
                span,
                ..
            }) => {
                self.bump();
                Ok(Some(Expr {
                    kind: ExprKind::FloatLiteral(s),
                    span,
                }))
            }
            Some(&Token {
                kind: TokenKind::Keyword(Keyword::Null),
                span,
                ..
            }) => {
                self.bump();
                Ok(Some(Expr {
                    kind: ExprKind::NullLiteral,
                    span,
                }))
            }
            Some(&Token {
                kind: TokenKind::Delim(Delim::OpenBrace),
                span: open_span,
                ..
            }) => {
                self.bump();
                let fields = self.parse_dictionary_fields()?;
                let close_span = self.expect_delim(Delim::CloseBrace)?;
                let span = open_span.to(close_span);
                Ok(Some(Expr {
                    kind: ExprKind::Dictionary(fields),
                    span,
                }))
            }
            Some(&Token {
                kind: TokenKind::Delim(Delim::OpenBrack),
                span: open_span,
                ..
            }) => {
                self.bump();
                let mut elements = vec![];

                loop {
                    match self.peek() {
                        Some(Token {
                            kind: TokenKind::Delim(Delim::CloseBrack),
                            ..
                        })
                        | None => break,
                        _ => {}
                    }

                    let element = self.parse_expr()?;
                    elements.push(element);

                    if self.eat(TokenKind::Comma).is_none() {
                        match self.peek() {
                            Some(Token {
                                kind: TokenKind::Delim(Delim::CloseBrack),
                                ..
                            }) => {
                                break;
                            }
                            Some(_) => {
                                return Err(Diagnostic::error(
                                    "Unexpected token",
                                    self.peek_span(),
                                )
                                .primary_label("I was expecting a comma here", Level::Error));
                            }
                            None => break,
                        }
                    }
                }

                let close_span = self.expect_delim(Delim::CloseBrack)?;
                let span = open_span.to(close_span);
                Ok(Some(Expr {
                    kind: ExprKind::Array(elements),
                    span,
                }))
            }
            _ => Ok(None),
        }
    }

    fn parse_dictionary_fields(&mut self) -> Result<Vec<DictionaryField<'input>>, Diagnostic> {
        let mut fields = vec![];

        loop {
            match self.peek() {
                Some(Token {
                    kind: TokenKind::Delim(Delim::CloseBrace),
                    ..
                })
                | None => break,
                _ => {}
            }

            let field = self.parse_dictionary_field()?;
            fields.push(field);

            if self.eat(TokenKind::Comma).is_none() {
                match self.peek() {
                    Some(Token {
                        kind: TokenKind::Delim(Delim::CloseBrace),
                        ..
                    }) => {
                        break;
                    }
                    Some(_) => {
                        return Err(Diagnostic::error("Unexpected token", self.peek_span())
                            .primary_label("I was expecting a comma here", Level::Error));
                    }
                    None => break,
                }
            }
        }

        Ok(fields)
    }

    fn parse_dictionary_field(&mut self) -> Result<DictionaryField<'input>, Diagnostic> {
        let key = self.parse_expr()?;
        if self.eat(TokenKind::Colon).is_none() {
            return Err(Diagnostic::error("Unexpected token", self.peek_span())
                .primary_label("I was expecting a colon here", Level::Error));
        }

        let value = self.parse_expr()?;
        Ok(DictionaryField { key, value })
    }

    fn parse_name(&mut self) -> Option<Name<'input>> {
        if let Some(&Token {
            kind: TokenKind::Identifier(text),
            span,
            ..
        }) = self.peek()
        {
            self.bump();
            Some(Name { text, span })
        } else {
            None
        }
    }

    fn eat(&mut self, kind: TokenKind) -> Option<Span> {
        if let Some(token) = self.peek()
            && token.kind == kind
        {
            let span = token.span;
            self.bump();
            Some(span)
        } else {
            None
        }
    }

    fn eat_keyword(&mut self, kw: Keyword) -> Option<Span> {
        if let Some(&Token {
            kind: TokenKind::Keyword(kw2),
            span,
            ..
        }) = self.peek()
            && kw == kw2
        {
            self.bump();
            Some(span)
        } else {
            None
        }
    }

    fn expect_delim(&mut self, delim: Delim) -> Result<Span, Diagnostic> {
        if let Some(&Token {
            kind: TokenKind::Delim(delim2),
            span,
            ..
        }) = self.peek()
            && delim == delim2
        {
            self.bump();
            Ok(span)
        } else {
            let label_message = if delim.is_open() {
                format!("I was expecting an opening delimiter `{delim}` here")
            } else {
                format!("I was expecting a closing delimiter `{delim}` here")
            };

            Err(Diagnostic::error("Expected delimiter", self.peek_span())
                .primary_label(label_message, Level::Error))
        }
    }

    fn expect_newline(&mut self) -> Result<(), Diagnostic> {
        match self.peek() {
            None => Ok(()),
            Some(token) if token.skipped_newline => Ok(()),
            Some(token) => Err(Diagnostic::error("Missing newline", token.span)
                .primary_label("I was expecting a newline here", Level::Error)),
        }
    }

    fn peek(&self) -> Option<&Token<'input>> {
        self.tokens.get(self.pos)
    }

    fn peek_span(&self) -> Span {
        if let Some(token) = self.peek() {
            token.span
        } else if let Some(last) = self.tokens.last() {
            last.span
        } else {
            Span::new(0, 0)
        }
    }

    fn bump(&mut self) {
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
    }
}
